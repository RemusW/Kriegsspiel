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

use model::{Doc, Side, UmpireId, UnitKind};
use protocol::{ClientMessage, ServerMessage};

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
        ClientMessage::Move { moves } => {
            if !moves.iter().all(|m| m.pos.iter().all(|v| v.is_finite())) {
                return Effect::Me;
            }
            // Another umpire may have removed a unit mid-drag: the rest still move, but with
            // nothing left to move there is no reason to wake every connection.
            let any_known = moves
                .iter()
                .any(|m| doc.board.pawns.iter().any(|unit| unit.id == m.id));
            if !any_known {
                return Effect::Me;
            }
            edited(doc.edit(umpire, |board| {
                for m in &moves {
                    if let Some(unit) = board.unit_mut(m.id) {
                        unit.pawn.pos = m.pos;
                    }
                }
            }))
        }
    }
}

/// A refused edit still sends its umpire a board, so their client stops showing it.
fn edited(applied: bool) -> Effect {
    if applied { Effect::Everyone } else { Effect::Me }
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
    use protocol::UnitMove;

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

    fn move_to(moves: &[(u64, [f32; 2])]) -> ClientMessage {
        ClientMessage::Move {
            moves: moves.iter().map(|&(id, pos)| UnitMove { id, pos }).collect(),
        }
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
    fn move_parses_from_client_json() {
        let json = r#"{"type":"move","moves":[{"id":3,"pos":[1.5,-2]}]}"#;
        let message: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message, move_to(&[(3, [1.5, -2.0])]));
    }
}
