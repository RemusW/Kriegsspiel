//! Game server: holds the game state and serves it to clients over WebSocket at `/ws`.

pub mod model;
pub mod protocol;

use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::get,
};
use tokio::sync::{RwLock, watch};

use model::{Doc, MAX_SCALE, MIN_SCALE, Side, Step, UmpireId, UnitKind};
use protocol::{ClientMessage, Placement, ServerMessage};

pub const DEFAULT_PORT: u16 = 7878;

/// One running game. Cheap to clone; all clones share the same state.
#[derive(Clone)]
pub struct Game {
    doc: Arc<RwLock<Doc>>,
    /// Wakes every connection after the document changes. It carries no data: each
    /// connection then reads the view its own client may see.
    changed: watch::Sender<()>,
    next_umpire: Arc<AtomicU64>,
}

impl Game {
    pub fn new(doc: Doc) -> Self {
        Self {
            doc: Arc::new(RwLock::new(doc)),
            changed: watch::Sender::new(()),
            next_umpire: Arc::new(AtomicU64::new(0)),
        }
    }

    /// A game with four fixed units on the live board.
    pub fn demo() -> Self {
        let mut doc = Doc::default();
        doc.board.spawn(UnitKind::Infantry, Side::Blue, [-15.0, -5.0]);
        doc.board.spawn(UnitKind::Cavalry, Side::Blue, [-5.0, 5.0]);
        doc.board.spawn(UnitKind::Artillery, Side::Red, [10.0, 0.0]);
        doc.board.spawn(UnitKind::Infantry, Side::Red, [20.0, 10.0]);
        Self::new(doc)
    }

    /// Parse and apply one message from a client. Wakes every connection when the live board
    /// changed; otherwise the caller sends the board this umpire alone needs.
    async fn receive(&self, umpire: UmpireId, text: &str) -> Effect {
        let message = match serde_json::from_str(text) {
            Ok(message) => message,
            Err(err) => {
                eprintln!("ignoring malformed message from umpire {umpire}: {err}");
                // A message that did not parse cannot have changed anything, but the sender
                // may be showing an edit it thinks is in flight.
                return Effect::Me;
            }
        };
        let effect = apply(&mut *self.doc.write().await, umpire, message);
        if effect == Effect::Everyone {
            // Unlike `send`, this does not fail when no connection is listening. It wakes this
            // connection too, which is how the sender gets its own board.
            self.changed.send_replace(());
        }
        effect
    }

    /// Send this umpire's view of the document. Fails once the connection has closed.
    async fn send_view(&self, socket: &mut WebSocket, umpire: UmpireId) -> Result<(), axum::Error> {
        // Clone and release the lock before sending, so a slow client never holds it.
        let board = self.doc.read().await.shown(umpire).clone();
        let text = serde_json::to_string(&ServerMessage::Snapshot { board })
            .expect("boards always serialize");
        socket.send(Message::Text(text.into())).await
    }
}

/// Who has to be sent a board after a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Nothing changed and nobody needs telling.
    None,
    /// Only the umpire who sent the command: their view changed, or it was refused and their
    /// client is showing an edit that never happened.
    Me,
    /// The live board changed, so every connection needs its own view again.
    Everyone,
}

/// Apply one client message to the document. Returns who must be sent a board.
fn apply(doc: &mut Doc, umpire: UmpireId, message: ClientMessage) -> Effect {
    match message {
        ClientMessage::Spawn { kind, side, pos } => {
            // A number too large for f32, such as 1e39, arrives as infinity.
            if !pos.iter().all(|v| v.is_finite()) {
                return Effect::Me;
            }
            edited(doc.edit(umpire, |board| {
                board.spawn(kind, side, pos);
            }))
        }
        ClientMessage::Place { units } => {
            if !units.iter().all(valid_placement) {
                return Effect::Me;
            }
            // Another umpire may have removed a unit mid-drag: the rest still change, but with
            // nothing left to change there is no reason to wake every connection.
            let any_known = units
                .iter()
                .any(|p| doc.board.pawns.iter().any(|unit| unit.id == p.id));
            if !any_known {
                return Effect::Me;
            }
            edited(doc.edit(umpire, |board| {
                for p in &units {
                    if let Some(unit) = board.unit_mut(p.id) {
                        let pawn = &mut unit.pawn;
                        pawn.pos = p.pos.unwrap_or(pawn.pos);
                        pawn.rot = p.rot.unwrap_or(pawn.rot);
                        pawn.scale = p.scale.unwrap_or(pawn.scale);
                    }
                }
            }))
        }
        // Each umpire has their own history, so an undo needs no checking: it can only take
        // back something this umpire did.
        ClientMessage::Undo => stepped(doc.undo(umpire)),
        ClientMessage::Redo => stepped(doc.redo(umpire)),
    }
}

