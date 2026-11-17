import { EMPTY_BOARD } from "../model";
import {
  DEFAULT_PORT,
  type ClientMessage,
  type ServerMessage,
} from "../protocol";
import { clientStore, type ClientStore } from "../state/store";

const RETRY_MS = 1500;

function serverUrl(): string {
  const host = window.location.hostname || "localhost";
  return `ws://${host}:${DEFAULT_PORT}/ws`;
}

/**
 * The WebSocket to the game server. Writes the boards the server sends into the store, and
 * reconnects on its own between `connect` and `disconnect`.
 */
export class ServerConnection {
  private socket: WebSocket | null = null;
  private retry: ReturnType<typeof setTimeout> | undefined;

  constructor(private store: ClientStore) {}

  connect() {
    if (this.socket) return;
    this.open();
  }

  disconnect() {
    clearTimeout(this.retry);
    const socket = this.socket;
    this.socket = null;
    socket?.close();
    this.store.getState().setBoard(EMPTY_BOARD);
  }

  /**
   * Send a message to the server. While not connected the message is dropped and this
   * returns false: replaying commands after a reconnect could apply them to a board that
   * has changed since.
   */
  send(message: ClientMessage): boolean {
    if (this.socket?.readyState !== WebSocket.OPEN) return false;
    this.socket.send(JSON.stringify(message));
    return true;
  }

  private open() {
    const { setBoard, setConnection } = this.store.getState();
    setConnection("connecting");

    const socket = new WebSocket(serverUrl());
    this.socket = socket;
    socket.onopen = () => setConnection("connected");
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data as string) as ServerMessage;
      if (message.type === "snapshot") {
        setBoard(message.board);
      }
    };
    socket.onclose = () => {
      // A socket that `disconnect` closed has already been replaced or dropped.
      if (this.socket !== socket) return;
      setConnection("disconnected");
      this.retry = setTimeout(() => this.open(), RETRY_MS);
    };
  }
}

export const serverConnection = new ServerConnection(clientStore);
