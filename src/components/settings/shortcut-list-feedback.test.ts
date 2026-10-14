// Tests of what a shortcut list shows after an edit, of inherited conflict
// warnings, hints and contract test ids; run with
// `bun run test:shortcut-states`.
import { describe, expect, test } from "bun:test";
import type { StoredShortcutConflict } from "@/bindings";
import {
  type MessageNames,
  UNEXPECTED_FEEDBACK,
  feedbackAfterEdit,
  legacyWarningsOf,
  listHint,
  shortcutTestIds,
  systemRefusalTitle,
} from "./shortcut-list-feedback";

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
    expect(dictation.limit).toBe("shortcut-settings-limit-dictation-limit");
    expect(dictation.legacy).toBe(
      "shortcut-settings-conflict-dictation-legacy",
    );
    expect(dictation.system).toBe("shortcut-settings-conflict-dictation-os");
    expect(shortcutTestIds("transcribe_with_post_process").last).toBe(
      "shortcut-settings-ai-dictation-last",
    );
    expect(shortcutTestIds("cancel").shortcut).toBe(
      "shortcut-settings-cancel-shortcut",
    );
  });
});
