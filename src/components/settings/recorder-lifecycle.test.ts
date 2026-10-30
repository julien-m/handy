// Backend side of the recorders when their chip disappears (004): a start
// that answers after the chip unmounted is undone, and a Tauri chip that
// unmounts while it holds the shortcuts suspended resumes them. Run with
// `bun run test:shortcut-states`.
import { beforeEach, describe, expect, mock, test } from "bun:test";

const calls: string[] = [];
let startAnswer: { status: "ok" | "error"; error?: string } = { status: "ok" };
mock.module("@/bindings", () => ({
  commands: {
    startHandyKeysRecording: async () => {
      calls.push("start");
      return startAnswer;
    },
    stopHandyKeysRecording: async () => {
      calls.push("stop");
    },
    suspendAllBindings: async () => {
      calls.push("suspend");
    },
    resumeAllBindings: async () => {
      calls.push("resume");
    },
  },
}));
const { resumeOnUnmount, startHandyKeys, suspendForCapture } = await import(
  "./recorder-lifecycle"
);

const settle = () => new Promise((done) => setTimeout(done, 0));

beforeEach(() => {
  calls.length = 0;
  startAnswer = { status: "ok" };
});

describe("HandyKeys start answered after the chip disappeared", () => {
  test("a mounted chip records", async () => {
    expect(await startHandyKeys("transcribe", () => true)).toEqual({
      status: "recording",
    });
    expect(calls).toEqual(["start"]);
  });

  test("an unmounted chip stops the backend recording at once", async () => {
    // Blocked draft: the chip is clicked again, then Discard before the answer.
    expect(await startHandyKeys("transcribe", () => false)).toEqual({
      status: "abandoned",
    });
    expect(calls).toEqual(["start", "stop"]);
  });

  test("a refused start reports its reason and stops nothing", async () => {
    startAnswer = { status: "error", error: "secure-input-active" };
    expect(await startHandyKeys("transcribe", () => false)).toEqual({
      status: "refused",
      failure: { kind: "secure_input" },
    });
    startAnswer = { status: "error", error: "listener down" };
    expect(await startHandyKeys("transcribe", () => true)).toEqual({
      status: "refused",
      failure: { kind: "failed", message: "listener down" },
    });
    expect(calls).toEqual(["start", "start"]);
  });
});

describe("Tauri recorder and the suspended shortcuts", () => {
  test("a suspension answered after unmount is resumed", async () => {
    expect(await suspendForCapture(() => false)).toEqual({
      status: "abandoned",
    });
    expect(calls).toEqual(["suspend", "resume"]);
  });

  test("unmounting while recording resumes the shortcuts once", async () => {
    const isSuspending = { current: true };
    const cleanup = resumeOnUnmount(isSuspending);
    cleanup();
    cleanup();
    await settle();
    expect(calls).toEqual(["resume"]);
    expect(isSuspending.current).toBe(false);
  });

  test("unmounting an idle chip resumes nothing", async () => {
    resumeOnUnmount({ current: false })();
    await settle();
    expect(calls).toEqual([]);
  });
});
