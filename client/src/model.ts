// The board document. The types are generated from `crates/server/src/model.rs` into
// `bindings/model.ts` by `cargo test -p kriegsspiel-server`.

import farleyURL from "../assets/farley.png";
import type { Board } from "./bindings/model";

export type { Board, Pawn, Side, Unit, UnitKind } from "./bindings/model";

export const EMPTY_BOARD: Board = {
  mapImage: farleyURL,
  pawns: [],
  nextId: 0,
};
