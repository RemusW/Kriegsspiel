import type { Unit } from "../model";
import type { Placement } from "../protocol";
import type { TablePoint } from "./coords";

/** The fields of one unit this client is showing as already changed. */
export interface PendingPawn {
  pos?: TablePoint;
  rot?: number;
  scale?: number;
}

/**
 * The placements a command asks for, as edits to draw until the server answers. Only the
 * fields a placement carries are copied, so the rest of each unit is drawn as it is.
 */
export function previewOf(
  units: readonly Placement[],
): Map<number, PendingPawn> {
  return new Map(
    units.map(({ id, pos, rot, scale }) => {
      const edit: PendingPawn = {};
      if (pos !== undefined) edit.pos = pos;
      if (rot !== undefined) edit.rot = rot;
      if (scale !== undefined) edit.scale = scale;
      return [id, edit];
    }),
  );
}

/**
 * The difference between the board the server sent and what is on screen: the edits this
 * client has asked for and not yet seen in a board.
 *
 * Only the server moves units, so an edit is shown from here until the board that answers it
 * arrives, which keeps a drag from dropping back for the length of a round trip. Edits a
 * gesture is still holding survive that board, so a change made by another umpire cannot
 * clear a drag that is still under way.
 */
export class PendingEdits {
  private edits = new Map<number, PendingPawn>();
  /** Ids the gesture in progress is holding. */
  private held = new Set<number>();

  /** Show these edits for a gesture that is still running. Each carries only what it changes. */
  hold(edits: Map<number, PendingPawn>) {
    for (const [id, edit] of edits) {
      this.edits.set(id, edit);
    }
    this.held = new Set(edits.keys());
  }

  /** The gesture is over: its edits stand until the next board arrives. */
  release() {
    this.held.clear();
  }

  /** Stop showing the gesture's edits now, because no command was sent. */
  discard() {
    for (const id of this.held) {
      this.edits.delete(id);
    }
    this.held.clear();
  }

  /** A board arrived. It answers every edit no gesture is holding. */
  settle() {
    for (const id of [...this.edits.keys()]) {
      if (!this.held.has(id)) this.edits.delete(id);
    }
  }

  /** These units as they are drawn. The same array comes back when nothing is pending. */
  applyTo(units: readonly Unit[]): readonly Unit[] {
    if (this.edits.size === 0) return units;
    return units.map((unit) => {
      const edit = this.edits.get(unit.id);
      return edit ? { ...unit, ...edit } : unit;
    });
  }
}
