import type { EditorContext } from "../EditorContext";
import { redoAction, undoAction } from "./history";

/** One thing a key can ask for, independent of the key that asks. */
export interface Action {
  readonly id: string;
  /** What a keybinding menu calls it. */
  readonly label: string;
  run(context: EditorContext): void;
}

/** Every action a binding may name. Adding one here is all it takes to make it bindable. */
export const ACTIONS = [undoAction, redoAction] as const;

/** The id of one of the actions; what `Bindings` maps keys to. */
export type ActionId = (typeof ACTIONS)[number]["id"];

/** The action with this id, or undefined if nothing answers to it. */
export function actionById(id: ActionId): Action | undefined {
  return ACTIONS.find((action) => action.id === id);
}
