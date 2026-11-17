// Messages exchanged with the server, as JSON. The types are generated from
// `crates/server/src/protocol.rs` into `bindings/` by `cargo test -p kriegsspiel-server`.

export type { ServerMessage } from "./bindings/ServerMessage";

/** Matches `DEFAULT_PORT` in `crates/server/src/lib.rs`. */
export const DEFAULT_PORT = 7878;
