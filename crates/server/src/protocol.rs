//! Messages exchanged with clients, as JSON. Their TypeScript definitions are written to
//! `client/src/bindings/` when the tests run.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::model::{Board, Side, UnitKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub enum ServerMessage {
    /// The board this client should show.
    Snapshot { board: Board },
}

/// Where one unit should end up. The position is absolute, so a command states the result
/// it wants instead of how far a pointer moved.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub struct UnitMove {
    pub id: u64,
    /// Position on the map plane (world X, world Z).
    pub pos: [f32; 2],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub enum ClientMessage {
    /// Add a unit to the live board at a point on the map plane (world X, world Z).
    Spawn {
        kind: UnitKind,
        side: Side,
        pos: [f32; 2],
    },
    /// Put units at new positions. One drag sends one command, so moving several units at
    /// once is one undo step. Ids that are no longer on the board are skipped.
    Move { moves: Vec<UnitMove> },
    /// Take back this umpire's last edit. Other umpires' later changes stay, and any field
    /// one of them has changed since is left alone.
    Undo,
    /// Put back this umpire's last undone edit.
    Redo,
}
