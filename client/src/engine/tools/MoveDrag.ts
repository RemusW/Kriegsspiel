import type { EditorContext } from "../EditorContext";
import type { ScreenPoint, TablePoint } from "../coords";
import type { ManipulationInput } from "../Manipulation";
import { IGNORED, type Gesture } from "./Gesture";
import type { PressHit } from "./Tool";

/**
 * Begin moving units because a drag started on one, or return null when it started on empty
 * table, which leaves the gesture to the tool.
 *
 * Dragging a selected unit moves the whole selection. Dragging any other unit selects it
 * first, with `extend` adding it to the selection instead of replacing it, so the units about
 * to move are always the outlined ones. The modifier is the one held at the press, since the
 * key may be released during the drag.
 *
 * While a held key is already turning or resizing units, the drag joins that manipulation if
 * it grabbed one of them, and does nothing otherwise: the units being changed are fixed for
 * as long as any input is still changing them.
 */
export function grabUnits(
  context: EditorContext,
  start: PressHit,
): Gesture | null {
  // Both from the moment of the press, not of the drag: a formation turning under a held key
  // could otherwise carry the pressed unit out from under the pointer before the drag began.
  const id = start.unit;
  if (id === null) return null;
  // Where on the table the drag began, so the grabbed point stays under the pointer. The press
  // was on a unit, so this tool owns the drag either way: with no table point under it there
  // is nothing to move along, but it must not turn into a selection box.
  const origin = start.table;
  if (!origin) return IGNORED;

  const active = context.manipulation();
  if (active) {
    return active.has(id)
      ? new MoveDrag(context, origin, active.join(true))
      : IGNORED;
  }
  if (!context.selection().includes(id)) {
    context.select([id], start.mods.has("extend") ? "add" : "replace");
  }
  const manipulation = context.manipulate();
  return manipulation
    ? new MoveDrag(context, origin, manipulation.join(true))
    : IGNORED;
}

/**
 * Units following the pointer across the table, as one input to their manipulation. The
 * board changes only when the server answers the command sent once every input has let go.
 */
export class MoveDrag implements Gesture {
  constructor(
    private context: EditorContext,
    /** The point on the table the drag began at. */
    private origin: TablePoint,
    private input: ManipulationInput,
  ) {}

  move(point: ScreenPoint) {
    const at = this.context.pick.table(point);
    // Above the horizon or beyond the far plane: keep the offset rather than jump.
    if (at) {
      this.input.set({
        delta: [at[0] - this.origin[0], at[1] - this.origin[1]],
      });
    }
  }

  end() {
    this.input.release();
  }

  cancel() {
    this.input.cancel();
  }
}
