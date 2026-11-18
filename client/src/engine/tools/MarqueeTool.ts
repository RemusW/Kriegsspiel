import { selectAt } from "./SelectTool";
import type {
  EditorTool,
  ScreenBox,
  ScreenPoint,
  ToolContext,
} from "./ToolInput";

/**
 * Box selection: a drag draws a box and, on release, selects the units inside it. With
 * Shift held at release, they are added to the selection instead. A click selects like the
 * select tool.
 */
export class MarqueeTool implements EditorTool {
  private start: ScreenPoint | null = null;

  constructor(private context: ToolContext) {}

  click(event: PointerEvent) {
    selectAt(this.context, event);
  }

  dragStart(start: ScreenPoint) {
    this.start = start;
  }

  dragMove(event: PointerEvent) {
    const box = this.boxTo(event);
    if (box) this.context.showMarquee(box);
  }

  dragEnd(event: PointerEvent) {
    const box = this.boxTo(event);
    this.finish();
    if (!box) return;
    this.context.select(
      this.context.pickUnitsInBox(box),
      event.shiftKey ? "add" : "replace",
    );
  }

  dragCancel() {
    this.finish();
  }

  private finish() {
    this.start = null;
    this.context.showMarquee(null);
  }

  private boxTo(event: PointerEvent): ScreenBox | null {
    const start = this.start;
    if (!start) return null;
    return {
      left: Math.min(start.x, event.clientX),
      top: Math.min(start.y, event.clientY),
      right: Math.max(start.x, event.clientX),
      bottom: Math.max(start.y, event.clientY),
    };
  }
}
