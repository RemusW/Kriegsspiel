// Messages exchanged with the server, as JSON. The types are generated from
// `crates/server/src/protocol.rs` into `bindings/protocol.ts` by
// `cargo test -p kriegsspiel-server`.

export type {
  ClientMessage,
  Placement,
  ServerMessage,
  TurnInfo,
} from "./bindings/protocol";

/** Matches `DEFAULT_PORT` in `crates/server/src/lib.rs`. */
export const DEFAULT_PORT = 7878;
