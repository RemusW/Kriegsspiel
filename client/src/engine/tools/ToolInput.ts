import type { ClientStore } from "../../state/store";
import { screenPointOf, type PressPoint } from "../coords";
import type { EditorContext } from "../EditorContext";
import type { Keymap } from "../input/Keymap";
import type { Gesture } from "./Gesture";
import { toolById, type Tool } from "./Tool";

/** How far the pointer may move between press and release, in CSS pixels, for a click. */
const CLICK_SLOP = 4;

/** A left-button press in progress. */
interface Press {
  /** The tool chosen when the press began, which handles the whole gesture. */
  tool: Tool | undefined;
  pointerId: number;
  start: PressPoint;
  /** Set once the pointer passed the click threshold, whether or not the tool took the drag. */
  dragging: boolean;
  /** The drag the tool began, if it took it. */
  gesture: Gesture | null;
}

/**
 * Turns left-button input on the canvas into clicks and drags for the tool chosen in the
 * store. The camera controllers use the right and middle buttons.
 */
export class ToolInput {
  private press: Press | null = null;

  constructor(
    private element: HTMLElement,
    private store: ClientStore,
    /** Says what the keys held with a press mean, so no tool has to name one. */
    private keymap: Keymap,
    private context: EditorContext,
  ) {
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
    // The tool is resolved here and kept for the whole gesture, so switching tools mid-drag
    // leaves the drag with the tool that began it.
    this.press = {
      tool: toolById(this.store.getState().tool),
      pointerId: event.pointerId,
      start: this.keymap.press(event),
      dragging: false,
      gesture: null,
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
      // Asked once per press: a tool that declines the drag is not asked again as the pointer
      // moves on, so nothing can start a gesture from a point that was never pressed.
      press.dragging = true;
      press.gesture = press.tool?.begin?.(this.context, press.start) ?? null;
    }
    press.gesture?.move(screenPointOf(event));
  };

  private onPointerUp = (event: PointerEvent) => {
    const press = this.press;
    if (!press || event.pointerId !== press.pointerId || event.button !== 0) {
      return;
    }
    this.press = null;
    if (press.dragging) {
      press.gesture?.end(this.keymap.press(event));
    } else {
      press.tool?.click(this.context, this.keymap.press(event));
    }
  };

  private onPointerLost = (event: PointerEvent) => {
    const press = this.press;
    if (!press || event.pointerId !== press.pointerId) return;
    this.press = null;
    press.gesture?.cancel();
  };
}
