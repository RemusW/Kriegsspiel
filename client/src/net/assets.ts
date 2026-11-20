// Images and save files: everything that travels as bytes rather than as a command.
//
// An image's id is the SHA-256 of its bytes, which the server computes too. Hashing a file
// here means the client knows the id before the upload has even finished, so it can send the
// command that uses the image straight after the bytes without waiting for a reply.

import { SAVE_EXTENSION } from "../protocol";
import { httpEndpoint } from "./endpoint";
import { serverConnection } from "./ServerConnection";

/**
 * Where an image is served from. The id is the hash of the content, so this URL can never
 * come to mean a different image and the browser is told it may keep it for good.
 */
export function assetUrl(id: string): string {
  return httpEndpoint(`/assets/${id}`);
}

/** The asset id of these bytes: the same SHA-256 the server files them under. */
export async function assetId(bytes: ArrayBuffer): Promise<string> {
  // `crypto.subtle` needs a secure context, which localhost and the HTTPS relay both are.
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Upload an image and return its id. The bytes go over the socket as one binary message; a
 * server that will not take them answers with `assetRejected`, which the connection shows as
 * a notice. Throws when there is no connection to send them on, rather than returning
 * quietly: a button that does nothing and says nothing is the one failure nobody can debug.
 */
export async function uploadImage(file: File): Promise<string> {
  const bytes = await file.arrayBuffer();
  const id = await assetId(bytes);
  if (!serverConnection.sendBytes(bytes)) {
    throw new Error("not connected to the game server");
  }
  return id;
}

/** Lay an image on the table: upload it, then point the board at it. */
export async function setMapImage(file: File): Promise<void> {
  const asset = await uploadImage(file);
  serverConnection.send({ type: "setMapImage", asset });
}

/** Put an image on the top face of these units. */
export async function setUnitImage(
  file: File,
  units: readonly number[],
): Promise<void> {
  if (units.length === 0) return;
  const asset = await uploadImage(file);
  serverConnection.send({ type: "setUnitImage", units: [...units], asset });
}

/** What a save file is called when it is downloaded. */
function saveName(): string {
  // Local time, so the name matches the clock the umpire is looking at.
  const now = new Date();
  const stamp = [
    now.getFullYear(),
    `${now.getMonth() + 1}`.padStart(2, "0"),
    `${now.getDate()}`.padStart(2, "0"),
  ].join("-");
  return `kriegsspiel-${stamp}.${SAVE_EXTENSION}`;
}

/**
 * Download the game as a save file. The server builds it, because it is the only thing that
 * holds both the document and the images; the client only asks for the bytes and hands them
 * to the browser.
 */
export async function saveGame(): Promise<void> {
  const response = await fetch(httpEndpoint("/save"));
  if (!response.ok) {
    throw new Error(await response.text());
  }
  const url = URL.createObjectURL(await response.blob());
  try {
    const link = document.createElement("a");
    link.href = url;
    link.download = saveName();
    link.click();
  } finally {
    // The click has already started the download, which holds its own reference.
    URL.revokeObjectURL(url);
  }
}

/**
 * Load a save file, replacing the game for everyone connected. The server reads it and sends
 * every client a fresh board, so nothing here has to unpack the file.
 */
export async function loadGame(file: File): Promise<void> {
  const response = await fetch(httpEndpoint("/load"), {
    method: "POST",
    body: file,
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
}
