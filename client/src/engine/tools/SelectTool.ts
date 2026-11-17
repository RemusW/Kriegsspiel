import type { EditorTool, ToolContext } from "./ToolInput";

/** Selects the clicked unit, or clears the selection when the click hits no unit. */
export class SelectTool implements EditorTool {
  constructor(private context: ToolContext) {}

  click(event: PointerEvent) {
    const id = this.context.pickUnit(event);
    this.context.select(id === null ? [] : [id]);
  }
}
