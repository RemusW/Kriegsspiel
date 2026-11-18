import type { ClientMessage } from "../../protocol";
import type { ClientStore, SelectMode, Tool } from "../../state/store";
import { MarqueeTool } from "./MarqueeTool";
import { SelectTool } from "./SelectTool";
import { SpawnTool } from "./SpawnTool";

/** How far the pointer may move between press and release, in CSS pixels, for a click. */
const CLICK_SLOP = 4;

/** A point in client coordinates: CSS pixels from the top left of the page's viewport. */
export interface ScreenPoint {
  x: number;
  y: number;
}

/** A rectangle in client coordinates, with `left <= right` and `top <= bottom`. */
export interface ScreenBox {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** What the engine gives its editing tools. */
export interface ToolContext {
  /** The point on the table under the pointer as [world X, world Z], on or off the map. */
  pickTable(event: PointerEvent): [number, number] | null;
  /** The unit whose top face is under the pointer, or null. */
  pickUnit(event: PointerEvent): number | null;
  /** The units whose top-face centre is inside this box on screen. */
  pickUnitsInBox(box: ScreenBox): number[];
  /** Draw the marquee over this box, or hide it with null. */
  showMarquee(box: ScreenBox | null): void;
  select(ids: number[], mode: SelectMode): void;
  /** Send a command to the server. The board it sends back shows the result. */
  send(message: ClientMessage): void;
}

/** One way of editing the board with the left mouse button. */
export interface EditorTool {
  /** A left-button press and release without moving in between. */
  click(event: PointerEvent): void;
  /** The pointer moved past the click threshold with the left button held. */
  dragStart?(start: ScreenPoint): void;
  /** Every move of the pointer during a drag, starting with the one that began it. */
  dragMove?(event: PointerEvent): void;
  dragEnd?(event: PointerEvent): void;
  /** The drag ended without a release, for example because the browser took the pointer. */
  dragCancel?(): void;
}

/** A left-button press in progress. */
interface Press {
  /** The tool chosen when the press began, which handles the whole gesture. */
  tool: EditorTool | undefined;
  pointerId: number;
  start: ScreenPoint;
  dragging: boolean;
}

/**
 * Turns left-button input on the canvas into clicks and drags for the tool chosen in the
 * store. The camera controllers use the right and middle buttons.
 */
export class ToolInput {
  private tools: Partial<Record<Tool, EditorTool>>;
  private press: Press | null = null;

  constructor(
    private element: HTMLElement,
    private store: ClientStore,
    context: ToolContext,
  ) {
    this.tools = {
      select: new SelectTool(context),
      marquee: new MarqueeTool(context),
      spawn: new SpawnTool(context),
    };
    element.addEventListener("pointerdown", this.onPointerDown);
    element.addEventListener("pointermove", this.onPointerMove);
    element.addEventListener("pointerup", this.onPointerUp);
    element.addEventListener("pointercancel", this.onPointerLost);
    element.addEventListener("lostpointercapture", this.onPointerLost);
  }

  dispose() {
    const element = this.element;
    element.removeEventListener("pointerdown", this.onPointerDown);
    element.removeEventListener("pointermove", this.onPointerMove);
    element.removeEventListener("pointerup", this.onPointerUp);
    element.removeEventListener("pointercancel", this.onPointerLost);
    element.removeEventListener("lostpointercapture", this.onPointerLost);
  }

  private onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0 || this.press) return;
    this.press = {
      tool: this.tools[this.store.getState().tool],
      pointerId: event.pointerId,
      start: { x: event.clientX, y: event.clientY },
      dragging: false,
    };
    // Keep receiving this pointer's events when it leaves the canvas, for example over the
    // toolbar, so the release is never missed.
    this.element.setPointerCapture(event.pointerId);
  };

  private onPointerMove = (event: PointerEvent) => {
    const press = this.press;
    if (!press || event.pointerId !== press.pointerId) return;
    if (!press.dragging) {
      const moved = Math.hypot(
        event.clientX - press.start.x,
        event.clientY - press.start.y,
      );
      if (moved <= CLICK_SLOP) return;
      press.dragging = true;
      press.tool?.dragStart?.(press.start);
    }
    press.tool?.dragMove?.(event);
  };

  private onPointerUp = (event: PointerEvent) => {
    const press = this.press;
    if (!press || event.pointerId !== press.pointerId || event.button !== 0) {
      return;
    }
    this.press = null;
    if (press.dragging) {
      press.tool?.dragEnd?.(event);
    } else {
      press.tool?.click(event);
    }
  };

  private onPointerLost = (event: PointerEvent) => {
    const press = this.press;
    if (!press || event.pointerId !== press.pointerId) return;
    this.press = null;
    if (press.dragging) press.tool?.dragCancel?.();
  };
}
