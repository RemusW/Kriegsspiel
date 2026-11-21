import type { Modifier } from "./input/Modifier";

/** A point in client coordinates: CSS pixels from the top left of the page's viewport. */
export interface ScreenPoint {
  x: number;
  y: number;
}

/**
 * Where a pointer event happened and what was held with it, by what the keys mean rather than
 * which keys they are. `Keymap.press` makes these; tools read `mods` and never a key.
 */
export interface PressPoint extends ScreenPoint {
  mods: ReadonlySet<Modifier>;
}

/** A rectangle in client coordinates, with `left <= right` and `top <= bottom`. */
export interface ScreenBox {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** A point on the map plane, as [world X, world Z]. */
export type TablePoint = [number, number];

/**
 * A rectangle of the table in world units: where it is and how far it reaches. What the
 * maps cover, which is what the camera is framed on.
 */
export interface TableArea {
  centre: TablePoint;
  /** Along world X. */
  width: number;
  /** Along world Z. */
  depth: number;
}

/** The client-coordinate point a pointer event happened at. */
export function screenPointOf(event: PointerEvent): ScreenPoint {
  return { x: event.clientX, y: event.clientY };
}

/** The box between two points, whichever corners they are. */
export function boxBetween(a: ScreenPoint, b: ScreenPoint): ScreenBox {
  return {
    left: Math.min(a.x, b.x),
    top: Math.min(a.y, b.y),
    right: Math.max(a.x, b.x),
    bottom: Math.max(a.y, b.y),
  };
}
