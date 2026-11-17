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
}
