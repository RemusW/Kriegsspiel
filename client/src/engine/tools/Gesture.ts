import type { PressPoint, ScreenPoint } from "../coords";

/**
 * One drag in progress. A tool makes it when the drag begins, `ToolInput` drives it to the
 * end, and nothing keeps it afterwards, so no gesture can leave state behind for the next.
 */
export interface Gesture {
  /** Every move of the pointer during the drag, starting with the one that began it. */
  move(point: ScreenPoint): void;
  /** The release. Its modifiers are the ones held then, not the ones held at the press. */
  end(point: PressPoint): void;
  /** The drag ended without a release, for example because the browser took the pointer. */
  cancel(): void;
}

/** A gesture that does nothing, for a press a tool claims but cannot act on. */
export const IGNORED: Gesture = {
  move() {},
  end() {},
  cancel() {},
};
