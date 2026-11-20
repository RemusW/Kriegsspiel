import type { Keymap } from "../input/Keymap";
import {
  chordsOf,
  NO_CHORDS,
  type ChordState,
  type Modifier,
} from "../input/Modifier";

/** Physical key codes, so the same key positions work on every keyboard layout. */
export const MOVE_KEYS = {
  forward: "KeyW",
  back: "KeyS",
  left: "KeyA",
  right: "KeyD",
  up: "KeyR",
  down: "KeyF",
} as const;

/** Speed multiplier while the `fast` modifier is held. */
export const FAST_MULTIPLIER = 3;

/**
 * The keys held down in the window, for camera movement. Keys pressed while typing in a text
 * field, or together with Ctrl, Cmd or Alt, are ignored, so shortcuts such as Ctrl+W never
 * move the camera.
 */
export class KeyState {
  private pressed = new Set<string>();
  /** The modifier keys held, as of the last key event the window saw. */
  private chords: ChordState = NO_CHORDS;

  /** `keymap` says which keys a modifier stands for, so this class never decides. */
  constructor(private keymap: Keymap) {
    window.addEventListener("keydown", this.onKeyDown);
    window.addEventListener("keyup", this.onKeyUp);
    window.addEventListener("blur", this.onBlur);
  }

  /** -1 while only `negative` is held, 1 while only `positive` is held, otherwise 0. */
  axis(negative: string, positive: string): number {
    return (
      (this.pressed.has(positive) ? 1 : 0) -
      (this.pressed.has(negative) ? 1 : 0)
    );
  }

  /** True while the keys bound to this modifier are held. */
  holds(modifier: Modifier): boolean {
    return this.keymap.holds(modifier, this.chords);
  }

  dispose() {
    window.removeEventListener("keydown", this.onKeyDown);
    window.removeEventListener("keyup", this.onKeyUp);
    window.removeEventListener("blur", this.onBlur);
  }

  private onKeyDown = (event: KeyboardEvent) => {
    // Before the filters below, which drop the movement keys but not the modifiers: a
    // modifier's own keydown carries its flag, so Ctrl would otherwise never be seen.
    this.chords = chordsOf(event);
    // macOS sends no keyup for keys released while Cmd is held, so forget everything on Cmd.
    if (event.key === "Meta") {
      this.pressed.clear();
      return;
    }
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (isTextInput(event.target)) return;
    this.pressed.add(event.code);
  };

  private onKeyUp = (event: KeyboardEvent) => {
    this.chords = chordsOf(event);
    this.pressed.delete(event.code);
  };

  private onBlur = () => {
    this.pressed.clear();
    this.chords = NO_CHORDS;
  };
}

function isTextInput(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}
