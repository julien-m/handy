// Visibility of the Tauri window that holds the settings (feature 001, AC-002), for the status
// polling of `status-polling.ts`. The page visibility alone is not enough: closing Handy hides its
// main window (`window.hide()` in `lib.rs`), and on Windows wry 0.55 does not tell WebView2 that
// the page is hidden. The window answers `isVisible()` (permission `core:window:allow-is-visible`,
// part of `core:default`), and its focus changes are the signal to ask again: reopening Handy from
// the tray shows and focuses the window (`show_main_window`), and hiding it usually blurs it.

import { getCurrentWindow } from "@tauri-apps/api/window";
import type { VisibilitySource } from "./status-polling";

/** The window operations the source needs, injected so tests can fake the window. */
export interface WindowPort {
  isVisible: () => Promise<boolean>;
  /** Call `handler` with the new focus state; resolves to the unlisten function. */
  onFocusChanged: (handler: (focused: boolean) => void) => Promise<() => void>;
}

/** Where failures of the window port are reported. */
export type WarningSink = (message: string, error: unknown) => void;

/** The current Tauri window; it is looked up on each call, so a missing window only rejects. */
export const tauriWindowPort: WindowPort = {
  isVisible: async () => getCurrentWindow().isVisible(),
  onFocusChanged: async (handler) =>
    getCurrentWindow().onFocusChanged((event) => handler(event.payload)),
};

/**
 * Subscribe to the window's focus changes; the subscription resolves later, so an unsubscribe that
 * comes first releases it as soon as it resolves.
 * @param port window operations
 * @param onChange called on every focus change
 * @param warn failure log
 * @returns the unsubscribe function
 */
const subscribeToFocus = (
  port: WindowPort,
  onChange: () => void,
  warn: WarningSink,
): (() => void) => {
  let unlisten: (() => void) | null = null;
  let closed = false;
  const listen = async () => {
    try {
      const release = await port.onFocusChanged(() => onChange());
      if (closed) release();
      else unlisten = release;
    } catch (error) {
      warn("Window focus subscription failed:", error);
    }
  };
  void listen();
  return () => {
    closed = true;
    unlisten?.();
  };
};

/**
 * Window visibility as a polling source.
 *
 * When `isVisible()` fails the answer is "visible", so the page visibility alone decides, as before
 * this source existed: the status shown in an open window keeps refreshing (AC-002), at the cost of
 * possible background reads if the page also stays visible while hidden. The first failure is
 * logged; later ones are not, to avoid a warning every 5 s. A failed focus subscription is logged
 * too: the polling then resumes only on a page visibility change.
 * @param port window operations
 * @param warn failure log, `console.warn` by default
 * @returns the source
 */
export const windowVisibility = (
  port: WindowPort,
  warn: WarningSink = console.warn,
): VisibilitySource => {
  let failureLogged = false;
  return {
    isVisible: async () => {
      try {
        return await port.isVisible();
      } catch (error) {
        if (!failureLogged) warn("Window visibility read failed:", error);
        failureLogged = true;
        return true;
      }
    },
    subscribe: (onChange) => subscribeToFocus(port, onChange, warn),
  };
};
