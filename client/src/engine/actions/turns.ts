import type { PressAction } from "./Action";

/**
 * The turn history. Recording and stepping are the server's to decide: which turn an umpire
 * is looking at is kept per umpire there, so these only ask, and the snapshot that comes back
 * says what is on screen. The same commands are sent by the timeline bar.
 */
export const recordTurnAction = {
  id: "recordTurn",
  label: "Record the board as a turn",
  run: (context) => {
    // The live board is what would be recorded, so there is nothing to record from the past.
    if (context.readOnly()) return;
    context.send({ type: "recordTurn" });
  },
} satisfies PressAction;

export const previousTurnAction = {
  id: "previousTurn",
  label: "Step back through recorded turns",
  run: (context) => {
    context.send({ type: "stepTurn", forward: false });
  },
} satisfies PressAction;

/** Stepping forward past the last recorded turn returns to the live board. */
export const nextTurnAction = {
  id: "nextTurn",
  label: "Step forward through recorded turns",
  run: (context) => {
    context.send({ type: "stepTurn", forward: true });
  },
} satisfies PressAction;

export const restoreTurnAction = {
  id: "restoreTurn",
  label: "Restore the turn being viewed onto the board",
  run: (context) => {
    context.send({ type: "restoreTurn" });
  },
} satisfies PressAction;
