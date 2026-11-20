import type { EditorContext } from "../EditorContext";
import type { PressPoint, ScreenPoint, TablePoint } from "../coords";
import type { PendingPawn } from "../PendingEdits";
import type { UnitMove } from "../../protocol";
import { IGNORED, type Gesture } from "./Gesture";

/**
 * Begin moving units because a drag started on one, or return null when it started on empty
 * table, which leaves the gesture to the tool.
 *
 * Dragging a selected unit moves the whole selection. Dragging any other unit selects it
 * first, with `extend` adding it to the selection instead of replacing it, so the units about
 * to move are always the outlined ones. The modifier is the one held at the press, since the
 * key may be released during the drag.
 */
export function grabUnits(
  context: EditorContext,
  start: PressPoint,
): Gesture | null {
  const id = context.pick.unit(start);
  if (id === null) return null;
  if (!context.selection().includes(id)) {
    context.select([id], start.mods.has("extend") ? "add" : "replace");
  }

  // Where on the table the drag began, so the grabbed point stays under the pointer.
  const origin = context.pick.table(start);
  const selected = new Set(context.selection());
  const starts: UnitMove[] = context
    .units()
    .filter((unit) => selected.has(unit.id))
    .map((unit) => ({ id: unit.id, pos: unit.pos }));
  // The press was on a unit, so this tool owns the drag either way: with no table point under
  // it there is nothing to move along, but it must not turn into a selection box.
  if (!origin || starts.length === 0) return IGNORED;
  return new MoveDrag(context, origin, starts);
}

/**
 * Units following the pointer across the table. They are drawn at the offset as it changes;
 * the board changes only when the server answers the command sent on release.
 */
export class MoveDrag implements Gesture {
  /** How far the pointer has carried the units, as [world dX, world dZ]. */
  private delta: TablePoint = [0, 0];

  constructor(
    private context: EditorContext,
    /** The point on the table the drag began at. */
    private origin: TablePoint,
    /** Where the units were drawn when they were grabbed, which their new positions add to. */
    private starts: UnitMove[],
  ) {}

  move(point: ScreenPoint) {
    const at = this.context.pick.table(point);
    // Above the horizon or beyond the far plane: keep the offset rather than jump.
    if (at) this.delta = [at[0] - this.origin[0], at[1] - this.origin[1]];
    this.context.preview(previewOf(this.moves()));
  }

  /** Ask the server to put the units where they are drawn. */
  end() {
    const [dx, dz] = this.delta;
    if (dx === 0 && dz === 0) {
      this.cancel();
      return;
    }
    if (this.context.send({ type: "move", moves: this.moves() })) {
      this.context.releasePreview();
    } else {
      this.cancel();
    }
  }

  cancel() {
    this.context.discardPreview();
  }

  /** Where the units go: where they were grabbed, plus the pointer's offset. */
  private moves(): UnitMove[] {
    const [dx, dz] = this.delta;
    return this.starts.map(({ id, pos }) => ({
      id,
      pos: [pos[0] + dx, pos[1] + dz],
    }));
  }
}

/** The same positions, as the edits the engine draws until the server answers. */
function previewOf(moves: UnitMove[]): Map<number, PendingPawn> {
  return new Map(moves.map(({ id, pos }) => [id, { pos }]));
}
