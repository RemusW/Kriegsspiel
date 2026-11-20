import type { EditorContext } from "../EditorContext";
import type { PressPoint } from "../coords";
import { grabUnits } from "./MoveDrag";
import type { Tool } from "./Tool";

/**
 * Click selection, shared by the select and marquee tools. A click selects the unit under the
 * pointer, or clears the selection on empty table. With `extend` it toggles that unit and
 * leaves the rest of the selection alone. The modifier is the one held at the release.
 */
export function selectAt(context: EditorContext, press: PressPoint) {
  const id = context.pick.unit(press);
  if (id !== null) {
    context.select([id], press.mods.has("extend") ? "toggle" : "replace");
  } else if (!press.mods.has("extend")) {
    context.select([], "replace");
  }
}

/**
 * Click to select, drag a unit to move it. A drag from empty table does nothing, which leaves
 * box selection to the marquee tool.
 */
export const selectTool = {
  id: "select",
  label: "Select: click a unit, Shift-click to add or remove one, drag to move",
  click: selectAt,
  begin: grabUnits,
} satisfies Tool;
