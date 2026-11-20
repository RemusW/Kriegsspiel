import type { EditorContext } from "../EditorContext";
import type { PressPoint } from "../coords";
import type { Gesture } from "./Gesture";
import { marqueeTool } from "./marquee";
import { selectTool } from "./select";
import { spawnTool } from "./spawn";

/** One way of editing the board with the left mouse button. */
export interface Tool {
  readonly id: string;
  /** The toolbar button's tooltip and accessible name. */
  readonly label: string;
  /** A press and release without moving in between. */
  click(context: EditorContext, press: PressPoint): void;
  /** Start a drag, or return null to leave the press to the tool's other gestures. */
  begin?(context: EditorContext, press: PressPoint): Gesture | null;
}

/** Every tool, in toolbar order. Adding one here is all it takes to offer it. */
export const TOOLS = [selectTool, marqueeTool, spawnTool] as const;

/** The id of one of the tools; what the store keeps as the chosen tool. */
export type ToolId = (typeof TOOLS)[number]["id"];

/** The tool with this id, or undefined if nothing answers to it. */
export function toolById(id: ToolId): Tool | undefined {
  return TOOLS.find((tool) => tool.id === id);
}
