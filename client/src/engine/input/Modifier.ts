/**
 * What a held key means to a tool. Tools and the camera ask for these and never for a key, so
 * the keys behind them can be rebound without touching anything that reads them.
 */
export type Modifier =
  /** Add to the selection instead of replacing it. */
  | "extend"
  /** Move the camera faster. */
  | "fast";

/** A modifier key on the keyboard. Only `Keymap` turns these into modifiers. */
export type Chord = "shift" | "ctrl" | "alt" | "meta";

/**
 * Which modifier keys were held. Pointer and keyboard events both have these fields, so an
 * event is one of these without converting it.
 */
export interface ChordState {
  readonly shiftKey: boolean;
  readonly ctrlKey: boolean;
  readonly altKey: boolean;
  readonly metaKey: boolean;
}

/** Every chord, to compare a binding against the keys actually held. */
export const CHORDS: readonly Chord[] = ["shift", "ctrl", "alt", "meta"];

/** No modifier key held, for before the first event and after the window loses focus. */
export const NO_CHORDS: ChordState = {
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
};

/** The modifier keys this event carried. */
export function chordsOf(event: ChordState): ChordState {
  return {
    shiftKey: event.shiftKey,
    ctrlKey: event.ctrlKey,
    altKey: event.altKey,
    metaKey: event.metaKey,
  };
}