/// Every value is finite, and the scale is within the allowed range. A number too large for
/// f32, such as 1e39, arrives as infinity.
fn valid_placement(p: &Placement) -> bool {
    let pos_ok = p.pos.is_none_or(|pos| pos.iter().all(|v| v.is_finite()));
    let rot_ok = p.rot.is_none_or(f32::is_finite);
    // NaN fails the range check too.
    let scale_ok = p.scale.is_none_or(|s| (MIN_SCALE..=MAX_SCALE).contains(&s));
    pos_ok && rot_ok && scale_ok
}

/// A refused edit still sends its umpire a board, so their client stops showing it.
fn edited(applied: bool) -> Effect {
    if applied { Effect::Everyone } else { Effect::Me }
}

/// An empty history changed nothing anywhere, so nobody is told; a step refused because the
/// umpire is viewing a recorded turn answers them, so their client is not left waiting.
fn stepped(step: Step) -> Effect {
    match step {
        Step::Empty => Effect::None,
        Step::ReadOnly => Effect::Me,
        Step::Applied | Step::Partial => Effect::Everyone,
    }
}

pub fn router(game: Game) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(upgrade))
        .with_state(game)
}

pub async fn serve(addr: SocketAddr, game: Game) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router(game)).await
}

async fn upgrade(ws: WebSocketUpgrade, State(game): State<Game>) -> Response {
    ws.on_upgrade(|socket| client(socket, game))
}

