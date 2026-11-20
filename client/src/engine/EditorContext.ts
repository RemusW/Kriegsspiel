import type { Unit } from "../model";
import type { ClientMessage } from "../protocol";
import type { SelectMode } from "../state/store";
import type { ScreenBox } from "./coords";
import type { PendingPawn } from "./PendingEdits";
import type { PickSource } from "./picking/Picker";

/** What the engine gives its editing tools. */
export interface EditorContext {
  /** Scene queries under a point on the canvas, against the units as drawn. */
  pick: PickSource;
  /**
   * The units as drawn: the board the server sent with this client's pending edits applied.
   * Tools read this and never the stored board, which lags the screen by a round trip.
   */
  units(): readonly Unit[];
  /** Ids of the selected units. */
  selection(): readonly number[];
  /** Change the selection. Ids that are not on the board are ignored. */
  select(ids: number[], mode: SelectMode): void;
  /** Draw these units' fields as already changed, for a gesture that is still running. */
  preview(edits: Map<number, PendingPawn>): void;
  /** The gesture is over: keep its edits on screen until the board answering them arrives. */
  releasePreview(): void;
  /** Stop drawing the gesture's edits, because no command was sent. */
  discardPreview(): void;
  /** Draw the marquee over this box, or hide it with null. */
  showMarquee(box: ScreenBox | null): void;
  /**
   * Send a command to the server. The board it sends back shows the result. False means the
   * command was dropped because the connection is down.
   */
  send(message: ClientMessage): boolean;
}
