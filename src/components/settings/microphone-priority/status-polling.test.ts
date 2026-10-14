/**
 * Polling of the microphone status (feature 001, AC-002) on a simulated clock and a simulated
 * page visibility: a read on open, one every 5 s while visible, none while hidden, a read at once
 * when shown again, nothing after stop. Run with `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import {
  startStatusPolling,
  type PollClock,
  type VisibilitySource,
} from "./status-polling";

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
    isVisible: () => visible,
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

const setup = (visible: boolean) => {
  const time = simulatedClock();
  const page = simulatedVisibility(visible);
  const counts = { reads: 0, invalidations: 0 };
  const stop = startStatusPolling({
    read: () => void (counts.reads += 1),
    invalidate: () => void (counts.invalidations += 1),
    clock: time.clock,
    visibility: page.source,
    periodMs: PERIOD,
  });
  return { time, page, counts, stop };
};

// @spec AC-002: Status read on open and every 5 s while visible — .specs/features/001-microphone-priority/spec.md#ac-002
describe("status polling", () => {
  test("reads when opened, then every 5 seconds", () => {
    const { time, counts } = setup(true);
    expect(counts.reads).toBe(1);
    time.advance(PERIOD - 1);
    expect(counts.reads).toBe(1);
    time.advance(1);
    expect(counts.reads).toBe(2);
    time.advance(3 * PERIOD);
    expect(counts.reads).toBe(5);
  });

  test("hidden: the timer stops and the pending read is dropped", () => {
    const { time, page, counts } = setup(true);
    page.set(false);
    expect(time.live()).toBe(0);
    expect(counts.invalidations).toBe(1);
    time.advance(60 * PERIOD);
    expect(counts.reads).toBe(1);
  });

  test("shown again: a read at once, then the 5 s period again", () => {
    const { time, page, counts } = setup(true);
    page.set(false);
    time.advance(7000);
    page.set(true);
    expect(counts.reads).toBe(2);
    time.advance(PERIOD);
    expect(counts.reads).toBe(3);
    expect(time.live()).toBe(1);
  });

  test("a repeated visible signal starts no second timer", () => {
    const { time, page, counts } = setup(true);
    page.set(true);
    expect(time.live()).toBe(1);
    expect(counts.reads).toBe(1);
  });

  test("mounted while hidden (start hidden in the tray): no read until shown", () => {
    const { time, page, counts } = setup(false);
    time.advance(10 * PERIOD);
    expect(counts.reads).toBe(0);
    expect(time.live()).toBe(0);
    page.set(true);
    expect(counts.reads).toBe(1);
  });

  test("stop (unmount): no timer, no subscriber, pending read dropped", () => {
    const { time, page, counts, stop } = setup(true);
    stop();
    expect(time.live()).toBe(0);
    expect(page.subscribers()).toBe(0);
    expect(counts.invalidations).toBe(1);
    time.advance(10 * PERIOD);
    page.set(true);
    expect(counts.reads).toBe(1);
  });
});
