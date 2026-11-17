import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";
import { EMPTY_BOARD, type Board } from "../model";

export type Tool = "select" | "marquee" | "spawn";
export type MarqueeShape = "box" | "lasso";
export type Connection = "connecting" | "connected" | "disconnected";
export type CameraMode = "fly" | "map";

/**
 * Client state shared by the React UI and the engine. Components read it with
 * `useClientStore`; the engine uses `getState` and `subscribe` on the store it is given.
 */
export interface ClientState {
  tool: Tool;
  marqueeShape: MarqueeShape;
  cameraMode: CameraMode;
  /** Ids of the selected units. Only ids present in `board`. */
  selection: number[];
  /** The latest board the server sent. */
  board: Board;
  connection: Connection;

  setTool: (tool: Tool) => void;
  setMarqueeShape: (shape: MarqueeShape) => void;
  setCameraMode: (mode: CameraMode) => void;
  setSelection: (ids: number[]) => void;
  setBoard: (board: Board) => void;
  setConnection: (connection: Connection) => void;
}

export const clientStore = createStore<ClientState>()((set) => ({
  tool: "select",
  marqueeShape: "box",
  cameraMode: "fly",
  selection: [],
  board: EMPTY_BOARD,
  connection: "connecting",

  setTool: (tool) => set({ tool }),
  setMarqueeShape: (marqueeShape) => set({ marqueeShape }),
  setCameraMode: (cameraMode) => set({ cameraMode }),
  setSelection: (selection) => set({ selection }),
  setBoard: (board) =>
    set((state) => {
      const ids = new Set(board.pawns.map((unit) => unit.id));
      const kept = state.selection.filter((id) => ids.has(id));
      // Keep the old array when nothing was dropped, so selection subscribers are not notified.
      const selection =
        kept.length === state.selection.length ? state.selection : kept;
      return { board, selection };
    }),
  setConnection: (connection) => set({ connection }),
}));

export type ClientStore = typeof clientStore;

export function useClientStore<T>(selector: (state: ClientState) => T): T {
  return useStore(clientStore, selector);
}
