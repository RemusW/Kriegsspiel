import type { Action } from "./Action";

/**
 * Undo and redo. Each umpire has their own history on the server, so these take back only
 * what this client did, and the board that comes back is the result. Nothing is previewed:
 * unlike a drag, there is no local guess at what undoing will produce.
 */
export const undoAction = {
  id: "undo",
  label: "Undo",
  run: (context) => {
    context.send({ type: "undo" });
  },
} satisfies Action;

export const redoAction = {
  id: "redo",
  label: "Redo",
  run: (context) => {
    context.send({ type: "redo" });
  },
} satisfies Action;
