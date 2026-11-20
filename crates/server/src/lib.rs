//! Game server: holds the game state and serves it to clients over WebSocket at `/ws`.
//!
//! Alongside the socket it serves the images a game uses from `/assets/{id}`, and whole save
//! files from `/save` and `/load`. Those are plain HTTP because a 16 MB map image belongs in
//! something the browser can cache and a save belongs in something it can download, neither
//! of which a WebSocket does for us.

pub mod assets;
pub mod model;
pub mod protocol;
pub mod save;

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
        DefaultBodyLimit, Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bytes::Bytes;
use tokio::sync::{RwLock, watch};
use tower_http::cors::{Any, CorsLayer};

use assets::{Asset, AssetId, AssetStore, is_asset_id};
use model::{Doc, MAX_SCALE, MIN_SCALE, Side, Step, UmpireId, UnitKind};
use protocol::{ClientMessage, Placement, ServerMessage, TurnInfo};
use save::{MAX_SAVE_BYTES, SAVE_MIME, SaveError};

pub const DEFAULT_PORT: u16 = 7878;

/// One running game. Cheap to clone; all clones share the same state.
///
/// Two locks, always taken in the order they are declared here: the document first, then the
/// images. Every path that needs both follows that order, which is what stops a save (which
/// reads both) deadlocking against a load (which replaces both).
#[derive(Clone)]
pub struct Game {
    doc: Arc<RwLock<Doc>>,
    /// The images the document names. Append-only while a game runs; replaced wholesale when
    /// a save is loaded.
    assets: Arc<RwLock<AssetStore>>,
    /// Wakes every connection after the document changes. It carries no data: each
    /// connection then reads the view its own client may see.
    changed: watch::Sender<()>,
    next_umpire: Arc<AtomicU64>,
}

impl Game {
    pub fn new(doc: Doc) -> Self {
        Self::with_assets(doc, AssetStore::default())
    }

    pub fn with_assets(doc: Doc, assets: AssetStore) -> Self {
        Self {
            doc: Arc::new(RwLock::new(doc)),
            assets: Arc::new(RwLock::new(assets)),
            changed: watch::Sender::new(()),
            next_umpire: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Store one uploaded image and return its id, or say why it was refused. The image is
    /// checked and hashed before the store is locked, so commands are not held up behind it.
    async fn add_asset(&self, bytes: Bytes) -> Result<AssetId, assets::AssetError> {
        let (id, asset) = blocking(move || Asset::read(bytes)).await?;
        self.assets.write().await.add(id.clone(), asset);
        Ok(id)
    }

    /// Write the whole game to a save file. The locks are held only to copy what goes in it,
    /// which shares the images' bytes rather than copying them, so edits carry on while the
    /// file is written.
    pub async fn save(&self) -> Result<Vec<u8>, SaveError> {
        let (board, turns, assets) = {
            let doc = self.doc.read().await;
            let assets = self.assets.read().await;
            (doc.board.clone(), doc.turns.clone(), assets.clone())
        };
        blocking(move || save::pack(&board, &turns, &assets)).await
    }

    /// Replace the game with the contents of a save file, and tell every client. Nothing is
    /// changed unless the whole file reads, so a damaged file leaves the game alone.
    /// Every umpire's undo history goes with the document that held it.
    pub async fn load(&self, bytes: Bytes) -> Result<(), SaveError> {
        // Read before any lock is taken: the game carries on while the file is checked.
        let (loaded, loaded_assets) = blocking(move || save::unpack(&bytes)).await?;
        {
            let mut doc = self.doc.write().await;
            let mut assets = self.assets.write().await;
            *doc = loaded;
            *assets = loaded_assets;
        }
        // Umpires viewing a turn are sent the live board: the turns they were stepping
        // through belong to the game that was just replaced.
        self.changed.send_replace(());
        Ok(())
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
        // The document first, then the images, as everywhere else: see `Game`.
        let effect = {
            let mut doc = self.doc.write().await;
            let assets = self.assets.read().await;
            apply(&mut doc, &assets, umpire, message)
        };
        if effect == Effect::Everyone {
            // Unlike `send`, this does not fail when no connection is listening. It wakes this
            // connection too, which is how the sender gets its own board.
            self.changed.send_replace(());
        }
        effect
    }

    /// Send this umpire's view of the document. Fails once the connection has closed.
    async fn send_view(&self, socket: &mut WebSocket, umpire: UmpireId) -> Result<(), axum::Error> {
        // Take everything the snapshot needs under one read, so the board and the turn it
        // came from cannot disagree, and release the lock before sending: a slow client must
        // never hold it.
        let snapshot = {
            let doc = self.doc.read().await;
            ServerMessage::Snapshot {
                board: doc.shown(umpire).clone(),
                turns: doc.turns.iter().map(TurnInfo::from).collect(),
                viewing: doc.viewing(umpire),
            }
        };
        send(socket, &snapshot).await
    }
}

/// Run slow, synchronous work (hashing images, zipping a save) on tokio's thread pool for
/// blocking work. On an async worker it would hold that thread until done, stalling every
/// connection the worker serves.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(work)
        .await
        .expect("blocking work does not panic")
}

/// Send one message to a client. Fails once the connection has closed.
async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).expect("server messages always serialize");
    socket.send(Message::Text(text.into())).await
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

