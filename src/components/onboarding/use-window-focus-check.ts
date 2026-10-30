// Checks the permissions again whenever the window takes the focus back (native session finding
// N10): the user returns from System Settings, where the permission was granted, and the screen
// no longer waits for a polling that may have stopped.
import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Runs `check` each time the window gains the focus, while `active`.
 * Side effects: listens to the Tauri window focus event and to the page's `focus` event while
 * active; a listener that cannot be installed is logged and the other one still works.
 * @param active whether permissions are still asked for
 * @param check one check of the permissions
 */
export const useWindowFocusCheck = (
  active: boolean,
  check: () => Promise<void>,
): void => {
  const latest = useRef(check);
  latest.current = check;
  useEffect(() => {
    if (!active) return;
    const run = () => void latest.current();
    window.addEventListener("focus", run);
    let stop: (() => void) | null = null;
    let cancelled = false;
    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) run();
      })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else stop = unlisten;
      })
      .catch((error: unknown) => {
        console.warn("Window focus changes are not observed:", error);
      });
    return () => {
      cancelled = true;
      window.removeEventListener("focus", run);
      stop?.();
    };
  }, [active]);
};
