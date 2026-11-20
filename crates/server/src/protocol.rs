//! Messages exchanged with clients, as JSON. Their TypeScript definitions are written to
//! `client/src/bindings/` when the tests run.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::model::{Board, Side, Turn, UnitKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub enum ServerMessage {
    /// Everything this client should show: the board, the recorded turns it may step
    /// through, and which of them it is looking at.
    Snapshot {
        board: Board,
        /// Every recorded turn, oldest first. Their boards are not sent; stepping to one
        /// asks the server for it.
        turns: Vec<TurnInfo>,
        /// The recorded turn this client is viewing, as an index into `turns`. `None` is the
        /// live board.
        viewing: Option<usize>,
    },
}

/// What a client is told about one recorded turn: enough to list and label it, without the
/// board it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "protocol.ts")]
pub struct TurnInfo {
    /// Milliseconds since the Unix epoch, when an umpire recorded the turn.
    pub recorded_at: u64,
    /// How many units the recorded board holds.
    pub pawns: usize,
}

impl From<&Turn> for TurnInfo {
    fn from(turn: &Turn) -> Self {
        Self {
            recorded_at: turn.recorded_at,
            pawns: turn.board.pawns.len(),
        }
    }
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
    /// Record the live board as a new turn, which every client may then step through.
    /// Refused while the sender is viewing a recorded turn, since that is not the board it
    /// would record.
    RecordTurn,
    /// Move this umpire's view one turn back or forward. Stepping back from the live board
    /// goes to the last turn; stepping forward past the last turn returns to the live board.
    StepTurn { forward: bool },
    /// Show this umpire one recorded turn, or the live board with `null`. Only this umpire's
    /// view changes; the others keep working on the live board.
    ViewTurn {
        #[serde(default)]
        turn: Option<usize>,
    },
    /// Copy the turn this umpire is viewing into the live board, as one undoable edit, and
    /// return them to the live board.
    RestoreTurn,
}
