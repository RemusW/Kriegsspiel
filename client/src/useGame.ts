import { useEffect, useState } from "react";
import { EMPTY_BOARD, type Board } from "./model";
import { DEFAULT_PORT, type ServerMessage } from "./protocol";
import type { Connection } from "./state/store";

const RETRY_MS = 1500;

function serverUrl(): string {
  const host = window.location.hostname || "localhost";
  return `ws://${host}:${DEFAULT_PORT}/ws`;
}

/** Connect to the game server and keep the latest board it sent. Reconnects on its own. */
export function useGame(): { board: Board; connection: Connection } {
  const [board, setBoard] = useState<Board>(EMPTY_BOARD);
  const [connection, setConnection] = useState<Connection>("connecting");

  useEffect(() => {
    let socket: WebSocket | undefined;
    let retry: ReturnType<typeof setTimeout> | undefined;
    let closed = false;

    const connect = () => {
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
    };
  }, []);

  return { board, connection };
}
