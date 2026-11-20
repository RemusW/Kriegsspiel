import type { EditorContext } from "../EditorContext";
import type { PressPoint } from "../coords";
import type { Tool } from "./Tool";

/**
 * Asks the server for a new unit where the table is clicked. The unit appears when the board
 * comes back, since only the server can give it an id.
 */
export const spawnTool = {
  id: "spawn",
  label: "Spawn: click the map to place an infantry unit",
  click: (context: EditorContext, press: PressPoint) => {
    if (context.readOnly()) return;
    const pos = context.pick.table(press);
    if (!pos) return;
    context.send({ type: "spawn", kind: "Infantry", side: "Blue", pos });
  },
} satisfies Tool;
