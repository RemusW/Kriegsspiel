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

/** Matches `SAVE_MIME` in `crates/server/src/save.rs`. */
export const SAVE_MIME = "application/vnd.kriegsspiel+zip";
/** The extension a save file is downloaded and offered under. */
export const SAVE_EXTENSION = "ksg";
