// Polling of the microphone status (feature 001, AC-002): a read when the audio settings become
// visible and every 5 seconds while they stay visible. The main window is hidden, not closed, when
// the user closes it (it stays in the tray with the settings mounted), so polling follows the
// page visibility: hidden stops the timer and drops the pending read, visible reads at once and
// restarts the timer.
//
// Visibility source: the Page Visibility API (`document.visibilityState` and `visibilitychange`).
// Tauri hides the window natively and each webview reports it: on macOS, tao orders the NSWindow
// out and WKWebView marks the page hidden when its window leaves the screen; on Windows, wry calls
// `ICoreWebView2Controller::SetIsVisible(false)`; WebKitGTK follows the widget mapping. It needs
// no permission and no IPC, and the same code runs in tests. Observation in the real window is
// pending with Julien (Web Inspector: `document.visibilityState` after closing to the tray).

/** Timer functions, injected so tests can drive a simulated clock. */
export interface PollClock {
  setInterval: (callback: () => void, ms: number) => number;
  clearInterval: (id: number) => void;
}

/** Whether the settings are on screen, and a subscription to its changes. */
export interface VisibilitySource {
  isVisible: () => boolean;
  /** Call `onChange` on every change; returns the unsubscribe function. */
  subscribe: (onChange: () => void) => () => void;
}

/** What the polling needs. */
export interface StatusPollingDeps {
  /** Start one status read. */
  read: () => void;
  /** Drop the pending read so its answer is never applied. */
  invalidate: () => void;
  clock: PollClock;
  visibility: VisibilitySource;
  periodMs: number;
}

/**
 * Poll the status while the settings are visible.
 * Side effects: reads at once when visible, then on every period; subscribes to visibility.
 * @param deps read, invalidation, clock, visibility and period
 * @returns the stop function (unsubscribes, clears the timer, drops the pending read)
 */
export const startStatusPolling = (deps: StatusPollingDeps): (() => void) => {
  let timer: number | null = null;
  const pause = () => {
    if (timer !== null) deps.clock.clearInterval(timer);
    timer = null;
    deps.invalidate();
  };
  const resume = () => {
    if (timer !== null) return;
    deps.read();
    timer = deps.clock.setInterval(deps.read, deps.periodMs);
  };
  const follow = () => (deps.visibility.isVisible() ? resume() : pause());
  follow();
  const unsubscribe = deps.visibility.subscribe(follow);
  return () => {
    unsubscribe();
    pause();
  };
};

/** Visibility of the settings page, from the Page Visibility API. */
export const documentVisibility: VisibilitySource = {
  isVisible: () => document.visibilityState !== "hidden",
  subscribe: (onChange) => {
    document.addEventListener("visibilitychange", onChange);
    return () => document.removeEventListener("visibilitychange", onChange);
  },
};

/** The window's timers. */
export const windowClock: PollClock = {
  setInterval: (callback, ms) => window.setInterval(callback, ms),
  clearInterval: (id) => window.clearInterval(id),
};
