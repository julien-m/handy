// Delayed menu actions of the recording indicator (features 005, 006): the destination menu
// closes 400 ms after the pointer leaves the overlay and the prompt menu 250 ms after it leaves
// the pill. A pending action is cancelled by a newer one, by an explicit cancel, when its
// component unmounts and when a new dictation starts, so a timer left over from one dictation
// can never close a menu opened at the start of the next one.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** Backend event that shows the overlay for a state (`overlay::show_overlay_state`). */
export const SHOW_OVERLAY_EVENT = "show-overlay";

/** Overlay states that start a dictation (the others continue one). */
const DICTATION_STARTS = new Set(["recording", "streaming"]);

/** One pending delayed action at most. */
export interface MenuTimer {
  /** Runs `action` after `delayMs`, replacing any pending action. */
  schedule: (action: () => void, delayMs: number) => void;
  /** Drops the pending action, if any. */
  cancel: () => void;
}

/** A timer with no pending action. */
export const createMenuTimer = (): MenuTimer => {
  let pending: ReturnType<typeof setTimeout> | undefined;
  const cancel = () => {
    clearTimeout(pending);
    pending = undefined;
  };
  return {
    schedule: (action, delayMs) => {
      cancel();
      pending = setTimeout(() => {
        pending = undefined;
        action();
      }, delayMs);
    },
    cancel,
  };
};

/**
 * Effect body of [`useMenuTimer`]: nothing to set up; its cleanup (unmount, or a new
 * dictation) cancels the pending action.
 * @param timer The component's timer.
 * @returns The cleanup.
 */
export const cancelOnCleanup = (timer: MenuTimer): (() => void) => timer.cancel;

/**
 * The delayed-action timer of a component, cancelled on unmount and whenever `dictation`
 * changes.
 * @param dictation Number of the dictation the overlay shows.
 */
export const useMenuTimer = (dictation: number): MenuTimer => {
  const [timer] = useState(createMenuTimer);
  useEffect(() => cancelOnCleanup(timer), [timer, dictation]);
  return timer;
};

/**
 * Calls `onStart` each time the overlay is shown for a new dictation (recording or Live).
 * @param onStart Told of each new dictation.
 * @returns Cleanup that stops listening.
 */
export const watchDictationStarts = (onStart: () => void): (() => void) => {
  const unlisten = listen<string>(SHOW_OVERLAY_EVENT, (event) => {
    if (DICTATION_STARTS.has(event.payload)) onStart();
  });
  return () => {
    void unlisten.then((stop) => stop());
  };
};

/** Number of dictations the overlay has shown, starting at 0. */
export const useDictationCount = (): number => {
  const [count, setCount] = useState(0);
  useEffect(() => watchDictationStarts(() => setCount((n) => n + 1)), []);
  return count;
};