/// Apply one client message to the document. Returns who must be sent a board. The images
/// are read-only here: a command may only name one that has already been uploaded.
fn apply(
    doc: &mut Doc,
    assets: &AssetStore,
    umpire: UmpireId,
    message: ClientMessage,
) -> Effect {
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
            if !any_on_board(doc, units.iter().map(|p| p.id)) {
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
        ClientMessage::RecordTurn => {
            // Read-only while viewing a turn: the board that would be recorded is not the one
            // this umpire is looking at.
            if doc.viewing(umpire).is_some() {
                return Effect::Me;
            }
            doc.record_turn(now_millis());
            // The turn list is part of every client's view, so everyone is told.
            Effect::Everyone
        }
        // Which turn an umpire looks at is theirs alone, so only they are sent a board, and
        // they are sent one even when the turn asked for does not exist: their client is
        // showing a timeline and must be told what it is really looking at.
        ClientMessage::StepTurn { forward } => {
            doc.step_turn(umpire, forward);
            Effect::Me
        }
        ClientMessage::ViewTurn { turn } => {
            doc.view_turn(umpire, turn);
            Effect::Me
        }
        ClientMessage::RestoreTurn => edited(doc.restore_viewed_turn(umpire)),
        ClientMessage::SetMapImage { asset } => {
            if !known_asset(assets, &asset) {
                return Effect::Me;
            }
            edited(doc.edit(umpire, |board| board.map_image = asset))
        }
        ClientMessage::SetUnitImage { units, asset } => {
            if !known_asset(assets, &asset) {
                return Effect::Me;
            }
            if !any_on_board(doc, units.iter().copied()) {
                return Effect::Me;
            }
            edited(doc.edit(umpire, |board| {
                for id in &units {
                    if let Some(unit) = board.unit_mut(*id) {
                        unit.pawn.image = asset.clone();
                    }
                }
            }))
        }
    }
}

/// Whether any of these units is still on the live board. Another umpire may have removed
/// them mid-drag: the rest still change, but with nothing left to change there is no reason
/// to wake every connection, and the sender is still answered.
fn any_on_board(doc: &Doc, ids: impl IntoIterator<Item = u64>) -> bool {
    ids.into_iter()
        .any(|id| doc.board.pawns.iter().any(|unit| unit.id == id))
}

/// Whether an image a command names can be used: either it is already in the store, or the
/// command is clearing the image. A board that named an image the store does not hold would
/// save to a file that could not be loaded back.
///
/// A refusal is logged because a snapshot cannot carry a reason: until commands have request
/// ids and replies of their own, the server's own output is the only place this is visible.
fn known_asset(assets: &AssetStore, asset: &Option<String>) -> bool {
    match asset.as_deref() {
        None => true,
        Some(id) if assets.contains(id) => true,
        Some(id) => {
            eprintln!("refusing a command: no image {id} has been uploaded");
            false
        }
    }
}

