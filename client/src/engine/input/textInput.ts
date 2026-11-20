/** True while the event's target is somewhere the user is typing, where keys are theirs. */
export function isTextInput(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

/** Controls the browser presses with Enter or Space while they have focus. */
const ACTIVATED_BY_KEY = 'button, summary, a[href], [role="button"], [role="option"]';

/**
 * True when a key press belongs to the control it happened in rather than to the editor:
 * typing in a field, or the Enter and Space that press a focused button. A bound action must
 * leave those alone, because acting on a key also cancels the browser's own handling of it,
 * and that handling is what makes a button work from the keyboard.
 */
export function belongsToControl(event: KeyboardEvent): boolean {
  if (isTextInput(event.target)) return true;
  if (event.code !== "Enter" && event.code !== "Space") return false;
  return (
    event.target instanceof HTMLElement &&
    event.target.closest(ACTIVATED_BY_KEY) !== null
  );
}
