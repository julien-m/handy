/**
 * Bindings of `useDestinationSupport` (006 AC-007): the first answer, where `null` means the
 * build does not offer destination return (gate closed), and `destination-support-changed`,
 * which keeps an open settings window current. No DOM renderer is installed, so the effects'
 * bodies are tested directly. Run with `bun run test:output-settings`.
 */
import { describe, expect, mock, test } from "bun:test";
import type { DestinationSupport } from "@/bindings";

type Handler = (event: { payload: DestinationSupport }) => void;
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
let answer: () => Promise<DestinationSupport | null> = async () => null;
mock.module("@/bindings", () => ({
  commands: { destinationSupport: () => answer() },
}));
const {
  DESTINATION_SUPPORT_CHANGED_EVENT,
  readSupport,
  supportAfterChange,
  watchSupport,
} = await import("./use-destination-support");
type State = Parameters<typeof supportAfterChange>[0];

const UNREADABLE: DestinationSupport = {
  state: "unsupported",
  reason: "frontmost_unreadable",
};

/** A setter that applies updates to a local state, like React's. */
const holder = (initial: State) => {
  const box = { state: initial };
  const set = (update: (current: State) => State) => {
    box.state = update(box.state);
  };
  return { box, set };
};

const settle = () => new Promise((done) => setTimeout(done, 0));

describe("first answer", () => {
  test("null: this build does not offer the setting", async () => {
    answer = async () => null;
    const { box, set } = holder(undefined);
    readSupport(set);
    await settle();
    expect(box.state).toBe("not_offered");
  });

  test("a failed question is unavailable, never guessed", async () => {
    answer = async () => {
      throw new Error("ipc");
    };
    const { box, set } = holder(undefined);
    readSupport(set);
    await settle();
    expect(box.state).toBeNull();
  });

  test("an answer after unmount is ignored", async () => {
    answer = async () => ({ state: "supported" });
    const { box, set } = holder(undefined);
    readSupport(set)();
    await settle();
    expect(box.state).toBe(undefined);
  });
});

describe("destination-support-changed", () => {
  test("an open window follows a change of the runtime support", () => {
    const { box, set } = holder({ state: "supported" });
    const stop = watchSupport(set);
    const listener = listeners[listeners.length - 1];
    expect(listener?.event).toBe(DESTINATION_SUPPORT_CHANGED_EVENT);
    listener?.handler({ payload: UNREADABLE });
    expect(box.state).toEqual(UNREADABLE);
    listener?.handler({ payload: { state: "supported" } });
    expect(box.state).toEqual({ state: "supported" });
    stop();
  });

  test("unmounting stops listening", async () => {
    const before = stopped;
    watchSupport(() => undefined)();
    await settle();
    expect(stopped).toBe(before + 1);
  });

  test("a build without the setting stays hidden whatever the event says", () => {
    expect(supportAfterChange("not_offered", { state: "supported" })).toBe(
      "not_offered",
    );
  });

  test("a change before the first answer waits for that answer", () => {
    expect(supportAfterChange(undefined, UNREADABLE)).toBe(undefined);
    expect(supportAfterChange(null, UNREADABLE)).toEqual(UNREADABLE);
  });
});
