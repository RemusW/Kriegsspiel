//! Messages exchanged with clients, as JSON. Mirrored by `client/src/protocol.ts`;
//! change both together.

use serde::{Deserialize, Serialize};

use crate::model::Board;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// The live board.
    Snapshot { board: Board },
}
