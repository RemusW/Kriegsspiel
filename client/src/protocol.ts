// Messages exchanged with the server, as JSON. Mirrors `crates/server/src/protocol.rs`;
// change both together.

import type { Board } from "./model";

export type ServerMessage = { type: "snapshot"; board: Board };

export const DEFAULT_PORT = 7878;
