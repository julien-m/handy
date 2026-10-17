// Tests of what a shortcut list shows after an edit, of inherited conflict
// warnings, hints and contract test ids; run with
// `bun run test:shortcut-states`.
import { describe, expect, test } from "bun:test";
import type { StoredShortcutConflict } from "@/bindings";
import { CLOSED_DRAFT, draftReducer } from "./shortcut-draft-model";
import {
  type MessageNames,
  UNEXPECTED_FEEDBACK,
  describeStartFailure,
  draftSystemRefusal,
  feedbackAfterEdit,
  isSameEdit,
  legacyWarningsOf,
  listHint,
  shortcutInstanceId,
  shortcutTestIds,
  systemRefusalTitle,
} from "./shortcut-list-feedback";
import { shortcutScreenOf } from "./shortcut-screen";

const LIST_KEY = "settings.general.shortcut.list";
const names: MessageNames = {
  actionName: (id) => `name:${id}`,
  shortcutLabel: (shortcut) => `label:${shortcut}`,
};
const ADD = { kind: "add", shortcut: "shift+option+space" } as const;

describe("feedback after an edit", () => {
  test("a saved edit is a success with no message", () => {
    expect(feedbackAfterEdit(ADD, null, names)).toEqual({
      state: "success",
      error: null,
      systemRefusal: null,
    });
  });

  test("a conflict names the candidate and its owner, nothing saved", () => {
    const refusal = {
      code: "shortcut_conflict",
      conflict: {
        kind: "duplicate",
        owning_action: "transcribe_with_post_process",
        owning_shortcut: "option+shift+space",
      },
    } as const;
    const feedback = feedbackAfterEdit(ADD, refusal, names);
    expect(feedback.state).toBe("error");
    expect(feedback.error).toEqual({
      key: `${LIST_KEY}.draftConflict`,
      params: {
        shortcut: "label:shift+option+space",
        action: "name:transcribe_with_post_process",
      },
    });
  });

  test("a system refusal is an alert with the edit to retry", () => {
    const refusal = { code: "registration_failed", message: "taken" } as const;
    const feedback = feedbackAfterEdit(ADD, refusal, names);
    expect(feedback.error).toBeNull();
    expect(feedback.systemRefusal).toEqual({
      shortcut: "shift+option+space",
      reason: "taken",
      edit: ADD,
    });
  });

  test("a refused reset has no candidate shortcut", () => {
    const refusal = { code: "registration_failed", message: "taken" } as const;
    const feedback = feedbackAfterEdit({ kind: "reset" }, refusal, names);
    expect(feedback.systemRefusal?.shortcut).toBeNull();
  });

  test("other refusals keep their existing message", () => {
    const feedback = feedbackAfterEdit(ADD, { code: "last_shortcut" }, names);
    expect(feedback.error?.key).toBe(`${LIST_KEY}.removeLast`);
    expect(UNEXPECTED_FEEDBACK.error?.key).toBe(`${LIST_KEY}.errors.generic`);
  });
});

describe("inherited conflicts", () => {
  const conflicts: StoredShortcutConflict[] = [
    {
      action_id: "transcribe",
      shortcut: "ctrl+option+d",
      conflict: {
        kind: "modifier_overlap",
        owning_action: "cancel",
        owning_shortcut: "ctrl+option",
      },
    },
    {
      action_id: "cancel",
      shortcut: "ctrl+option",
      conflict: {
        kind: "modifier_overlap",
        owning_action: "transcribe",
        owning_shortcut: "ctrl+option+d",
      },
    },
  ];

  test("each involved action gets its own warning, cancel included", () => {
    const dictation = legacyWarningsOf(conflicts, "transcribe", names);
    expect(dictation).toEqual([
      {
        shortcut: "ctrl+option+d",
        title: {
          key: `${LIST_KEY}.legacyOverlap`,
          params: { shortcut: "label:ctrl+option+d", action: "name:cancel" },
        },
      },
    ]);
    const cancel = legacyWarningsOf(conflicts, "cancel", names);
    expect(cancel.map((warning) => warning.shortcut)).toEqual(["ctrl+option"]);
    expect(
      legacyWarningsOf(conflicts, "transcribe_with_post_process", names),
    ).toEqual([]);
  });
});

