import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";
import { DEFAULT_BINDINGS, type Bindings } from "../engine/input/Keymap";
import type { ToolId } from "../engine/tools/Tool";
import { EMPTY_BOARD, type Board } from "../model";
import type { TurnInfo } from "../protocol";
export type MarqueeShape = "box" | "lasso";
export type Connection = "connecting" | "connected" | "disconnected";
export type CameraMode = "fly" | "map";
/** How `select` combines ids with the current selection. */
export type SelectMode = "replace" | "add" | "toggle";

/**
 * What the server's latest snapshot says this client is looking at: a board, and where it
 * came from. The three travel together so the timeline can never label the wrong board.
 */
export interface View {
  board: Board;
  /** Every recorded turn, oldest first. */
  turns: TurnInfo[];
  /** Index into `turns` of the recorded turn on screen; null is the live board. */
  viewing: number | null;
}

/** The list that was already in the store if it says the same thing, else the new one. */
function sameTurns(old: TurnInfo[], next: TurnInfo[]): TurnInfo[] {
  const unchanged =
    old.length === next.length &&
    next.every(
      (turn, i) =>
        turn.recordedAt === old[i].recordedAt && turn.pawns === old[i].pawns,
    );
  return unchanged ? old : next;
}

/**
 * Client state shared by the React UI and the engine. Components read it with
 * `useClientStore`; the engine uses `getState` and `subscribe` on the store it is given.
 */
export interface ClientState {
  tool: ToolId;
  marqueeShape: MarqueeShape;
  cameraMode: CameraMode;
  /** Ids of the selected units. Only ids present in `board`. */
  selection: number[];
  /** The latest board the server sent. */
  board: Board;
  /** The recorded turns, oldest first, as the server last listed them. */
  turns: TurnInfo[];
  /**
   * The recorded turn on screen, or null for the live board. While a turn is on screen this
   * umpire may not edit: the server refuses it, and the client offers nothing that would.
   */
  viewing: number | null;
  connection: Connection;
  /**
   * Something to tell the umpire that no board can show: an image the server would not
   * take, or a save file that would not load. Cleared when it has been read.
   */
  notice: string | null;
  /** Which keys mean what. Nothing changes this yet; a settings menu will. */
  bindings: Bindings;

  setTool: (tool: ToolId) => void;
  setMarqueeShape: (shape: MarqueeShape) => void;
  setCameraMode: (mode: CameraMode) => void;
  /** Change the selection. Ids that are not on the board are ignored. */
  select: (ids: number[], mode: SelectMode) => void;
  /** Show what the server's latest snapshot carried. */
  showView: (view: View) => void;
  setConnection: (connection: Connection) => void;
  setNotice: (notice: string | null) => void;
  setBindings: (bindings: Bindings) => void;
}

export const clientStore = createStore<ClientState>()((set) => ({
  tool: "select",
  marqueeShape: "box",
  cameraMode: "fly",
  selection: [],
  board: EMPTY_BOARD,
  turns: [],
  viewing: null,
  connection: "connecting",
  notice: null,
  bindings: DEFAULT_BINDINGS,

  setTool: (tool) => set({ tool }),
  setMarqueeShape: (marqueeShape) => set({ marqueeShape }),
  setCameraMode: (cameraMode) => set({ cameraMode }),
  select: (ids, mode) =>
    set((state) => {
      const onBoard = new Set(state.board.pawns.map((unit) => unit.id));
      const selected = new Set(mode === "replace" ? [] : state.selection);
      for (const id of ids) {
        if (!onBoard.has(id)) continue;
        if (mode === "toggle" && selected.has(id)) {
          selected.delete(id);
        } else {
          selected.add(id);
        }
      }
      return { selection: [...selected] };
    }),
  showView: ({ board, turns, viewing }) =>
    set((state) => {
      const ids = new Set(board.pawns.map((unit) => unit.id));
      const kept = state.selection.filter((id) => ids.has(id));
      // Keep the old array when nothing was dropped, so subscribers see no selection change.
      const selection =
        kept.length === state.selection.length ? state.selection : kept;
      // The same for the turn list, which comes with every board but rarely changes: the
      // timeline then re-renders when the history does, not on every edit anyone makes.
      return {
        board,
        viewing,
        selection,
        turns: sameTurns(state.turns, turns),
      };
    }),
  setConnection: (connection) => set({ connection }),
  setNotice: (notice) => set({ notice }),
  setBindings: (bindings) => set({ bindings }),
}));

export type ClientStore = typeof clientStore;

export function useClientStore<T>(selector: (state: ClientState) => T): T {
  return useStore(clientStore, selector);
}
