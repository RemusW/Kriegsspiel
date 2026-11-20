import { actionById, type KeyHold } from "../actions/Action";
import type { EditorContext } from "../EditorContext";
import type { Keymap } from "./Keymap";
import { isTextInput } from "./textInput";

/**
 * The window's one owner of keys that ask for something. `KeyState` is the other keyboard
 * reader and tracks keys that are *held* for the camera; this one acts on bound keys, so a
 * binding cannot be claimed twice.
 *
 * A press action runs on every keydown, and a held key repeats, which is how holding undo
 * keeps undoing. A hold action starts on the first keydown, ignores the repeats, and runs
 * every frame until the key comes up. Several can be held at once, such as turning while
 * resizing; they join the same manipulation, which is one edit.
 */
export class KeyInput {
  /** Hold actions whose keys are down, by physical key, to recognise each one's release. */
  private held = new Map<string, KeyHold>();

  constructor(
    private keymap: Keymap,
    private context: EditorContext,
  ) {
    window.addEventListener("keydown", this.onKeyDown);
    window.addEventListener("keyup", this.onKeyUp);
    window.addEventListener("blur", this.onBlur);
  }

  /** Advance the held keys. The engine calls this once a frame. */
  update(dt: number) {
    for (const hold of this.held.values()) hold.update(dt);
  }

  dispose() {
    window.removeEventListener("keydown", this.onKeyDown);
    window.removeEventListener("keyup", this.onKeyUp);
    window.removeEventListener("blur", this.onBlur);
  }

  private onKeyDown = (event: KeyboardEvent) => {
    // While typing, the keys belong to the field, Ctrl+Z included.
    if (isTextInput(event.target)) return;
    const id = this.keymap.action(event);
    if (!id) return;
    const action = actionById(id);
    if (!action) return;
    // The browser has its own Ctrl+Z; ours is the one that runs.
    event.preventDefault();
    if ("run" in action) {
      action.run(this.context);
      return;
    }
    if (event.repeat || this.held.has(event.code)) return;
    const hold = action.hold(this.context);
    if (hold) this.held.set(event.code, hold);
  };

  /** Matched by key alone, whatever modifiers are held by the time it comes up. */
  private onKeyUp = (event: KeyboardEvent) => {
    const hold = this.held.get(event.code);
    if (!hold) return;
    this.held.delete(event.code);
    hold.end();
  };

  /**
   * No key's release will arrive once the window loses focus. What the keys did is already on
   * screen, so it is kept rather than thrown away.
   */
  private onBlur = () => {
    const holds = [...this.held.values()];
    this.held.clear();
    for (const hold of holds) hold.end();
  };
}
