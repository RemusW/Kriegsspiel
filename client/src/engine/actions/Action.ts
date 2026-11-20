import type { EditorContext } from "../EditorContext";
import { redoAction, undoAction } from "./history";
import {
  growAction,
  shrinkAction,
  turnLeftAction,
  turnRightAction,
} from "./placement";

/** One thing a key asks for when it is pressed. A held key repeats it. */
export interface PressAction {
  readonly id: string;
  /** What a keybinding menu calls it. */
  readonly label: string;
  run(context: EditorContext): void;
}

/**
 * Something a key does for as long as it is held, such as turning the selection. It is one
 * edit however long the key is held, the keyboard's version of a drag.
 */
export interface HoldAction {
  readonly id: string;
  readonly label: string;
  /** Start on the key's first press, or return null when there is nothing to act on. */
  hold(context: EditorContext): KeyHold | null;
}

/** One key held down. `KeyInput` drives it until the key comes up, then drops it. */
export interface KeyHold {
  /** Called every frame while the key is down, with the seconds since the last frame. */
  update(dt: number): void;
  /** The key came up, or the window lost focus with it down. */
  end(): void;
}

export type Action = PressAction | HoldAction;

/** Every action a binding may name. Adding one here is all it takes to make it bindable. */
export const ACTIONS = [
  undoAction,
  redoAction,
  turnLeftAction,
  turnRightAction,
  growAction,
  shrinkAction,
] as const;

/** The id of one of the actions; what `Bindings` maps keys to. */
export type ActionId = (typeof ACTIONS)[number]["id"];

/** The action with this id, or undefined if nothing answers to it. */
export function actionById(id: ActionId): Action | undefined {
  return ACTIONS.find((action) => action.id === id);
}
