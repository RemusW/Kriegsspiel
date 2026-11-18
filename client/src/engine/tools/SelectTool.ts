import type { EditorTool, ToolContext } from "./ToolInput";

/**
 * Click selection, shared by the select and marquee tools. A click selects the unit under
 * the pointer, or clears the selection on empty table. With Shift it toggles that unit and
 * leaves the rest of the selection alone.
 */
export function selectAt(context: ToolContext, event: PointerEvent) {
  const id = context.pickUnit(event);
  if (id !== null) {
    context.select([id], event.shiftKey ? "toggle" : "replace");
  } else if (!event.shiftKey) {
    context.select([], "replace");
  }
}

export class SelectTool implements EditorTool {
  constructor(private context: ToolContext) {}

  click(event: PointerEvent) {
    selectAt(this.context, event);
  }
}
