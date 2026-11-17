import type { ClientMessage } from "../../protocol";
import type { ClientStore, Tool } from "../../state/store";
import { SelectTool } from "./SelectTool";
import { SpawnTool } from "./SpawnTool";

/** How far the pointer may move between press and release, in CSS pixels, for a click. */
const CLICK_SLOP = 4;

/** What the engine gives its editing tools. */
export interface ToolContext {
  /** The point on the table under the pointer as [world X, world Z], on or off the map. */
  pickTable(event: PointerEvent): [number, number] | null;
  /** The unit whose top face is under the pointer, or null. */
  pickUnit(event: PointerEvent): number | null;
  /** Replace the selection with these unit ids. */
  select(ids: number[]): void;
  /** Send a command to the server. The board it sends back shows the result. */
  send(message: ClientMessage): void;
}

/** One way of editing the board with the left mouse button. */
export interface EditorTool {
  /** A left-button press and release without moving in between. */
  click(event: PointerEvent): void;
}

/**
 * Passes left-button input on the canvas to the tool chosen in the store. The camera
 * controllers use the right and middle buttons.
 */
export class ToolInput {
  private tools: Partial<Record<Tool, EditorTool>>;
  private pressedAt: { x: number; y: number } | null = null;

  constructor(
    private element: HTMLElement,
    private store: ClientStore,
    context: ToolContext,
  ) {
    this.tools = {
      select: new SelectTool(context),
      spawn: new SpawnTool(context),
    };
    element.addEventListener("pointerdown", this.onPointerDown);
    element.addEventListener("pointerup", this.onPointerUp);
  }

  dispose() {
    this.element.removeEventListener("pointerdown", this.onPointerDown);
    this.element.removeEventListener("pointerup", this.onPointerUp);
  }

  private onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0) return;
    this.pressedAt = { x: event.clientX, y: event.clientY };
  };

  private onPointerUp = (event: PointerEvent) => {
    if (event.button !== 0 || !this.pressedAt) return;
    const moved = Math.hypot(
      event.clientX - this.pressedAt.x,
      event.clientY - this.pressedAt.y,
    );
    this.pressedAt = null;
    if (moved > CLICK_SLOP) return;
    this.tools[this.store.getState().tool]?.click(event);
  };
}
