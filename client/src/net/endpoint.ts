// Where the game server is, derived from the page's own host: the desktop app serves the
// client from the same machine the server runs on, and a player's browser is pointed at the
// umpire. Both the socket and the HTTP routes are on the one port.

import { DEFAULT_PORT } from "../protocol";

function host(): string {
  return window.location.hostname || "localhost";
}

/** The game socket, which carries the board and the commands that change it. */
export function socketEndpoint(): string {
  return `ws://${host()}:${DEFAULT_PORT}/ws`;
}

/**
 * An HTTP route on the game server, for the things a socket is the wrong tool for: images
 * the browser should cache, and save files it should download.
 */
export function httpEndpoint(path: string): string {
  return `http://${host()}:${DEFAULT_PORT}${path}`;
}
