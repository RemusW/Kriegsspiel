import type { EditorTool, ToolContext } from "./ToolInput";

/**
 * Asks the server for a new unit where the table is clicked. The unit appears when the board
 * comes back, since only the server can give it an id.
 */
export class SpawnTool implements EditorTool {
  constructor(private context: ToolContext) {}

  click(event: PointerEvent) {
    const pos = this.context.pickTable(event);
    if (!pos) return;
    this.context.send({ type: "spawn", kind: "Infantry", side: "Blue", pos });
  }
}
