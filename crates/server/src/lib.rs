//! Game server: holds the game state and serves it to clients over WebSocket at `/ws`.

pub mod model;
pub mod protocol;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::get,
};
use tokio::sync::RwLock;

use model::{Doc, Side, UnitKind};
use protocol::ServerMessage;

pub const DEFAULT_PORT: u16 = 7878;

/// One running game. Cheap to clone; all clones share the same state.
#[derive(Clone, Default)]
pub struct Game {
    doc: Arc<RwLock<Doc>>,
}

impl Game {
    /// A game with four fixed units on the live board.
    pub fn demo() -> Self {
        let mut doc = Doc::default();
        doc.board.spawn(UnitKind::Infantry, Side::Blue, [-15.0, -5.0]);
        doc.board.spawn(UnitKind::Cavalry, Side::Blue, [-5.0, 5.0]);
        doc.board.spawn(UnitKind::Artillery, Side::Red, [10.0, 0.0]);
        doc.board.spawn(UnitKind::Infantry, Side::Red, [20.0, 10.0]);
        Self {
            doc: Arc::new(RwLock::new(doc)),
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

/// Serve one connected client: send the live board, then hold the connection until it closes
/// or fails.
async fn client(mut socket: WebSocket, game: Game) {
    let snapshot = ServerMessage::Snapshot {
        board: game.doc.read().await.board.clone(),
    };
    let Ok(text) = serde_json::to_string(&snapshot) else {
        return;
    };
    if socket.send(Message::Text(text.into())).await.is_err() {
        return;
    }
    while let Some(Ok(message)) = socket.recv().await {
        if matches!(message, Message::Close(_)) {
            break;
        }
    }
}
