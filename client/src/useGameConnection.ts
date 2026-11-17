import { useEffect } from "react";
import { DEFAULT_PORT, type ServerMessage } from "./protocol";
import { clientStore } from "./state/store";
import { EMPTY_BOARD } from "./model";

const RETRY_MS = 1500;

function serverUrl(): string {
  const host = window.location.hostname || "localhost";
  return `ws://${host}:${DEFAULT_PORT}/ws`;
}

/** Connect to the game server and keep the latest board it sent. Reconnects on its own. */
export function useGameConnection() {
  useEffect(() => {
    const { setBoard, setConnection } = clientStore.getState();

    let socket: WebSocket | undefined;
    let retry: ReturnType<typeof setTimeout> | undefined;
    let closed = false;

    const connect = () => {
      setConnection("connecting");

      socket = new WebSocket(serverUrl());
      socket.onopen = () => setConnection("connected");
      socket.onmessage = (event) => {
        const message = JSON.parse(event.data as string) as ServerMessage;
        if (message.type === "snapshot") {
          setBoard(message.board);
        }
      };
      socket.onclose = () => {
        if (closed) return;
        setConnection("disconnected");
        retry = setTimeout(connect, RETRY_MS);
      };
    };
    connect();

    return () => {
      closed = true;
      clearTimeout(retry);
      socket?.close();
      setBoard(EMPTY_BOARD);
    };
  }, []);
}
