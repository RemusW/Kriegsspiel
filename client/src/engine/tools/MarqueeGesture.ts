import type { EditorContext } from "../EditorContext";
import { boxBetween, type PressPoint, type ScreenPoint } from "../coords";
import type { Gesture } from "./Gesture";

/**
 * A box drawn across the screen that selects the units inside it on release. With `extend`
 * held at the release the units are added to the selection instead of replacing it: at the
 * release rather than the press, so the choice can be made while the box is being drawn.
 */
export class MarqueeGesture implements Gesture {
  constructor(
    private context: EditorContext,
    private start: ScreenPoint,
  ) {}

  move(point: ScreenPoint) {
    this.context.showMarquee(boxBetween(this.start, point));
  }

  end(point: PressPoint) {
    const box = boxBetween(this.start, point);
    this.context.showMarquee(null);
    this.context.select(
      this.context.pick.unitsInBox(box),
      point.mods.has("extend") ? "add" : "replace",
    );
  }

  cancel() {
    this.context.showMarquee(null);
  }
}
