/**
 * Polling of the microphone status (feature 001, AC-002) on a simulated clock, a simulated page
 * visibility and a fake Tauri window: a read on open, one every 5 s while visible, none while the
 * page or the window is hidden or minimized, a read at once when shown again (focus or the
 * backend's `main-window-shown`), nothing after stop, and the documented fallback when the window
 * cannot tell. Run with
 * `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import {
  allVisible,
  startStatusPolling,
  type PollClock,
  type VisibilitySource,
} from "./status-polling";
import { windowVisibility, type WindowPort } from "./window-visibility";

const PERIOD = 5000;

/** Interval timers driven by `advance`, with the number of live timers. */
const simulatedClock = () => {
  let now = 0;
  let nextId = 1;
  const timers = new Map<
    number,
    { callback: () => void; ms: number; due: number }
  >();
  const clock: PollClock = {
    setInterval: (callback, ms) => {
      const id = nextId++;
      timers.set(id, { callback, ms, due: now + ms });
      return id;
    },
    clearInterval: (id) => void timers.delete(id),
  };
  const advance = (ms: number) => {
    const end = now + ms;
    for (;;) {
      const next = [...timers.values()].sort((a, b) => a.due - b.due)[0];
      if (next === undefined || next.due > end) break;
      now = next.due;
      next.due += next.ms;
      next.callback();
    }
    now = end;
  };
  return { clock, advance, live: () => timers.size };
};

/** Page visibility switched by `set`, with the number of subscribers. */
const simulatedVisibility = (initial: boolean) => {
  let visible = initial;
  const listeners = new Set<() => void>();
  const source: VisibilitySource = {
    isVisible: async () => visible,
    subscribe: (onChange) => {
      listeners.add(onChange);
      return () => void listeners.delete(onChange);
    },
  };
  const set = (next: boolean) => {
    visible = next;
    listeners.forEach((listener) => listener());
  };
  return { source, set, subscribers: () => listeners.size };
};

/** Let pending visibility answers settle (they resolve in microtasks). */
const answered = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** Polling on a simulated clock with `visibility`, counting reads and invalidations. */
const poll = (visibility: VisibilitySource) => {
  const time = simulatedClock();
  const counts = { reads: 0, invalidations: 0 };
  const stop = startStatusPolling({
    read: () => void (counts.reads += 1),
    invalidate: () => void (counts.invalidations += 1),
    clock: time.clock,
    visibility,
    periodMs: PERIOD,
  });
  return { time, counts, stop };
};

/** Polling bound to a simulated page visibility only, once the first answer settled. */
const setup = async (visible: boolean) => {
  const page = simulatedVisibility(visible);
  const run = poll(page.source);
  await answered();
  /** Advance the clock and let the answers of the elapsed periods settle. */
  const advance = async (ms: number) => {
    run.time.advance(ms);
    await answered();
  };
  /** Switch the page visibility and let the answer settle. */
  const set = async (next: boolean) => {
    page.set(next);
    await answered();
  };
  return { ...run, page, advance, set };
};

// @spec AC-002: Status read on open and every 5 s while visible — .specs/features/001-microphone-priority/spec.md#ac-002
describe("status polling", () => {
  test("reads when opened, then every 5 seconds", async () => {
    const { advance, counts } = await setup(true);
    expect(counts.reads).toBe(1);
    await advance(PERIOD - 1);
    expect(counts.reads).toBe(1);
    await advance(1);
    expect(counts.reads).toBe(2);
    for (let period = 0; period < 3; period += 1) await advance(PERIOD);
    expect(counts.reads).toBe(5);
  });

  test("hidden: the timer stops and the pending read is dropped", async () => {
    const { time, advance, set, counts } = await setup(true);
    await set(false);
    expect(time.live()).toBe(0);
    expect(counts.invalidations).toBe(1);
    await advance(60 * PERIOD);
    expect(counts.reads).toBe(1);
  });

  test("shown again: a read at once, then the 5 s period again", async () => {
    const { time, advance, set, counts } = await setup(true);
    await set(false);
    await advance(7000);
    await set(true);
    expect(counts.reads).toBe(2);
    await advance(PERIOD);
    expect(counts.reads).toBe(3);
    expect(time.live()).toBe(1);
  });

  test("a repeated visible signal starts no second timer", async () => {
    const { time, set, counts } = await setup(true);
    await set(true);
    expect(time.live()).toBe(1);
    expect(counts.reads).toBe(1);
  });

  test("mounted while hidden (start hidden in the tray): no read until shown", async () => {
    const { time, advance, set, counts } = await setup(false);
    await advance(10 * PERIOD);
    expect(counts.reads).toBe(0);
    expect(time.live()).toBe(0);
    await set(true);
    expect(counts.reads).toBe(1);
  });

  test("stop (unmount): no timer, no subscriber, pending read dropped", async () => {
    const { time, advance, set, page, counts, stop } = await setup(true);
    stop();
    expect(time.live()).toBe(0);
    expect(page.subscribers()).toBe(0);
    expect(counts.invalidations).toBe(1);
    await advance(10 * PERIOD);
    await set(true);
    expect(counts.reads).toBe(1);
  });

  test("only the newest visibility answer decides", async () => {
    let answerFirst: (visible: boolean) => void = () => undefined;
    let calls = 0;
    const listeners = new Set<() => void>();
    const source: VisibilitySource = {
      isVisible: () => {
        calls += 1;
        if (calls === 2) {
          return new Promise((resolve) => (answerFirst = resolve));
        }
        return Promise.resolve(calls < 2);
      },
      subscribe: (onChange) => {
        listeners.add(onChange);
        return () => void listeners.delete(onChange);
      },
    };
    const { time, counts } = poll(source);
    await answered();
    // The period's question stays pending; a newer signal answers hidden first.
    time.advance(PERIOD);
    listeners.forEach((listener) => listener());
    await answered();
    expect(time.live()).toBe(0);
    answerFirst(true);
    await answered();
    expect(counts.reads).toBe(1);
    expect(time.live()).toBe(0);
  });
});

