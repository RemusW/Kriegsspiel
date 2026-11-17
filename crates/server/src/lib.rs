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

    /// Parse and apply one message from a client, then wake every connection if the
    /// document changed.
    async fn receive(&self, umpire: UmpireId, text: &str) {
        let message = match serde_json::from_str(text) {
            Ok(message) => message,
            Err(err) => {
                eprintln!("ignoring malformed message from umpire {umpire}: {err}");
                return;
            }
        };
        let changed = apply(&mut *self.doc.write().await, umpire, message);
        if changed {
            // Unlike `send`, this does not fail when no connection is listening.
            self.changed.send_replace(());
        }
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

/// Apply one client message to the document. Returns whether the document changed.
fn apply(doc: &mut Doc, umpire: UmpireId, message: ClientMessage) -> bool {
    match message {
        ClientMessage::Spawn { kind, side, pos } => {
            // A number too large for f32, such as 1e39, arrives as infinity.
            if !pos.iter().all(|v| v.is_finite()) {
                return false;
            }
            doc.edit(umpire, |board| {
                board.spawn(kind, side, pos);
            })
        }
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
                Some(Ok(Message::Text(text))) => game.receive(umpire, &text).await,
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
        assert!(apply(&mut doc, 0, spawn_at([3.0, -4.0])));
        assert_eq!(doc.board.pawns.len(), 1);
        assert_eq!(doc.board.pawns[0].pawn.pos, [3.0, -4.0]);
        doc.undo(0);
        assert!(doc.board.pawns.is_empty());
    }

    #[test]
    fn spawn_rejects_non_finite_positions() {
        let mut doc = Doc::default();
        assert!(!apply(&mut doc, 0, spawn_at([f32::INFINITY, 0.0])));
        assert!(doc.board.pawns.is_empty());
    }

    #[test]
    fn spawn_parses_from_client_json() {
        let json = r#"{"type":"spawn","kind":"Infantry","side":"Blue","pos":[1.5,2]}"#;
        let message: ClientMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message, spawn_at([1.5, 2.0]));
    }
}
