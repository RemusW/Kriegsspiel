import { actionById, type Action } from "../actions/Action";
import type { EditorContext } from "../EditorContext";
import type { Keymap } from "./Keymap";
import { isTextInput } from "./textInput";

/**
 * The window's one owner of keys that ask for something. `KeyState` is the other keyboard
 * reader and tracks keys that are *held* for the camera; this one acts on the press and
 * nothing else, so a binding cannot be claimed twice.
 *
 * A held key repeats, which is how holding undo keeps undoing.
 */
export class KeyInput {
  constructor(
    private keymap: Keymap,
    private context: EditorContext,
  ) {
    window.addEventListener("keydown", this.onKeyDown);
  }

  dispose() {
    window.removeEventListener("keydown", this.onKeyDown);
  }

  private onKeyDown = (event: KeyboardEvent) => {
    // While typing, the keys belong to the field, Ctrl+Z included.
    if (isTextInput(event.target)) return;
    const id = this.keymap.action(event);
    if (!id) return;
    const action: Action | undefined = actionById(id);
    if (!action) return;
    // The browser has its own Ctrl+Z; ours is the one that runs.
    event.preventDefault();
    action.run(this.context);
  };
}
