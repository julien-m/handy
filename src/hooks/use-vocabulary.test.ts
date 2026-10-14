/**
 * The bindings of `useVocabulary` against a stubbed editor store: a settings revision rereads the
 * list, the status and (paste box open) the preview; `model-state-changed` rereads the status;
 * the paste preview waits for typing to pause. No DOM renderer is installed, so the effects'
 * bodies are tested directly; the hook only passes them to `useEffect`.
 * Run with `bun run test:vocabulary`.
 */
import assert from "node:assert/strict";
import { describe, mock, test } from "bun:test";
import type { VocabularyStore } from "../stores/vocabulary-store";

type Handler = () => void;
const listeners: Array<{ event: string; handler: Handler }> = [];
let stopped = 0;
// The real modules need Tauri IPC; the bindings under test only need these.
mock.module("@tauri-apps/api/event", () => ({
  listen: async (event: string, handler: Handler) => {
    listeners.push({ event, handler });
    return () => {
      stopped += 1;
    };
  },
}));
mock.module("@/bindings", () => ({ commands: {} }));
mock.module("../stores/settingsStore", () => ({
  useSettingsStore: () => 0,
}));
const {
  PREVIEW_DELAY_MS,
  refreshVocabulary,
  schedulePreview,
  watchModelState,
} = await import("./use-vocabulary");

/** Store stub that records the reads the bindings trigger. */
const stubStore = (pasteOpen: boolean) => {
  const calls: string[] = [];
  const read = (name: string) => () => {
    calls.push(name);
    return Promise.resolve();
  };
  const state = {
    pasteOpen,
    loadList: read("list"),
    loadStatus: read("status"),
    loadPreview: read("preview"),
  };
  const store = { getState: () => state } as unknown as VocabularyStore;
  return { store, calls };
};

const wait = (ms: number) => new Promise((done) => setTimeout(done, ms));

describe("settings revision", () => {
  test("rereads the list and the status", () => {
    const { store, calls } = stubStore(false);
    refreshVocabulary(store);
    assert.deepEqual(calls, ["list", "status"]);
  });

  test("also rereads the preview while the paste box is open", () => {
    const { store, calls } = stubStore(true);
    refreshVocabulary(store);
    assert.deepEqual(calls, ["list", "status", "preview"]);
  });
});

describe("model state", () => {
  test("a model load or unload rereads the status, and cleanup stops listening", async () => {
    const { store, calls } = stubStore(false);
    const stop = watchModelState(store);
    const listener = listeners[listeners.length - 1];
    assert.equal(listener?.event, "model-state-changed");
    listener?.handler();
    assert.deepEqual(calls, ["status"]);
    stop();
    await wait(0);
    assert.equal(stopped, 1);
  });
});

describe("paste preview debounce", () => {
  test("waits 200 ms after the last keystroke and asks once", async () => {
    assert.equal(PREVIEW_DELAY_MS, 200);
    const { store, calls } = stubStore(true);
    const cancelFirst = schedulePreview(store, 30);
    await wait(10);
    cancelFirst(); // the next keystroke replaces the pending request
    schedulePreview(store, 30);
    await wait(10);
    assert.deepEqual(calls, []);
    await wait(40);
    assert.deepEqual(calls, ["preview"]);
  });
});
