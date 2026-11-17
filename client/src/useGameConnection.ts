import { useEffect } from "react";
import { serverConnection } from "./net/ServerConnection";

/** Keep the connection to the game server open while the calling component is mounted. */
export function useGameConnection() {
  useEffect(() => {
    serverConnection.connect();
    return () => serverConnection.disconnect();
  }, []);
}
