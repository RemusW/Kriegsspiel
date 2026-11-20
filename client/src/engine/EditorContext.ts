import type { Unit } from "../model";
import type { ClientMessage } from "../protocol";
import type { SelectMode } from "../state/store";
import type { ScreenBox } from "./coords";
import type { Manipulation } from "./Manipulation";
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
  /** The manipulation in progress, if any input is still moving, turning or resizing units. */
  manipulation(): Manipulation | null;
  /**
   * The manipulation in progress, or a new one over the selected units as drawn. Null when
   * nothing is in progress and nothing is selected.
   */
  manipulate(): Manipulation | null;
  /** Draw the marquee over this box, or hide it with null. */
  showMarquee(box: ScreenBox | null): void;
  /**
   * Send a command to the server. The board it sends back shows the result. False means the
   * command was dropped because the connection is down.
   */
  send(message: ClientMessage): boolean;
}
