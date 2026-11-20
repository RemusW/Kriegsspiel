import { screenPointOf, type PressPoint } from "../coords";
import type { Chord, ChordState, Modifier } from "./Modifier";

/**
 * Which keys stand for what. The whole mapping is data, so a settings menu only has to write
 * one of these into the store; nothing else in the client names a key.
 */
export interface Bindings {
  /** Every key that has to be held for each modifier, in any order. */
  readonly modifiers: Readonly<Record<Modifier, readonly Chord[]>>;
}

export const DEFAULT_BINDINGS: Bindings = {
  modifiers: {
    extend: ["shift"],
    fast: ["shift"],
  },
};

const IS_HELD: Record<Chord, (state: ChordState) => boolean> = {
  shift: (state) => state.shiftKey,
  ctrl: (state) => state.ctrlKey,
  alt: (state) => state.altKey,
  meta: (state) => state.metaKey,
};

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

  /**
   * Where a pointer event happened and what it meant. Resolved here, at the event, because a
   * key held at the press may be gone by the release.
   */
  press(event: PointerEvent): PressPoint {
    return { ...screenPointOf(event), mods: this.active(event) };
  }
}
