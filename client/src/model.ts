// The board document. Mirrors `crates/server/src/model.rs`; change both together.

export type UnitKind = "Infantry" | "Cavalry" | "Artillery";
export type Side = "Blue" | "Red";

export interface Pawn {
  /** Position on the map plane: world X, world Z. */
  pos: [number, number];
  /** Rotation around the vertical axis, in radians. */
  rot: number;
  scale: number;
  /** Asset id of a custom top-face image. `null` uses the generated symbol. */
  image: string | null;
}

export interface Unit extends Pawn {
  /** Stable across snapshots. */
  id: number;
  kind: UnitKind;
  side: Side;
}

export interface Board {
  /** Asset id of the map image. */
  mapImage: string | null;
  units: Pawn[];
  nextId: number;
}

export const EMPTY_BOARD: Board = { mapImage: null, units: [], nextId: 0 };
