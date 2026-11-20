import type { EditorContext } from "../EditorContext";
import type { PressPoint, TablePoint } from "../coords";
import type { Gesture } from "./Gesture";
import { marqueeTool } from "./marquee";
import { selectTool } from "./select";
import { spawnTool } from "./spawn";

/**
 * A press that became a drag, with what was under the pointer when the button went down.
 * A drag begins only once the pointer has moved past the click threshold, and by then the
 * scene may have changed under the press point: a held key may be turning the units, or the
 * camera moving. A tool begins its drag from this rather than picking again, so it acts on
 * what the umpire pressed on.
 */
export interface PressHit extends PressPoint {
  /** The unit under the press, or null. */
  unit: number | null;
  /** The point on the table under the press, or null above the horizon. */
  table: TablePoint | null;
}

/** One way of editing the board with the left mouse button. */
export interface Tool {
  readonly id: string;
  /** The toolbar button's tooltip and accessible name. */
  readonly label: string;
  /** A press and release without moving in between. */
  click(context: EditorContext, press: PressPoint): void;
  /** Start a drag, or return null to leave the press to the tool's other gestures. */
  begin?(context: EditorContext, press: PressHit): Gesture | null;
}

/** Every tool, in toolbar order. Adding one here is all it takes to offer it. */
export const TOOLS = [selectTool, marqueeTool, spawnTool] as const;

/** The id of one of the tools; what the store keeps as the chosen tool. */
export type ToolId = (typeof TOOLS)[number]["id"];

/** The tool with this id, or undefined if nothing answers to it. */
export function toolById(id: ToolId): Tool | undefined {
  return TOOLS.find((tool) => tool.id === id);
}
