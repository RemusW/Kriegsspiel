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

/// Where one unit should end up and how it should sit. Every value is absolute, so a command
/// states the result it wants instead of how far a pointer or key moved it. A field left out
/// stays as it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub struct Placement {
    pub id: u64,
    /// Position on the map plane (world X, world Z).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pos: Option<[f32; 2]>,
    /// Rotation around the vertical axis, in radians.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rot: Option<f32>,
    /// Footprint multiplier, from [`MIN_SCALE`](crate::model::MIN_SCALE) to
    /// [`MAX_SCALE`](crate::model::MAX_SCALE).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub scale: Option<f32>,
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
    /// Set units' positions, rotations and sizes. One drag or one held key sends one command,
    /// so changing several units at once is one undo step. Ids that are no longer on the board
    /// are skipped.
    Place { units: Vec<Placement> },
    /// Take back this umpire's last edit. Other umpires' later changes stay, and any field
    /// one of them has changed since is left alone.
    Undo,
    /// Put back this umpire's last undone edit.
    Redo,
}
