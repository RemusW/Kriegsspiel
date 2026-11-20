import { EMPTY_BOARD } from "../model";
import type { ClientMessage, ServerMessage } from "../protocol";
import { clientStore, type ClientStore, type View } from "../state/store";
import { socketEndpoint } from "./endpoint";

const RETRY_MS = 1500;

/** What is shown while no server has sent anything: an empty table and no history. */
const BLANK_VIEW: View = { board: EMPTY_BOARD, turns: [], viewing: null };

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
    this.store.getState().showView(BLANK_VIEW);
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

  /**
   * Send raw bytes: an image for the server's store, which is the only thing that does not
   * travel as a command. A socket keeps order, so a command naming this image may be sent
   * immediately after without waiting for the server to confirm it.
   */
  sendBytes(bytes: ArrayBuffer): boolean {
    if (this.socket?.readyState !== WebSocket.OPEN) return false;
    this.socket.send(bytes);
    return true;
  }

  private open() {
    const { showView, setConnection } = this.store.getState();
    setConnection("connecting");

    const socket = new WebSocket(socketEndpoint());
    this.socket = socket;
    socket.onopen = () => setConnection("connected");
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data as string) as ServerMessage;
      switch (message.type) {
        case "snapshot": {
          const { board, turns, viewing } = message;
          showView({ board, turns, viewing });
          break;
        }
        case "assetAdded":
          // The client already worked out this id by hashing the file, so there is nothing
          // to do but check the two agree.
          break;
        case "assetRejected":
          this.store.getState().setNotice(message.reason);
          break;
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