/// Wall-clock milliseconds since the Unix epoch, for stamping a recorded turn. A clock set
/// before 1970 stamps it 0 rather than failing the command.
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
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
        .route("/assets/{id}", get(asset))
        .route("/save", get(download))
        .route(
            "/load",
            // A save is far larger than axum's default body limit, which exists to stop one
            // request eating the server's memory; this raises it for this one route.
            post(upload).layer(DefaultBodyLimit::max(MAX_SAVE_BYTES)),
        )
        .layer(cors())
        .with_state(game)
}

/// Let a page on another origin read these routes.
///
/// The client is never on the same origin as the game server: `localhost:1420` in
/// development, `tauri://localhost` in the packaged app, and the relay for a player. Without
/// these headers the browser refuses to hand the page a response the server did answer,
/// which is silent in a way that is easy to misread: three.js fetches a map image with
/// `crossOrigin`, so the image goes unused, and a `POST` of a save never leaves the browser
/// because the preflight `OPTIONS` is answered 405.
///
/// Any origin is allowed, which is no weaker than what is already true: CORS does not apply
/// to the WebSocket, so any page can already open one and send commands. Keeping strangers
/// out needs umpire identity, which is an open question, not CORS.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Serve one image. The id is the hash of the bytes, so a URL can never come to mean a
/// different image and the browser may keep it for good.
async fn asset(Path(id): Path<String>, State(game): State<Game>) -> Response {
    if !is_asset_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    // Cloned out of the store so the lock is released before the bytes are written to the
    // socket; `Asset` holds its bytes behind a refcount, so this copies nothing.
    let Some(asset) = game.assets.read().await.get(&id).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (
        [
            (header::CONTENT_TYPE, asset.kind.mime()),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        asset.bytes,
    )
        .into_response()
}

/// The whole game as one save file, offered as a download.
async fn download(State(game): State<Game>) -> Response {
    match game.save().await {
        // No `Content-Disposition`: the client names the file when it hands it to the browser.
        Ok(file) => ([(header::CONTENT_TYPE, SAVE_MIME)], file).into_response(),
        Err(error) => {
            // Saving reads state the server already holds, so a failure here is a bug in the
            // writer rather than anything the umpire did.
            eprintln!("could not write a save file: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    }
}

/// Replace the game with an uploaded save file. The body is the file itself.
async fn upload(State(game): State<Game>, body: Bytes) -> Response {
    match game.load(body).await {
        // The board that comes back over the socket is the answer.
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        // The file came from whoever is at the keyboard, so the reason goes back to be shown.
        Err(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

pub async fn serve(addr: SocketAddr, game: Game) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router(game)).await
}

async fn upgrade(ws: WebSocketUpgrade, State(game): State<Game>) -> Response {
    // An uploaded image can arrive as one frame, so both limits have to clear the largest
    // image the store would accept; the store itself does the refusing, with a reason. The
    // frame limit defaults to 16 MiB, well under that, and overrunning it kills the
    // connection instead of answering, so it has to be raised alongside the message limit.
    let limit = assets::MAX_ASSET_BYTES + 1024;
    ws.max_message_size(limit)
        .max_frame_size(limit)
        .on_upgrade(|socket| client(socket, game))
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
                // A binary frame is an image being uploaded: the only thing a client sends
                // that is not JSON. The command that uses it follows as its own message, and
                // a WebSocket keeps order, so the image is always in the store by the time
                // that command is applied. Only a refusal is answered: the client hashed the
                // file itself, so an acceptance would tell it nothing.
                Some(Ok(Message::Binary(bytes))) => {
                    if let Err(error) = game.add_asset(bytes).await {
                        let reply = ServerMessage::AssetRejected {
                            reason: error.to_string(),
                        };
                        if send(&mut socket, &reply).await.is_err() {
                            break;
                        }
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                // Pings are answered by axum itself.
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
    use assets::test_png;

    /// Apply a command to a game with no images in it. Commands that name an image call
    /// `super::apply` with a store of their own.
    fn apply(doc: &mut Doc, umpire: UmpireId, message: ClientMessage) -> Effect {
        super::apply(doc, &AssetStore::default(), umpire, message)
    }

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
        doc.record_turn(0);
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
        doc.record_turn(0);
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
    fn recording_a_turn_stamps_it_and_tells_everyone() {
        let mut doc = two_units();
        assert_eq!(apply(&mut doc, 0, ClientMessage::RecordTurn), Effect::Everyone);
        assert_eq!(doc.turns.len(), 1);
        assert_eq!(doc.turns[0].board.pawns.len(), 2);
        assert!(doc.turns[0].recorded_at > 0, "stamped from the wall clock");

        // The recorded turn is a copy: later edits leave it alone.
        apply(&mut doc, 0, move_to(&[(0, [9.0, 9.0])]));
        assert_eq!(doc.turns[0].board.pawns[0].pawn.pos, [0.0, 0.0]);
    }

    #[test]
    fn recording_while_viewing_a_turn_is_refused() {
        let mut doc = two_units();
        apply(&mut doc, 0, ClientMessage::RecordTurn);
        apply(&mut doc, 0, ClientMessage::StepTurn { forward: false });

        assert_eq!(apply(&mut doc, 0, ClientMessage::RecordTurn), Effect::Me);
        assert_eq!(doc.turns.len(), 1);
    }

    #[test]
    fn stepping_turns_changes_only_the_senders_view() {
        let mut doc = two_units();
        apply(&mut doc, 0, ClientMessage::RecordTurn);
        apply(&mut doc, 0, move_to(&[(0, [5.0, 5.0])]));

        // Back from the live board goes to the last turn, which that umpire alone sees.
        assert_eq!(
            apply(&mut doc, 0, ClientMessage::StepTurn { forward: false }),
            Effect::Me
        );
        assert_eq!(doc.viewing(0), Some(0));
        assert_eq!(doc.viewing(1), None);
        assert_eq!(doc.shown(0).pawns[0].pawn.pos, [0.0, 0.0]);
        assert_eq!(doc.shown(1).pawns[0].pawn.pos, [5.0, 5.0]);

        // Forward past the last turn returns to the live board.
        apply(&mut doc, 0, ClientMessage::StepTurn { forward: true });
        assert_eq!(doc.viewing(0), None);
    }

    #[test]
    fn viewing_a_turn_that_does_not_exist_answers_the_sender() {
        let mut doc = two_units();
        apply(&mut doc, 0, ClientMessage::RecordTurn);

        let jump = |turn| ClientMessage::ViewTurn { turn };
        assert_eq!(apply(&mut doc, 0, jump(Some(0))), Effect::Me);
        assert_eq!(doc.viewing(0), Some(0));
        // Refused, but the sender is still sent a board: its timeline must show the truth.
        assert_eq!(apply(&mut doc, 0, jump(Some(7))), Effect::Me);
        assert_eq!(doc.viewing(0), Some(0));
        assert_eq!(apply(&mut doc, 0, jump(None)), Effect::Me);
        assert_eq!(doc.viewing(0), None);
    }

    #[test]
    fn restoring_a_turn_changes_the_live_board_for_everyone() {
        let mut doc = two_units();
        apply(&mut doc, 0, ClientMessage::RecordTurn);
        apply(&mut doc, 0, move_to(&[(0, [5.0, 5.0])]));
        apply(&mut doc, 0, ClientMessage::ViewTurn { turn: Some(0) });

        assert_eq!(apply(&mut doc, 0, ClientMessage::RestoreTurn), Effect::Everyone);
        assert_eq!(doc.viewing(0), None, "back on the live board");
        assert_eq!(doc.board.pawns[0].pawn.pos, [0.0, 0.0]);

        // One undoable edit, so a restore made by mistake is taken back like any other.
        assert_eq!(apply(&mut doc, 0, ClientMessage::Undo), Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.pos, [5.0, 5.0]);
    }

    #[test]
    fn restoring_without_viewing_a_turn_answers_the_sender() {
        let mut doc = two_units();
        assert_eq!(apply(&mut doc, 0, ClientMessage::RestoreTurn), Effect::Me);
        assert!(doc.turns.is_empty());
    }

    #[test]
    fn turn_commands_parse_from_client_json() {
        let parse = |json: &str| serde_json::from_str::<ClientMessage>(json).unwrap();
        assert_eq!(parse(r#"{"type":"recordTurn"}"#), ClientMessage::RecordTurn);
        assert_eq!(
            parse(r#"{"type":"stepTurn","forward":true}"#),
            ClientMessage::StepTurn { forward: true }
        );
        assert_eq!(
            parse(r#"{"type":"viewTurn","turn":2}"#),
            ClientMessage::ViewTurn { turn: Some(2) }
        );
        assert_eq!(
            parse(r#"{"type":"viewTurn","turn":null}"#),
            ClientMessage::ViewTurn { turn: None }
        );
        assert_eq!(parse(r#"{"type":"restoreTurn"}"#), ClientMessage::RestoreTurn);
    }

    #[test]
    fn a_snapshot_carries_the_turn_list_and_the_turn_being_viewed() {
        let mut doc = two_units();
        apply(&mut doc, 0, ClientMessage::RecordTurn);
        apply(&mut doc, 0, ClientMessage::ViewTurn { turn: Some(0) });

        let snapshot = ServerMessage::Snapshot {
            board: doc.shown(0).clone(),
            turns: doc.turns.iter().map(TurnInfo::from).collect(),
            viewing: doc.viewing(0),
        };
        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["type"], "snapshot");
        assert_eq!(json["viewing"], 0);
        assert_eq!(json["turns"][0]["pawns"], 2);
        assert!(json["turns"][0]["recordedAt"].is_number());
        // The recorded boards stay on the server.
        assert!(json["turns"][0]["board"].is_null());
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
    fn setting_the_map_image_is_one_undoable_edit() {
        let mut doc = two_units();
        let mut store = AssetStore::default();
        let map = store.insert(test_png(2048, 1024)).unwrap();

        let set = |asset| ClientMessage::SetMapImage { asset };
        let effect = super::apply(&mut doc, &store, 0, set(Some(map.clone())));
        assert_eq!(effect, Effect::Everyone);
        assert_eq!(doc.board.map_image, Some(map.clone()));

        // Undo puts the table back as it was, like any other edit.
        assert_eq!(doc.undo(0), model::Step::Applied);
        assert_eq!(doc.board.map_image, None);
        assert_eq!(doc.redo(0), model::Step::Applied);

        // And it can be cleared.
        assert_eq!(super::apply(&mut doc, &store, 0, set(None)), Effect::Everyone);
        assert_eq!(doc.board.map_image, None);
    }

    #[test]
    fn an_image_the_store_does_not_hold_is_refused() {
        let mut doc = two_units();
        let store = AssetStore::default();
        let stranger = assets::asset_id(b"never uploaded");

        let map = ClientMessage::SetMapImage {
            asset: Some(stranger.clone()),
        };
        assert_eq!(super::apply(&mut doc, &store, 0, map), Effect::Me);
        assert_eq!(doc.board.map_image, None, "the board names no missing image");

        let unit = ClientMessage::SetUnitImage {
            units: vec![0],
            asset: Some(stranger),
        };
        assert_eq!(super::apply(&mut doc, &store, 0, unit), Effect::Me);
        assert_eq!(doc.board.pawns[0].pawn.image, None);
    }

    #[test]
    fn one_image_covers_the_whole_selection_in_one_step() {
        let mut doc = two_units();
        let mut store = AssetStore::default();
        let badge = store.insert(test_png(64, 64)).unwrap();

        let message = ClientMessage::SetUnitImage {
            // An id that has left the board is skipped, as with a move.
            units: vec![0, 1, 99],
            asset: Some(badge.clone()),
        };
        assert_eq!(super::apply(&mut doc, &store, 0, message), Effect::Everyone);
        assert_eq!(doc.board.pawns[0].pawn.image, Some(badge.clone()));
        assert_eq!(doc.board.pawns[1].pawn.image, Some(badge));

        assert_eq!(doc.undo(0), model::Step::Applied);
        assert!(doc.board.pawns.iter().all(|u| u.pawn.image.is_none()));
    }

    #[test]
    fn image_commands_parse_from_client_json() {
        let parse = |json: &str| serde_json::from_str::<ClientMessage>(json).unwrap();
        let id = "a".repeat(64);
        assert_eq!(
            parse(&format!(r#"{{"type":"setMapImage","asset":"{id}"}}"#)),
            ClientMessage::SetMapImage {
                asset: Some(id.clone())
            }
        );
        assert_eq!(
            parse(r#"{"type":"setMapImage","asset":null}"#),
            ClientMessage::SetMapImage { asset: None }
        );
        assert_eq!(
            parse(&format!(
                r#"{{"type":"setUnitImage","units":[1,2],"asset":"{id}"}}"#
            )),
            ClientMessage::SetUnitImage {
                units: vec![1, 2],
                asset: Some(id)
            }
        );
    }

    #[tokio::test]
    async fn a_game_saves_and_loads_itself() {
        let game = Game::new(Doc::default());
        let id = game.add_asset(test_png(1024, 512)).await.unwrap();
        game.receive(0, r#"{"type":"spawn","kind":"Cavalry","side":"Red","pos":[4,5]}"#)
            .await;
        game.receive(0, &format!(r#"{{"type":"setMapImage","asset":"{id}"}}"#))
            .await;
        game.receive(0, r#"{"type":"recordTurn"}"#).await;

        let file = game.save().await.unwrap();

        // Into a different game, to show the file carries everything and needs nothing of
        // the server that wrote it.
        let other = Game::new(Doc::default());
        other.load(file.into()).await.unwrap();
        let doc = other.doc.read().await;
        assert_eq!(doc.board.map_image, Some(id.clone()));
        assert_eq!(doc.board.pawns[0].pawn.pos, [4.0, 5.0]);
        assert_eq!(doc.turns.len(), 1);
        // The image came with it, so the new game can serve it to its own clients.
        assert!(other.assets.read().await.contains(&id));
    }

    #[tokio::test]
    async fn a_damaged_save_leaves_the_game_alone() {
        let game = Game::demo();
        let before = game.doc.read().await.board.pawns.len();

        let error = game.load(Bytes::from_static(b"not a save file")).await.unwrap_err();
        assert!(matches!(error, SaveError::Archive(_)), "{error}");
        assert_eq!(game.doc.read().await.board.pawns.len(), before);
    }

    #[tokio::test]
    async fn the_http_routes_may_be_read_from_the_client_s_origin() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let game = Game::new(Doc::default());
        let id = game.add_asset(test_png(8, 8)).await.unwrap();
        let app = router(game);

        let from_the_page = |method: &str, uri: String| {
            Request::builder()
                .method(method)
                .uri(uri)
                .header("origin", "http://localhost:1420")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap()
        };

        // What the browser asks before it will POST a save file. Answered 405 before the
        // headers were added, which aborts the upload without it ever being sent.
        let preflight = app
            .clone()
            .oneshot(from_the_page("OPTIONS", "/load".into()))
            .await
            .unwrap();
        assert!(preflight.status().is_success(), "{}", preflight.status());
        assert_eq!(preflight.headers()["access-control-allow-origin"], "*");

        // And a map image, which three.js fetches with `crossOrigin` and cannot use without
        // the same header, however well the request itself went.
        let image = app
            .oneshot(from_the_page("GET", format!("/assets/{id}")))
            .await
            .unwrap();
        assert_eq!(image.status(), StatusCode::OK);
        assert_eq!(image.headers()["access-control-allow-origin"], "*");
    }

    #[tokio::test]
    async fn an_upload_is_not_an_edit() {
        let game = Game::new(Doc::default());
        let changed = game.changed.subscribe();

        let id = game.add_asset(test_png(8, 8)).await.unwrap();
        // Storing an image changes no board, so no client is sent one: a 16 MB map arriving
        // must not make every player redraw. The command that uses it is what they hear.
        assert!(!changed.has_changed().unwrap());

        game.receive(0, &format!(r#"{{"type":"setMapImage","asset":"{id}"}}"#))
            .await;
        assert!(changed.has_changed().unwrap(), "laying it down is an edit");
    }

    #[tokio::test]
    async fn an_upload_that_is_not_an_image_is_refused() {
        let game = Game::new(Doc::default());
        let error = game
            .add_asset(Bytes::from_static(b"<svg/>"))
            .await
            .unwrap_err();
        assert_eq!(error, assets::AssetError::Unsupported);
        assert!(game.assets.read().await.is_empty());
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
