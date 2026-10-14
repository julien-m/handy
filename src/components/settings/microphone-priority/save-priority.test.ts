/**
 * Save of the microphone priority editor (feature 001) against a stubbed command: the complete
 * order goes out once with its base revision; success rereads the settings before dropping the
 * draft; a conflict keeps the draft and rereads; a committed-but-not-applied value is reported; an
 * unknown outcome is confirmed when the reread shows the sent order, else kept as a refusal.
 * Run with `bun run test:microphone-priority`.
 */
import { describe, expect, mock, test } from "bun:test";
import type { SettingsCommandError } from "@/bindings";

const calls: Array<[string[], number | null]> = [];
let reply: unknown = { status: "ok", data: 9 };
mock.module("@/bindings", () => ({
  commands: {
    setMicrophonePriority: (order: string[], revision: number | null) => {
      calls.push([order, revision]);
      return reply instanceof Error
        ? Promise.reject(reply)
        : Promise.resolve(reply);
    },
  },
}));
// The hook module's other imports need Tauri IPC, Vite globals and React; these tests only use
// its pure write helpers.
mock.module("@/hooks/useSettings", () => ({ useSettings: () => ({}) }));
mock.module("@/hooks/use-microphone-status", () => ({
  useMicrophoneStatus: () => null,
}));
mock.module("@/stores/settingsStore", () => ({ useSettingsStore: () => null }));
mock.module("sonner", () => ({ toast: {} }));
mock.module("react-i18next", () => ({ useTranslation: () => ({ t: String }) }));
const { settleWrite, writePriority } = await import("./use-priority-editor");
const { EMPTY_DRAFT, SAVED_DRAFT, editOrder, startSave } = await import(
  "./draft-state"
);
type Draft = typeof EMPTY_DRAFT;
const json = (value: unknown) => JSON.stringify(value);

const conflict: SettingsCommandError = {
  code: "revision_conflict",
  message: "Settings changed since the expected revision.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: 6,
};

/** Effects that record their calls, in order; `committed` is the order a reread finds. */
const recorder = (committed: readonly string[] | null = null) => {
  const log: string[] = [];
  return {
    log,
    effects: {
      refreshSettings: async () => void log.push("refreshSettings"),
      committedOrder: () => committed,
      refreshStatus: async () => void log.push("refreshStatus"),
      notify: (key: string, kind: string) => void log.push(`${kind}:${key}`),
      focusStable: () => void log.push("focusStable"),
    },
  };
};

/** Apply draft updates the way React would, starting from `initial`. */
const draftHolder = (initial: Draft) => {
  let draft = initial;
  const setDraft = (next: Draft | ((current: Draft) => Draft)) => {
    draft = typeof next === "function" ? next(draft) : next;
  };
  return { setDraft, current: () => draft };
};

// @spec AC-007: One validated write with the base revision; conflicts keep the draft — .specs/features/001-microphone-priority/spec.md#ac-007
describe("save", () => {
  const saved = { order: ["USB headset", "MacBook"], revision: 4 };
  const sent = ["MacBook", "USB headset"];
  const editing = startSave(editOrder(EMPTY_DRAFT, saved, sent));
  const unknown: SettingsCommandError = {
    ...conflict,
    code: "unavailable",
    outcome: "unknown",
  };

  test("sends the complete order once with the base revision", async () => {
    calls.length = 0;
    reply = { status: "ok", data: 5 };
    const outcome = await writePriority(["MacBook", "USB headset"], 4);
    expect(outcome.ok).toBe(true);
    expect(JSON.stringify(calls)).toBe(
      JSON.stringify([[["MacBook", "USB headset"], 4]]),
    );
  });

  test("success rereads before dropping the draft, marks it saved, moves focus, then confirms", async () => {
    const { log, effects } = recorder();
    const holder = draftHolder(editing);
    await settleWrite({ ok: true }, sent, holder.setDraft, effects);
    expect(holder.current()).toBe(SAVED_DRAFT);
    expect(holder.current().savePhase).toBe("saved");
    expect(log.join()).toBe(
      "refreshSettings,focusStable,success:settings.sound.microphonePriority.draft.saved,refreshStatus",
    );
  });

  test("a stale revision keeps the draft, raises the conflict and rereads", async () => {
    reply = { status: "error", error: conflict };
    const outcome = await writePriority(["MacBook", "USB headset"], 4);
    const { log, effects } = recorder();
    const holder = draftHolder(editing);
    await settleWrite(outcome, sent, holder.setDraft, effects);
    expect(holder.current().conflict).toBe(true);
    expect(JSON.stringify(holder.current().order)).toBe(
      JSON.stringify(["MacBook", "USB headset"]),
    );
    // The bar stays, so does the focused Save: no focus move.
    expect(log.join()).toBe("refreshSettings");
  });

  test("a stored value whose live effect failed is reported, not kept as a draft", async () => {
    const { log, effects } = recorder();
    const holder = draftHolder(editing);
    const error = {
      ...conflict,
      code: "internal_error" as const,
      outcome: "committed" as const,
    };
    await settleWrite({ ok: false, error }, sent, holder.setDraft, effects);
    expect(holder.current()).toBe(EMPTY_DRAFT);
    expect(log.join()).toBe(
      "refreshSettings,focusStable,error:errors.settings.committedNotApplied,refreshStatus",
    );
  });

  test("a thrown command error is an untyped failure with a retry", async () => {
    reply = new Error("ipc");
    const outcome = await writePriority(["MacBook"], 4);
    const holder = draftHolder(editing);
    await settleWrite(outcome, sent, holder.setDraft, recorder().effects);
    expect(holder.current().savePhase).toBe("failed");
    expect(holder.current().saveError?.key).toBe("errors.settings.generic");
  });

  test("an unknown outcome whose order is committed is confirmed like a success", async () => {
    const { log, effects } = recorder(sent);
    const holder = draftHolder(editing);
    await settleWrite(
      { ok: false, error: unknown },
      sent,
      holder.setDraft,
      effects,
    );
    expect(holder.current()).toBe(SAVED_DRAFT);
    expect(log.join()).toBe(
      "refreshSettings,focusStable,success:settings.sound.microphonePriority.draft.saved,refreshStatus",
    );
  });

  test("an unknown outcome whose order is not committed keeps the draft with Retry", async () => {
    const { log, effects } = recorder(saved.order);
    const holder = draftHolder(editing);
    await settleWrite(
      { ok: false, error: unknown },
      sent,
      holder.setDraft,
      effects,
    );
    expect(holder.current().savePhase).toBe("failed");
    expect(json(holder.current().order)).toBe(json(sent));
    // Reread once, before deciding; the bar and the focused Save (now Retry) stay.
    expect(log.join()).toBe("refreshSettings");
  });
});
