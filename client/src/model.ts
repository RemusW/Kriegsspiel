// The board document. The types are generated from `crates/server/src/model.rs` into
// `bindings/` by `cargo test -p kriegsspiel-server`.

import farleyURL from "../assets/farley.png";
import type { Board } from "./bindings/Board";

export type { Board } from "./bindings/Board";
export type { Pawn } from "./bindings/Pawn";
export type { Side } from "./bindings/Side";
export type { Unit } from "./bindings/Unit";
export type { UnitKind } from "./bindings/UnitKind";

export const EMPTY_BOARD: Board = {
  mapImage: farleyURL,
  pawns: [],
  nextId: 0,
};
