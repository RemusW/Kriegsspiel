import { useEffect, useRef, useState, type ChangeEvent } from "react";
import { loadGame, saveGame, setMapImage, setUnitImage } from "../net/assets";
import { SAVE_EXTENSION, SAVE_MIME } from "../protocol";
import { useClientStore } from "../state/store";
import styles from "./FileBar.module.css";

/** What the file dialogs offer, matching the formats the server's asset store accepts. */
const IMAGE_TYPES = "image/png,image/jpeg";
const SAVE_TYPES = `.${SAVE_EXTENSION},${SAVE_MIME}`;

/** A save file holds the whole game, so replacing the one in play is worth a confirmation. */
const REPLACE_WARNING =
  "Load this save? It replaces the board, the recorded turns and the history for everyone connected.";

/**
 * Files in and out: lay a map image on the table, put an image on the selected units, and
 * save or open a game.
 *
 * None of this unpacks or builds a file here. The server owns the document and the images,
 * so it is the only thing that can write a consistent save; the client asks for the bytes,
 * or hands it the bytes it was given, and waits for the board that comes back.
 */
export function FileBar() {
  const selection = useClientStore((s) => s.selection);
  const viewing = useClientStore((s) => s.viewing);
  const notice = useClientStore((s) => s.notice);
  const setNotice = useClientStore((s) => s.setNotice);
  // `busy` redraws the buttons as disabled. `working` is what is checked: a ref is one object
  // for the component's whole life, so the window's drop listener, set up in an earlier
  // render, still reads the current value.
  const [busy, setBusy] = useState(false);
  const working = useRef(false);

  // Editing is refused while a recorded turn is on screen, so nothing that edits is offered.
  const readOnly = viewing !== null;

  /**
   * Run one file operation, showing the reason if it fails rather than only logging it.
   * Ignored while another is running, whether it came from a button or a dropped file.
   */
  const run = async (what: string, action: () => Promise<void>) => {
    if (working.current) return;
    working.current = true;
    setBusy(true);
    setNotice(null);
    try {
      await action();
    } catch (error) {
      setNotice(
        `${what} failed: ${error instanceof Error ? error.message : error}`,
      );
    } finally {
      working.current = false;
      setBusy(false);
    }
  };

  const openSave = (file: File) => {
    // Checked before asking, so a file dropped mid-operation brings up no dialog.
    if (working.current || !window.confirm(REPLACE_WARNING)) return;
    void run("Open", () => loadGame(file));
  };

  // Dropping a file anywhere on the window does what picking it would: an image goes on the
  // table, anything else is tried as a save.
  useEffect(() => {
    const onDragOver = (event: DragEvent) => event.preventDefault();
    const onDrop = (event: DragEvent) => {
      event.preventDefault();
      const file = event.dataTransfer?.files[0];
      if (!file) return;
      if (file.type.startsWith("image/")) {
        if (!readOnly) void run("Map image", () => setMapImage(file));
      } else {
        openSave(file);
      }
    };
    window.addEventListener("dragover", onDragOver);
    window.addEventListener("drop", onDrop);
    return () => {
      window.removeEventListener("dragover", onDragOver);
      window.removeEventListener("drop", onDrop);
    };
  }, [readOnly]);

  return (
    <div className={styles.filebar} role="group" aria-label="Files">
      <div className={styles.row}>
        <FilePicker
          label="Map image"
          title="Lay an image on the table"
          accept={IMAGE_TYPES}
          disabled={busy || readOnly}
          onPick={(file) => run("Map image", () => setMapImage(file))}
        />
        <FilePicker
          label="Unit image"
          title={
            selection.length === 0
              ? "Select units first"
              : `Put an image on ${selection.length} selected`
          }
          accept={IMAGE_TYPES}
          disabled={busy || readOnly || selection.length === 0}
          onPick={(file) =>
            run("Unit image", () => setUnitImage(file, selection))
          }
        />
        <div className={styles.divider} role="separator" />
        <button
          type="button"
          className={styles.button}
          title="Download the game as one file, images included"
          disabled={busy}
          onClick={() => void run("Save", saveGame)}
        >
          Save
        </button>
        <FilePicker
          label="Open"
          title="Load a save file, replacing the game"
          accept={SAVE_TYPES}
          disabled={busy}
          onPick={openSave}
        />
      </div>

      {notice && (
        <button
          type="button"
          className={styles.notice}
          title="Dismiss"
          onClick={() => setNotice(null)}
        >
          {notice}
        </button>
      )}
    </div>
  );
}

interface FilePickerProps {
  label: string;
  title: string;
  accept: string;
  disabled: boolean;
  onPick: (file: File) => void;
}

/**
 * A button that opens the browser's file dialog. The input itself is hidden: it is the only
 * way to read a local file, but it cannot be styled like the other controls.
 */
function FilePicker({
  label,
  title,
  accept,
  disabled,
  onPick,
}: FilePickerProps) {
  const input = useRef<HTMLInputElement>(null);

  const onChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    // Cleared so that picking the same file again still counts as a change.
    event.target.value = "";
    if (file) onPick(file);
  };

  return (
    <>
      <button
        type="button"
        className={styles.button}
        title={title}
        disabled={disabled}
        onClick={() => input.current?.click()}
      >
        {label}
      </button>
      <input
        ref={input}
        type="file"
        accept={accept}
        className={styles.input}
        tabIndex={-1}
        aria-hidden="true"
        onChange={onChange}
      />
    </>
  );
}
