import { useEffect, useRef, useState, type ReactNode } from "react";
import { serverConnection } from "../net/ServerConnection";
import type { ClientMessage, TurnInfo } from "../protocol";
import { useClientStore } from "../state/store";
import styles from "./Timeline.module.css";

/** The clock time a turn was recorded at, for its row in the list. */
function timeOf(turn: TurnInfo): string {
  return new Date(turn.recordedAt).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function unitCount(turn: TurnInfo): string {
  return turn.pawns === 1 ? "1 unit" : `${turn.pawns} units`;
}

/**
 * The turn history: step through recorded turns, jump to one from the list, record the board
 * as a turn, and come back to the live board.
 *
 * Which turn this umpire is looking at is the server's to know, so every control here only
 * sends a command; the snapshot that comes back is what the bar then shows. The timeline
 * therefore cannot disagree with the board on the table.
 */
export function Timeline() {
  const turns = useClientStore((s) => s.turns);
  const viewing = useClientStore((s) => s.viewing);
  const [open, setOpen] = useState(false);
  const panel = useRef<HTMLDivElement>(null);

  const send = (message: ClientMessage) => {
    serverConnection.send(message);
    setOpen(false);
  };

  // A pointer anywhere else, or Escape, closes the list. Listening on the window catches the
  // canvas too, where a press is about to start a drag.
  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!panel.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  const live = viewing === null;
  // How far along the history the board on screen sits, with the live board at the end.
  const spot = (live ? turns.length : viewing) + 1;
  const label = live
    ? turns.length === 0
      ? "Live · no turns recorded"
      : `Live · ${turns.length} recorded`
    : `Turn ${viewing + 1} of ${turns.length}`;

  return (
    <div
      ref={panel}
      className={styles.timeline}
      role="group"
      aria-label="Turn history"
    >
      <div className={styles.row}>
        <StepButton
          label="Previous turn"
          disabled={turns.length === 0 || viewing === 0}
          onClick={() => send({ type: "stepTurn", forward: false })}
        >
          <ArrowIcon />
        </StepButton>

        <button
          type="button"
          className={styles.bar}
          title="Recorded turns"
          aria-haspopup="listbox"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
        >
          <span
            className={live ? styles.fill : `${styles.fill} ${styles.past}`}
            style={{ width: `${(spot / (turns.length + 1)) * 100}%` }}
          />
          <span className={styles.label}>{label}</span>
          <span className={styles.caret} aria-hidden="true">
            ▾
          </span>
        </button>

        <StepButton
          label="Next turn"
          disabled={live}
          onClick={() => send({ type: "stepTurn", forward: true })}
        >
          <ArrowIcon forward />
        </StepButton>

        <div className={styles.divider} role="separator" />

        <StepButton
          label="Record the board as a turn"
          disabled={!live}
          onClick={() => send({ type: "recordTurn" })}
        >
          <RecordIcon />
        </StepButton>
      </div>

      {!live && (
        <div className={styles.viewing}>
          <span className={styles.note}>Recorded turn · read-only</span>
          <button
            type="button"
            className={styles.action}
            title="Copy this turn onto the live board"
            onClick={() => send({ type: "restoreTurn" })}
          >
            Restore
          </button>
          <button
            type="button"
            className={styles.action}
            onClick={() => send({ type: "viewTurn", turn: null })}
          >
            Go live
          </button>
        </div>
      )}

      {open && (
        <ul className={styles.menu} role="listbox" aria-label="Recorded turns">
          <li>
            <button
              type="button"
              role="option"
              aria-selected={live}
              className={styles.option}
              onClick={() => send({ type: "viewTurn", turn: null })}
            >
              <span className={styles.name}>Live</span>
              <span className={styles.detail}>current board</span>
            </button>
          </li>
          {/* Newest first: the turn an umpire wants back is usually the last one. */}
          {turns
            .map((turn, index) => ({ turn, index }))
            .reverse()
            .map(({ turn, index }) => (
              <li key={index}>
                <button
                  type="button"
                  role="option"
                  aria-selected={viewing === index}
                  className={styles.option}
                  onClick={() => send({ type: "viewTurn", turn: index })}
                >
                  <span className={styles.name}>Turn {index + 1}</span>
                  <span className={styles.detail}>
                    {timeOf(turn)} · {unitCount(turn)}
                  </span>
                </button>
              </li>
            ))}
          {turns.length === 0 && (
            <li className={styles.empty}>No turns recorded yet</li>
          )}
        </ul>
      )}
    </div>
  );
}

interface StepButtonProps {
  label: string;
  disabled: boolean;
  onClick: () => void;
  children: ReactNode;
}

function StepButton({ label, disabled, onClick, children }: StepButtonProps) {
  return (
    <button
      type="button"
      className={styles.step}
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function ArrowIcon({ forward }: { forward?: boolean }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      style={forward ? { transform: "scaleX(-1)" } : undefined}
    >
      <path d="M15 5l-7 7 7 7" />
    </svg>
  );
}

function RecordIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
      <circle cx="12" cy="12" r="6" fill="currentColor" />
    </svg>
  );
}
