// The board document. The types are generated from `crates/server/src/model.rs` into
// `bindings/model.ts` by `cargo test -p kriegsspiel-server`.

import type { Board } from "./bindings/model";

export type {
  Board,
  MapImage,
  Pawn,
  Side,
  Unit,
  UnitKind,
} from "./bindings/model";

/** Match `MIN_SCALE` and `MAX_SCALE` in `crates/server/src/model.rs`. */
export const MIN_SCALE = 0.25;
export const MAX_SCALE = 4;

/**
 * An empty table: no maps, so the engine draws the blank default plane until an umpire lays
 * one down. Each map names an asset id the server serves, never a bundled file.
 */
export const EMPTY_BOARD: Board = {
  maps: [],
  pawns: [],
  nextId: 0,
};
