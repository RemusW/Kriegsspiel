import type { EditorContext } from "../EditorContext";
import type { HoldAction, KeyHold } from "./Action";

/** How long a key must stay down before it starts acting continuously, in seconds. */
const HOLD_DELAY = 0.25;
/** A press turns the selection this far, in radians; holding turns it at TURN_RATE. */
const TURN_STEP = (5 * Math.PI) / 180;
const TURN_RATE = Math.PI / 2;
/** A press multiplies sizes by e^GROW_STEP; holding multiplies them by e^GROW_RATE a second. */
const GROW_STEP = Math.log(1.05);
const GROW_RATE = Math.log(1.5);

/**
 * How far a key has acted after being held this many seconds: one step on the press, then
 * nothing until HOLD_DELAY, then `perSecond`. The pause lets a tap make exactly one step.
 */
function ramp(held: number, step: number, perSecond: number): number {
  return step + Math.max(0, held - HOLD_DELAY) * perSecond;
}

/**
 * A hold action that takes part in the selection's manipulation, joining the drag or other
 * held keys if there are any. `amount` says how far it has acted after `held` seconds.
 */
function manipulateAction<Id extends string>(
  id: Id,
  label: string,
  field: "angle" | "logScale",
  amount: (held: number) => number,
) {
  return {
    id,
    label,
    hold: (context: EditorContext): KeyHold | null => {
      const manipulation = context.manipulate();
      if (!manipulation) return null;
      const input = manipulation.join();
      let held = 0;
      input.set({ [field]: amount(held) });
      return {
        update(dt) {
          held += dt;
          input.set({ [field]: amount(held) });
        },
        end: () => input.release(),
      };
    },
  } satisfies HoldAction;
}

/** Wheel the selection about its centroid, the way a formation turns. */
export const turnLeftAction = manipulateAction(
  "turnLeft",
  "Turn selection counterclockwise",
  "angle",
  (held) => ramp(held, TURN_STEP, TURN_RATE),
);
export const turnRightAction = manipulateAction(
  "turnRight",
  "Turn selection clockwise",
  "angle",
  (held) => -ramp(held, TURN_STEP, TURN_RATE),
);
/** Grow or shrink each unit's footprint in place. */
export const growAction = manipulateAction(
  "grow",
  "Enlarge selection",
  "logScale",
  (held) => ramp(held, GROW_STEP, GROW_RATE),
);
export const shrinkAction = manipulateAction(
  "shrink",
  "Shrink selection",
  "logScale",
  (held) => -ramp(held, GROW_STEP, GROW_RATE),
);
