// Visibility of the Tauri window that holds the settings (feature 001, AC-002), for the status
// polling of `status-polling.ts`. The page visibility alone is not enough: closing Handy hides its
// main window (`window.hide()` in `lib.rs`), and on Windows wry 0.55 does not tell WebView2 that
// the page is hidden. The window answers `isVisible()` and `isMinimized()` (permissions
// `core:window:allow-is-visible` and `allow-is-minimized`, part of `core:default`): a minimized
// window counts as hidden, because on Windows `IsWindowVisible` stays true for a minimized window
// and WebView2 is not told either. Two window signals ask again: a focus change (hiding or minimizing
// usually blurs the window, clicking it back focuses it) and the backend's `main-window-shown`
// event, which `show_main_window` emits after showing the window from the tray, the dock or a
// second launch, so polling resumes even when that window gets no focus event.

import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { VisibilitySource } from "./status-polling";

/** Backend event emitted by `show_main_window` (`MAIN_WINDOW_SHOWN_EVENT` in `lib.rs`). */
export const MAIN_WINDOW_SHOWN_EVENT = "main-window-shown";

/** Subscription to a window signal; resolves to the unlisten function. */
export type SignalListener = (handler: () => void) => Promise<() => void>;

/** The window operations the source needs, injected so tests can fake the window. */
export interface WindowPort {
  isVisible: () => Promise<boolean>;
  isMinimized: () => Promise<boolean>;
  /** Call `handler` on every focus change. */
  onFocusChanged: SignalListener;
  /** Call `handler` each time the backend shows the window (`main-window-shown`). */
  onShown: SignalListener;
}

/** Where failures of the window port are reported. */
export type WarningSink = (message: string, error: unknown) => void;

/** The current Tauri window; it is looked up on each call, so a missing window only rejects. */
export const tauriWindowPort: WindowPort = {
  isVisible: async () => getCurrentWindow().isVisible(),
  isMinimized: async () => getCurrentWindow().isMinimized(),
  onFocusChanged: async (handler) =>
    getCurrentWindow().onFocusChanged(() => handler()),
  onShown: async (handler) => listen(MAIN_WINDOW_SHOWN_EVENT, () => handler()),
};

/**
 * Subscribe to one window signal; the subscription resolves later, so an unsubscribe that comes
 * first releases it as soon as it resolves.
 * @param listener the signal's subscription
 * @param onChange called on every signal
 * @param fail logs a failed subscription
 * @returns the unsubscribe function
 */
const subscribeToSignal = (
  listener: SignalListener,
  onChange: () => void,
  fail: (error: unknown) => void,
): (() => void) => {
  let unlisten: (() => void) | null = null;
  let closed = false;
  const subscribe = async () => {
    try {
      const release = await listener(onChange);
      if (closed) release();
      else unlisten = release;
    } catch (error) {
      fail(error);
    }
  };
  void subscribe();
  return () => {
    closed = true;
    unlisten?.();
  };
};

/**
 * Subscribe to the window's focus changes and to `main-window-shown`.
 * @param port window operations
 * @param onChange called on every signal
 * @param warn failure log
 * @returns the function that releases both subscriptions
 */
const subscribeToWindow = (
  port: WindowPort,
  onChange: () => void,
  warn: WarningSink,
): (() => void) => {
  const focus = subscribeToSignal(port.onFocusChanged, onChange, (error) =>
    warn("Window focus subscription failed:", error),
  );
  const shown = subscribeToSignal(port.onShown, onChange, (error) =>
    warn("Window shown subscription failed:", error),
  );
  return () => {
    focus();
    shown();
  };
};

/**
 * Whether the window is on screen: shown and not minimized. The minimized state is asked only for
 * a shown window.
 * @param port window operations
 * @returns false for a hidden or minimized window
 */
const isOnScreen = async (port: WindowPort): Promise<boolean> =>
  (await port.isVisible()) && !(await port.isMinimized());

/**
 * Window visibility as a polling source.
 *
 * When `isVisible()` or `isMinimized()` fails the answer is "visible", so the page visibility
 * alone decides, as before this source existed: the status shown in an open window keeps
 * refreshing (AC-002), at the cost of possible background reads if the page also stays visible
 * while hidden. The first failure is logged; later ones are not, to avoid a warning every 5 s. A
 * failed subscription is logged too: the polling then resumes on the other signals only.
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
        return await isOnScreen(port);
      } catch (error) {
        if (!failureLogged) warn("Window visibility read failed:", error);
        failureLogged = true;
        return true;
      }
    },
    subscribe: (onChange) => subscribeToWindow(port, onChange, warn),
  };
};