/** A fake Tauri window: visibility, minimized state, focus and shown listeners, a failing read. */
const fakeWindow = () => {
  const state = { visible: true, minimized: false, failing: false, asks: 0 };
  const focusHandlers = new Set<() => void>();
  const shownHandlers = new Set<() => void>();
  const listenTo =
    (handlers: Set<() => void>): WindowPort["onShown"] =>
    async (handler) => {
      handlers.add(handler);
      return () => void handlers.delete(handler);
    };
  const port: WindowPort = {
    isVisible: async () => {
      state.asks += 1;
      if (state.failing) throw new Error("window unavailable");
      return state.visible;
    },
    isMinimized: async () => state.minimized,
    onFocusChanged: listenTo(focusHandlers),
    onShown: listenTo(shownHandlers),
  };
  const focus = () => focusHandlers.forEach((handler) => handler());
  const shown = () => shownHandlers.forEach((handler) => handler());
  const listeners = () => focusHandlers.size + shownHandlers.size;
  return { state, port, focus, shown, listeners };
};

/** Polling bound to a page that stays visible and a fake window, as WebView2 would report it. */
const setupWindow = async () => {
  const win = fakeWindow();
  const warnings: string[] = [];
  const page = simulatedVisibility(true);
  const source = windowVisibility(win.port, (message) =>
    warnings.push(message),
  );
  const run = poll(allVisible([page.source, source]));
  await answered();
  const advance = async (ms: number) => {
    run.time.advance(ms);
    await answered();
  };
  return { ...run, win, page, warnings, advance };
};

// @spec AC-002: Status read on open and every 5 s while visible — .specs/features/001-microphone-priority/spec.md#ac-002
describe("status polling with the Tauri window", () => {
  test("window hidden while the page stays visible: no read, no further window query", async () => {
    const { time, advance, win, counts } = await setupWindow();
    expect(counts.reads).toBe(1);
    win.state.visible = false;
    await advance(PERIOD);
    expect(counts.reads).toBe(1);
    expect(time.live()).toBe(0);
    const asks = win.state.asks;
    await advance(60 * PERIOD);
    expect(counts.reads).toBe(1);
    expect(win.state.asks).toBe(asks);
  });

  test("hidden then blurred: paused without waiting for the next period", async () => {
    const { time, win, counts } = await setupWindow();
    win.state.visible = false;
    win.focus();
    await answered();
    expect(time.live()).toBe(0);
    expect(counts.invalidations).toBe(1);
    expect(counts.reads).toBe(1);
  });

  test("shown and refocused from the tray: a read at once, then every 5 s", async () => {
    const { time, advance, win, counts } = await setupWindow();
    win.state.visible = false;
    await advance(3 * PERIOD);
    win.state.visible = true;
    win.focus();
    await answered();
    expect(counts.reads).toBe(2);
    await advance(PERIOD);
    expect(counts.reads).toBe(3);
    expect(time.live()).toBe(1);
  });

  test("focus changes of a visible window start no second timer and no extra read", async () => {
    const { time, win, counts } = await setupWindow();
    win.focus();
    win.focus();
    await answered();
    expect(counts.reads).toBe(1);
    expect(time.live()).toBe(1);
  });

  test("isVisible fails: the page visibility decides, one warning", async () => {
    const { time, advance, win, page, warnings, counts } = await setupWindow();
    win.state.failing = true;
    for (let period = 0; period < 3; period += 1) await advance(PERIOD);
    expect(counts.reads).toBe(4);
    expect(warnings).toEqual(["Window visibility read failed:"]);
    page.set(false);
    await answered();
    expect(time.live()).toBe(0);
    await advance(10 * PERIOD);
    expect(counts.reads).toBe(4);
  });

  test("shown by the backend without a focus event: a read at once, then every 5 s", async () => {
    const { time, advance, win, counts } = await setupWindow();
    win.state.visible = false;
    await advance(3 * PERIOD);
    expect(counts.reads).toBe(1);
    win.state.visible = true;
    win.shown();
    await answered();
    expect(counts.reads).toBe(2);
    await advance(PERIOD);
    expect(counts.reads).toBe(3);
    expect(time.live()).toBe(1);
  });

  test("minimized while the page stays visible: paused at the next period", async () => {
    const { time, advance, win, counts } = await setupWindow();
    win.state.minimized = true;
    await advance(PERIOD);
    expect(counts.reads).toBe(1);
    expect(time.live()).toBe(0);
    await advance(60 * PERIOD);
    expect(counts.reads).toBe(1);
  });

  test("restored from the taskbar: the focus event reads at once", async () => {
    const { advance, win, counts } = await setupWindow();
    win.state.minimized = true;
    await advance(PERIOD);
    win.state.minimized = false;
    win.focus();
    await answered();
    expect(counts.reads).toBe(2);
  });

  test("stop releases the focus and shown listeners", async () => {
    const { win, stop } = await setupWindow();
    expect(win.listeners()).toBe(2);
    stop();
    expect(win.listeners()).toBe(0);
  });
});
