import { MarqueeGesture } from "./MarqueeGesture";
import { grabUnits } from "./MoveDrag";
import { selectAt } from "./select";
import type { Tool } from "./Tool";

/**
 * Box selection. A drag on empty table draws a box and selects what it encloses; a drag that
 * starts on a unit moves it, as with the select tool. A click selects.
 */
export const marqueeTool = {
  id: "marquee",
  label:
    "Marquee: drag a box to select units, Shift to add them; drag a unit to move it",
  click: selectAt,
  // Reads as the order the press is offered in: the units first, then the box.
  begin: (context, press) =>
    grabUnits(context, press) ?? new MarqueeGesture(context, press),
} satisfies Tool;
