/**
 * Delayed menu closes of the indicator (005 prompt menu, 006 destination menu): one pending
 * action at most, cancelled by a newer one, by the component's cleanup (unmount or a new
 * dictation) and never run after it. No DOM renderer is installed, so the effect bodies are
 * tested directly. Run with `bun run test:indicator`.
 */
import { describe, expect, mock, test } from "bun:test";

type Handler = (event: { payload: string }) => void;
const listeners: Array<{ event: string; handler: Handler }> = [];
let stopped = 0;
mock.module("@tauri-apps/api/event", () => ({
  listen: async (event: string, handler: Handler) => {
    listeners.push({ event, handler });
    return () => {
      stopped += 1;
    };
  },
}));
const {
  SHOW_OVERLAY_EVENT,
  cancelOnCleanup,
  createMenuTimer,
  watchDictationStarts,
} = await import("./menu-timer");

const wait = (ms: number) => new Promise((done) => setTimeout(done, ms));

describe("menu timer", () => {
  test("a scheduled close runs after its delay", async () => {
    const timer = createMenuTimer();
    let closed = 0;
    timer.schedule(() => (closed += 1), 5);
    await wait(20);
    expect(closed).toBe(1);
  });

  test("a newer close replaces the pending one", async () => {
    const timer = createMenuTimer();
    const runs: string[] = [];
    timer.schedule(() => runs.push("first"), 5);
    timer.schedule(() => runs.push("second"), 5);
    await wait(20);
    expect(runs).toEqual(["second"]);
  });

  test("unmount or a new dictation: the cleanup cancels the pending close", async () => {
    const timer = createMenuTimer();
    let closed = 0;
    timer.schedule(() => (closed += 1), 5);
    // `useMenuTimer` returns this from its effect, run on unmount and on a dictation change.
    cancelOnCleanup(timer)();
    await wait(20);
    expect(closed).toBe(0);
  });

  test("the pointer coming back cancels the close; a later one still works", async () => {
    const timer = createMenuTimer();
    let closed = 0;
    timer.schedule(() => (closed += 1), 5);
    timer.cancel();
    timer.schedule(() => (closed += 1), 5);
    await wait(20);
    expect(closed).toBe(1);
  });
});

describe("dictation starts", () => {
  test("each recording or Live show is a new dictation; working states are not", () => {
    let starts = 0;
    const stop = watchDictationStarts(() => (starts += 1));
    const listener = listeners[listeners.length - 1];
    expect(listener.event).toBe(SHOW_OVERLAY_EVENT);
    for (const state of [
      "recording",
      "transcribing",
      "processing",
      "streaming",
    ]) {
      listener.handler({ payload: state });
    }
    expect(starts).toBe(2);
    stop();
  });

  test("unmounting stops listening", async () => {
    const before = stopped;
    watchDictationStarts(() => undefined)();
    await wait(0);
    expect(stopped).toBe(before + 1);
  });
});
