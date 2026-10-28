// Polling of the microphone status (feature 001, AC-002): a read when the audio settings become
// visible and every 5 seconds while they stay visible. The main window is hidden, not closed, when
// the user closes it (it stays in the tray with the settings mounted), so polling follows the
// visibility of the settings: hidden stops the timer and drops the pending read, visible reads at
// once and restarts the timer.
//
// Visibility sources, all required (see `allVisible`):
// - the Page Visibility API (`document.visibilityState` and `visibilitychange`), which a webview
//   may update when its window is hidden;
// - the Tauri window itself (`window-visibility.ts`): `isVisible()` and `isMinimized()` are asked
//   before every read, and a focus change or the backend's `main-window-shown` event asks again.
//   This covers a webview that keeps reporting "visible" after `window.hide()` or a minimize:
//   wry 0.55 does not call `ICoreWebView2Controller::SetIsVisible` on a window hide, so WebView2
//   on Windows very probably never reports the page hidden.
// Neither source has been observed in a real window yet: macOS is pending with Julien, Windows and
// Linux are pending release evidence (`document.visibilityState` and the status calls after
// closing the window to the tray, then after reopening it from the tray).

/** Timer functions, injected so tests can drive a simulated clock. */
export interface PollClock {
  setInterval: (callback: () => void, ms: number) => number;
  clearInterval: (id: number) => void;
}

/** Whether the settings are on screen, and a subscription to the signals that may change it. */
export interface VisibilitySource {
  /** Current visibility. Never rejects: a source that cannot tell answers its documented default. */
  isVisible: () => Promise<boolean>;
  /** Call `onChange` on every signal; returns the unsubscribe function. */
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
 * Timer and visibility questions of one polling run. Each period asks the visibility first and
 * reads only when it is visible; a hidden answer stops the timer until a visibility signal reports
 * the settings visible again, which reads at once and restarts the timer. Answers arrive
 * asynchronously, so only the newest question decides (an older answer could be stale).
 */
class StatusPoller {
  private timer: number | null = null;
  private question = 0;
  private readDue = false;
  private stopped = false;

  constructor(private readonly deps: StatusPollingDeps) {}

  /** Ask the visibility, then pause, resume or read. */
  follow = async (): Promise<void> => {
    const ticket = ++this.question;
    const visible = await this.deps.visibility.isVisible();
    if (!this.stopped && ticket === this.question) this.settle(visible);
  };

  /** Stop for good: no timer, no pending read, later answers ignored. */
  stop = (): void => {
    this.stopped = true;
    this.pause();
  };

  private onPeriod = (): void => {
    this.readDue = true;
    void this.follow();
  };

  private settle(visible: boolean): void {
    if (!visible) return this.pause();
    if (this.timer === null) {
      // Shown again (or first shown): read at once, then on every period.
      this.readDue = true;
      this.timer = this.deps.clock.setInterval(
        this.onPeriod,
        this.deps.periodMs,
      );
    }
    if (this.readDue) this.deps.read();
    this.readDue = false;
  }

  private pause(): void {
    if (this.timer !== null) this.deps.clock.clearInterval(this.timer);
    this.timer = null;
    this.readDue = false;
    this.deps.invalidate();
  }
}

/**
 * Poll the status while the settings are visible (see StatusPoller).
 * Side effects: reads, sets and clears an interval timer, subscribes to visibility signals.
 * @param deps read, invalidation, clock, visibility and period
 * @returns the stop function (unsubscribes, clears the timer, drops the pending read)
 */
export const startStatusPolling = (deps: StatusPollingDeps): (() => void) => {
  const poller = new StatusPoller(deps);
  void poller.follow();
  const unsubscribe = deps.visibility.subscribe(() => void poller.follow());
  return () => {
    unsubscribe();
    poller.stop();
  };
};

/**
 * Visibility that holds only when every source reports visible (a hidden page or a hidden window
 * is enough to stop polling); a signal from any source asks again.
 * @param sources visibility sources, asked in order
 * @returns the combined source
 */
export const allVisible = (
  sources: readonly VisibilitySource[],
): VisibilitySource => ({
  isVisible: async () => {
    for (const source of sources) {
      // The first hidden source decides: later ones (an IPC for the window) are not asked.
      if (!(await source.isVisible())) return false;
    }
    return true;
  },
  subscribe: (onChange) => {
    const unsubscribers = sources.map((source) => source.subscribe(onChange));
    return () => unsubscribers.forEach((unsubscribe) => unsubscribe());
  },
});

/** Visibility of the settings page, from the Page Visibility API. */
export const documentVisibility: VisibilitySource = {
  isVisible: async () => document.visibilityState !== "hidden",
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