/// Serve one connected client until it disconnects: send its view whenever the document
/// changes, and apply the messages it sends.
async fn client(mut socket: WebSocket, game: Game) {
    // Every connection is a new umpire until umpires can identify themselves.
    let umpire = game.next_umpire.fetch_add(1, Ordering::Relaxed);
    // Subscribe before the first send, so a change made in between still wakes this loop.
    let mut changed = game.changed.subscribe();
    if game.send_view(&mut socket, umpire).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            message = socket.recv() => match message {
                Some(Ok(Message::Text(text))) => {
                    // `Everyone` arrives through the watch channel below, like any other change.
                    if game.receive(umpire, &text).await == Effect::Me
                        && game.send_view(&mut socket, umpire).await.is_err()
                    {
                        break;
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                // Pings are answered by axum itself; binary messages are not used.
                Some(Ok(_)) => {}
            },
            Ok(()) = changed.changed() => {
                if game.send_view(&mut socket, umpire).await.is_err() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_at(pos: [f32; 2]) -> ClientMessage {
        ClientMessage::Spawn {
            kind: UnitKind::Infantry,
            side: Side::Blue,
            pos,
        }
    }

    #[test]
    fn spawn_adds_one_undoable_unit() {
        let mut doc = Doc::default();
        assert_eq!(apply(&mut doc, 0, spawn_at([3.0, -4.0])), Effect::Everyone);
        assert_eq!(doc.board.pawns.len(), 1);
        assert_eq!(doc.board.pawns[0].pawn.pos, [3.0, -4.0]);
        doc.undo(0);
        assert!(doc.board.pawns.is_empty());
    }

    #[test]
    fn spawn_rejects_non_finite_positions() {
        let mut doc = Doc::default();
        // Refused, and only the umpire who sent it is told.
        assert_eq!(apply(&mut doc, 0, spawn_at([f32::INFINITY, 0.0])), Effect::Me);
        assert!(doc.board.pawns.is_empty());
    }

    #[test]
    fn spawn_parses_from_client_json() {
        let json = r#"{"type":"spawn","kind":"Infantry","side":"Blue","pos":[1.5,2]}"#;
        let message: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message, spawn_at([1.5, 2.0]));
    }

    fn place(units: &[Placement]) -> ClientMessage {
        ClientMessage::Place {
            units: units.to_vec(),
        }
    }

    fn move_to(moves: &[(u64, [f32; 2])]) -> ClientMessage {
        let units: Vec<_> = moves
            .iter()
            .map(|&(id, pos)| Placement {
                id,
                pos: Some(pos),
                ..Placement::default()
            })
            .collect();
        place(&units)
    }

    /// A board with an infantry unit at the origin and a cavalry unit at [1, 1].
    fn two_units() -> Doc {
        let mut doc = Doc::default();
        doc.board.spawn(UnitKind::Infantry, Side::Blue, [0.0, 0.0]);
        doc.board.spawn(UnitKind::Cavalry, Side::Red, [1.0, 1.0]);
        doc
    }

    #[test]
    fn move_places_every_unit_in_one_undo_step() {
        let mut doc = two_units();
        let effect = apply(&mut doc, 0, move_to(&[(0, [5.0, 6.0]), (1, [-2.0, 3.0])]));
        assert_eq!(effect, Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.pos, [5.0, 6.0]);
        assert_eq!(doc.board.pawns[1].pawn.pos, [-2.0, 3.0]);

        // One drag is one undo step, however many units it moved.
        assert_eq!(doc.undo(0), model::Step::Applied);
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);
        assert_eq!(doc.board.pawns[1].pawn.pos, [1.0, 1.0]);
    }

    #[test]
    fn move_skips_ids_that_left_the_board() {
        let mut doc = two_units();
        let effect = apply(&mut doc, 0, move_to(&[(1, [4.0, 4.0]), (99, [7.0, 7.0])]));
        assert_eq!(effect, Effect::Everyone);
        assert_eq!(doc.board.pawns.len(), 2);
        assert_eq!(doc.board.pawns[1].pawn.pos, [4.0, 4.0]);
    }

    #[test]
    fn move_of_unknown_ids_only_changes_nothing() {
        let mut doc = two_units();
        assert_eq!(apply(&mut doc, 0, move_to(&[(99, [7.0, 7.0])])), Effect::Me);
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);
    }

    #[test]
    fn move_rejects_non_finite_positions() {
        let mut doc = two_units();
        let effect = apply(&mut doc, 0, move_to(&[(0, [3.0, 3.0]), (1, [f32::NAN, 0.0])]));
        assert_eq!(effect, Effect::Me);
        // The whole command is refused, so the valid part is not applied either.
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);
    }

    #[test]
    fn an_edit_refused_while_viewing_a_turn_answers_the_sender() {
        let mut doc = two_units();
        doc.record_turn();
        doc.step_turn(0, false);
        assert!(doc.viewing(0).is_some());
        // Read-only for this umpire, but their client is already showing the drag.
        assert_eq!(apply(&mut doc, 0, move_to(&[(0, [9.0, 9.0])])), Effect::Me);
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);
    }

    #[test]
    fn undo_takes_back_the_last_move_and_redo_puts_it_back() {
        let mut doc = two_units();
        apply(&mut doc, 0, move_to(&[(0, [5.0, 6.0])]));

        assert_eq!(apply(&mut doc, 0, ClientMessage::Undo), Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);
        assert_eq!(apply(&mut doc, 0, ClientMessage::Redo), Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.pos, [5.0, 6.0]);
    }

    #[test]
    fn undo_only_takes_back_the_senders_own_edits() {
        let mut doc = two_units();
        apply(&mut doc, 0, move_to(&[(0, [5.0, 6.0])]));
        apply(&mut doc, 1, move_to(&[(1, [7.0, 8.0])]));

        // Umpire 1 undoing leaves umpire 0's move where it is.
        assert_eq!(apply(&mut doc, 1, ClientMessage::Undo), Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.pos, [5.0, 6.0]);
        assert_eq!(doc.board.pawns[1].pawn.pos, [1.0, 1.0]);
    }

    #[test]
    fn undo_with_nothing_to_take_back_tells_nobody() {
        let mut doc = two_units();
        // No board to send: an empty history is not a refusal, nothing was being shown.
        assert_eq!(apply(&mut doc, 0, ClientMessage::Undo), Effect::None);
        assert_eq!(apply(&mut doc, 0, ClientMessage::Redo), Effect::None);
    }

    #[test]
    fn a_new_edit_drops_what_was_undone() {
        let mut doc = two_units();
        apply(&mut doc, 0, move_to(&[(0, [5.0, 6.0])]));
        apply(&mut doc, 0, ClientMessage::Undo);
        apply(&mut doc, 0, move_to(&[(0, [1.0, 2.0])]));

        assert_eq!(apply(&mut doc, 0, ClientMessage::Redo), Effect::None);
        assert_eq!(doc.board.pawns[0].pawn.pos, [1.0, 2.0]);
    }

    #[test]
    fn undo_while_viewing_a_turn_answers_the_sender() {
        let mut doc = two_units();
        apply(&mut doc, 0, move_to(&[(0, [5.0, 6.0])]));
        doc.record_turn();
        doc.step_turn(0, false);
        assert!(doc.viewing(0).is_some());

        assert_eq!(apply(&mut doc, 0, ClientMessage::Undo), Effect::Me);
        assert_eq!(doc.board.pawns[0].pawn.pos, [5.0, 6.0]);
    }

    #[test]
    fn history_commands_parse_from_client_json() {
        assert_eq!(
            serde_json::from_str::<ClientMessage>(r#"{"type":"undo"}"#).unwrap(),
            ClientMessage::Undo
        );
        assert_eq!(
            serde_json::from_str::<ClientMessage>(r#"{"type":"redo"}"#).unwrap(),
            ClientMessage::Redo
        );
    }

    #[test]
    fn place_parses_from_client_json() {
        let json = r#"{"type":"place","units":[{"id":3,"pos":[1.5,-2]}]}"#;
        let message: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message, move_to(&[(3, [1.5, -2.0])]));
    }

    #[test]
    fn place_sets_only_the_fields_it_carries() {
        let mut doc = two_units();
        let turn = Placement {
            id: 0,
            pos: Some([2.0, 3.0]),
            rot: Some(0.5),
            ..Placement::default()
        };
        let grow = Placement {
            id: 1,
            scale: Some(2.0),
            ..Placement::default()
        };
        assert_eq!(apply(&mut doc, 0, place(&[turn, grow])), Effect::Everyone);

        let [a, b] = [&doc.board.pawns[0].pawn, &doc.board.pawns[1].pawn];
        assert_eq!((a.pos, a.rot, a.scale), ([2.0, 3.0], 0.5, 1.0));
        assert_eq!((b.pos, b.rot, b.scale), ([1.0, 1.0], 0.0, 2.0));

        // Rotating a formation moves and turns its units in one undo step.
        assert_eq!(doc.undo(0), model::Step::Applied);
        let [a, b] = [&doc.board.pawns[0].pawn, &doc.board.pawns[1].pawn];
        assert_eq!((a.pos, a.rot), ([0.0, 0.0], 0.0));
        assert_eq!(b.scale, 1.0);
    }

    #[test]
    fn place_rejects_bad_rotations_and_scales() {
        let bad = [
            Placement {
                id: 0,
                rot: Some(f32::INFINITY),
                ..Placement::default()
            },
            Placement {
                id: 0,
                scale: Some(f32::NAN),
                ..Placement::default()
            },
            Placement {
                id: 0,
                scale: Some(0.0),
                ..Placement::default()
            },
            Placement {
                id: 0,
                scale: Some(MAX_SCALE * 2.0),
                ..Placement::default()
            },
        ];
        for p in bad {
            let mut doc = two_units();
            assert_eq!(apply(&mut doc, 0, place(&[p])), Effect::Me, "{p:?}");
            assert_eq!(doc.board.pawns[0].pawn.rot, 0.0);
            assert_eq!(doc.board.pawns[0].pawn.scale, 1.0);
        }
    }

    #[test]
    fn placement_json_leaves_out_unset_fields() {
        let p = Placement {
            id: 3,
            rot: Some(1.0),
            ..Placement::default()
        };
        assert_eq!(serde_json::to_string(&p).unwrap(), r#"{"id":3,"rot":1.0}"#);
    }
}