describe("hints, titles and test ids", () => {
  test("hint for the last shortcut and for the limit", () => {
    expect(listHint(1)).toBe("last");
    expect(listHint(2)).toBeNull();
    expect(listHint(4)).toBe("limit");
  });

  test("a system refusal names the system and the shortcut", () => {
    expect(systemRefusalTitle("macos", "⌘ ⇧ 5")).toEqual({
      key: `${LIST_KEY}.systemRefused`,
      params: { system: "macOS", shortcut: "⌘ ⇧ 5" },
    });
    expect(systemRefusalTitle("unknown", "F5").key).toBe(
      `${LIST_KEY}.systemRefusedGeneric`,
    );
  });

  test("test ids follow the shortcut-settings contract", () => {
    const dictation = shortcutTestIds("transcribe");
    expect(dictation.list).toBe("shortcut-settings-dictation-list");
    expect(dictation.recording).toBe("shortcut-settings-dictation-recording");
    const limit = shortcutTestIds("transcribe", "shortcut-settings-limit");
    expect(limit.limit).toBe("shortcut-settings-limit-dictation-limit");
    expect(limit.list).toBe("shortcut-settings-limit-dictation-list");
    expect(limit.add).toBe("shortcut-settings-limit-dictation-add");
    const conflict = shortcutTestIds(
      "transcribe",
      "shortcut-settings-conflict",
    );
    expect(conflict.legacy).toBe("shortcut-settings-conflict-dictation-legacy");
    expect(conflict.system).toBe("shortcut-settings-conflict-dictation-os");
    expect(conflict.error).toBe(
      "shortcut-settings-conflict-dictation-conflict",
    );
    expect(conflict.shortcut).toBe(
      "shortcut-settings-conflict-dictation-shortcut",
    );
    expect(conflict.recording).toBe(
      "shortcut-settings-conflict-dictation-recording",
    );
    expect(shortcutInstanceId("option+space")).toBe("shortcut-option+space");
    expect(shortcutTestIds("transcribe_with_post_process").last).toBe(
      "shortcut-settings-ai-dictation-last",
    );
    expect(shortcutTestIds("cancel").shortcut).toBe(
      "shortcut-settings-cancel-shortcut",
    );
  });
});

describe("draft system refusal (Try again)", () => {
  const SYSTEM = {
    code: "registration_failed",
    message: "HotKey already registered",
  } as const;
  const refused = [
    { type: "open" },
    { type: "captured", shortcut: "cmd+shift+5" },
    { type: "checked", shortcut: "cmd+shift+5", refusal: null },
    { type: "commit" },
    { type: "commitFailed", refusal: SYSTEM },
  ] as const;
  const refusedDraft = refused.reduce(draftReducer, CLOSED_DRAFT);

  test("a draft refused by the system keeps its alert and retry", () => {
    expect(draftSystemRefusal(refusedDraft)).toEqual({
      shortcut: "cmd+shift+5",
      reason: "HotKey already registered",
      edit: { kind: "add", shortcut: "cmd+shift+5" },
    });
  });

  test("the alert ends once the draft is recorded again or discarded", () => {
    const again = draftReducer(refusedDraft, {
      type: "captured",
      shortcut: "f13",
    });
    expect(draftSystemRefusal(again)).toBeNull();
    const gone = draftReducer(refusedDraft, { type: "discard" });
    expect(draftSystemRefusal(gone)).toBeNull();
  });

  test("a validation refusal is not a system refusal", () => {
    const invalid = draftReducer(
      draftReducer(draftReducer(CLOSED_DRAFT, { type: "open" }), {
        type: "captured",
        shortcut: "f13",
      }),
      {
        type: "checked",
        shortcut: "f13",
        refusal: { code: "invalid_shortcut", message: "bad" },
      },
    );
    expect(draftSystemRefusal(invalid)).toBeNull();
  });

  test("the draft's own Add is recognized among the edits", () => {
    const edit = { kind: "add", shortcut: "cmd+shift+5" } as const;
    expect(isSameEdit(edit, { ...edit })).toBe(true);
    expect(isSameEdit(edit, { kind: "add", shortcut: "f13" })).toBe(false);
    expect(isSameEdit(edit, undefined)).toBe(false);
  });
});

describe("recorder start failures", () => {
  test("Secure Input and other failures have their own reason", () => {
    expect(describeStartFailure({ kind: "secure_input" })).toEqual({
      key: "secureInput.recorderBlocked",
      params: {},
    });
    expect(describeStartFailure({ kind: "failed", message: "boom" })).toEqual({
      key: "settings.general.shortcut.errors.set",
      params: { error: "boom" },
    });
  });
});

describe("screen shown by the shortcut lists", () => {
  test("a conflict wins over a full list, which wins over the base screen", () => {
    expect(shortcutScreenOf([null, null])).toBe("shortcut-settings");
    expect(shortcutScreenOf(["shortcut-settings-limit", null])).toBe(
      "shortcut-settings-limit",
    );
    expect(
      shortcutScreenOf([
        "shortcut-settings-limit",
        "shortcut-settings-conflict",
      ]),
    ).toBe("shortcut-settings-conflict");
  });
});
