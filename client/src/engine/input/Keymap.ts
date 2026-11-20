import type { ActionId } from "../actions/Action";
import { screenPointOf, type PressPoint } from "../coords";
import { CHORDS, type Chord, type ChordState, type Modifier } from "./Modifier";

/**
 * One key that asks for an action. `code` is a physical key position
 * (`KeyboardEvent.code`), so a binding means the same key on every keyboard layout, and
 * `chords` must match exactly: Ctrl+Shift+Z is not also Ctrl+Z with a stray Shift.
 */
export interface KeyBinding {
  readonly code: string;
  readonly chords: readonly Chord[];
}

/**
 * Which keys stand for what. The whole mapping is data, so a settings menu only has to write
 * one of these into the store; nothing else in the client names a key.
 */
export interface Bindings {
  /** Every key that has to be held for each modifier, in any order. */
  readonly modifiers: Readonly<Record<Modifier, readonly Chord[]>>;
  /** The keys that ask for each action. Several may, and an action may have none. */
  readonly actions: Readonly<Record<ActionId, readonly KeyBinding[]>>;
}

export const DEFAULT_BINDINGS: Bindings = {
  modifiers: {
    extend: ["shift"],
    fast: ["shift"],
  },
  actions: {
    undo: [{ code: "KeyZ", chords: ["ctrl"] }],
    redo: [{ code: "KeyZ", chords: ["ctrl", "shift"] }],
  },
};

const IS_HELD: Record<Chord, (state: ChordState) => boolean> = {
  shift: (state) => state.shiftKey,
  ctrl: (state) => state.ctrlKey,
  alt: (state) => state.altKey,
  meta: (state) => state.metaKey,
};

/** Exactly this key with exactly these modifiers, and no others held. */
function matches(binding: KeyBinding, event: KeyboardEvent): boolean {
  if (event.code !== binding.code) return false;
  return CHORDS.every(
    (chord) => IS_HELD[chord](event) === binding.chords.includes(chord),
  );
}

/**
 * Turns the keys an event carried into what they mean. The only thing in the client that knows
 * which key does what: everything else asks about a `Modifier`.
 */
export class Keymap {
  /** Replace to rebind. Nothing is cached, so the next event uses the new bindings. */
  constructor(public bindings: Bindings = DEFAULT_BINDINGS) {}

  /** True while every key bound to this modifier is held. A modifier bound to nothing is off. */
  holds(modifier: Modifier, state: ChordState): boolean {
    const chords = this.bindings.modifiers[modifier];
    return chords.length > 0 && chords.every((chord) => IS_HELD[chord](state));
  }

  /** What the keys held during this event mean, to read back after the event is gone. */
  active(state: ChordState): ReadonlySet<Modifier> {
    const on = new Set<Modifier>();
    for (const modifier of Object.keys(this.bindings.modifiers) as Modifier[]) {
      if (this.holds(modifier, state)) on.add(modifier);
    }
    return on;
  }

  /** What this key press asks for, or null when no binding matches it. */
  action(event: KeyboardEvent): ActionId | null {
    for (const id of Object.keys(this.bindings.actions) as ActionId[]) {
      for (const binding of this.bindings.actions[id]) {
        if (matches(binding, event)) return id;
      }
    }
    return null;
  }

  /**
   * Where a pointer event happened and what it meant. Resolved here, at the event, because a
   * key held at the press may be gone by the release.
   */
  press(event: PointerEvent): PressPoint {
    return { ...screenPointOf(event), mods: this.active(event) };
  }
}
