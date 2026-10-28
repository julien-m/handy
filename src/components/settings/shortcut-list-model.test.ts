// Assertions for the shortcut list rules; run with
// `bun src/components/settings/shortcut-list-model.test.ts`.
import assert from "node:assert/strict";
import type {
  RefusedShortcut,
  ShortcutBinding,
  StoredShortcutConflict,
} from "@/bindings";
import {
  MAX_SHORTCUTS_PER_ACTION,
  PROMPT_SHORTCUT_LIMITS,
  canAddShortcut,
  canRemoveShortcut,
  describeEditError,
  keepStoredRefusals,
  limitsFor,
  promptIdOf,
  promptNameOfTarget,
  promptTargetOf,
  refusalFor,
  shortcutsOf,
  storedConflictFor,
} from "./shortcut-list-model";

const binding = (additional?: string[]): ShortcutBinding => ({
  id: "transcribe",
  name: "Transcribe",
  description: "Converts your speech into text.",
  default_binding: "option+space",
  current_binding: "option+space",
  additional_bindings: additional,
});

const actionName = (actionId: string): string => `name:${actionId}`;

// A legacy store has no alternatives field.
assert.deepEqual(shortcutsOf(binding()), ["option+space"]);
assert.deepEqual(shortcutsOf(binding(["ctrl+option+d"])), [
  "option+space",
  "ctrl+option+d",
]);

// Bounds: 1 to 4 shortcuts per action.
assert.equal(MAX_SHORTCUTS_PER_ACTION, 4);
assert.equal(canAddShortcut(3), true);
assert.equal(canAddShortcut(4), false);
assert.equal(canRemoveShortcut(1), false);
assert.equal(canRemoveShortcut(2), true);

// Conflicts name the owning action.
assert.deepEqual(
  describeEditError(
    {
      code: "shortcut_conflict",
      conflict: {
        kind: "duplicate",
        owning_action: "transcribe_with_post_process",
        owning_shortcut: "option+shift+space",
      },
    },
    actionName,
  ),
  {
    key: "settings.general.shortcut.list.errors.conflict",
    params: {
      action: "name:transcribe_with_post_process",
      shortcut: "option+shift+space",
    },
  },
);
assert.equal(
  describeEditError(
    {
      code: "shortcut_conflict",
      conflict: {
        kind: "modifier_overlap",
        owning_action: "transcribe",
        owning_shortcut: "option",
      },
    },
    actionName,
  ).key,
  "settings.general.shortcut.list.errors.overlap",
);
assert.deepEqual(
  describeEditError({ code: "limit_exceeded", limit: 4 }, actionName),
  {
    key: "settings.general.shortcut.list.addLimit",
    params: { limit: 4 },
  },
);
assert.equal(
  describeEditError({ code: "last_shortcut" }, actionName).key,
  "settings.general.shortcut.list.removeLast",
);
assert.deepEqual(
  describeEditError(
    { code: "registration_failed", message: "HotKey already registered" },
    actionName,
  ),
  {
    key: "settings.general.shortcut.list.errors.registration",
    params: { error: "HotKey already registered" },
  },
);

const conflicts: StoredShortcutConflict[] = [
  {
    action_id: "transcribe",
    shortcut: "alt+space",
    conflict: {
      kind: "duplicate",
      owning_action: "transcribe_with_post_process",
      owning_shortcut: "option+space",
    },
  },
];
assert.equal(
  storedConflictFor(conflicts, "transcribe", "alt+space"),
  conflicts[0],
);
assert.equal(storedConflictFor(conflicts, "transcribe", "f13"), undefined);

// Enabling post-processing succeeds although the system refuses one of its
// shortcuts: that row carries a warning and stays editable.
const refused: RefusedShortcut[] = [
  {
    action_id: "transcribe_with_post_process",
    shortcut: "f14",
    reason: "HotKey already registered",
  },
];
assert.deepEqual(refusalFor(refused, "transcribe_with_post_process", "f14"), {
  key: "settings.general.shortcut.list.errors.registration",
  params: { error: "HotKey already registered" },
});
assert.equal(
  refusalFor(refused, "transcribe_with_post_process", "f13"),
  undefined,
);
assert.equal(refusalFor(refused, "transcribe", "f14"), undefined);

// A refused shortcut that was changed or removed loses its warning.
const postBinding = (shortcuts: string[]): ShortcutBinding => ({
  ...binding(shortcuts.slice(1)),
  id: "transcribe_with_post_process",
  current_binding: shortcuts[0],
});
assert.deepEqual(
  keepStoredRefusals(refused, {
    transcribe_with_post_process: postBinding(["f13", "f14"]),
  }),
  refused,
);
assert.deepEqual(
  keepStoredRefusals(refused, {
    transcribe_with_post_process: postBinding(["f13", "f15"]),
  }),
  [],
);
assert.deepEqual(keepStoredRefusals(refused, {}), []);

// A refusal of the settings service is translated from its typed code in the
// list response.
const persistenceFailed = {
  code: "persistence_failed",
  message: "Settings persistence failed.",
  field: null,
  reason: null,
  outcome: null,
  current_revision: null,
} as const;
assert.deepEqual(
  describeEditError(
    { code: "settings_refused", error: persistenceFailed },
    actionName,
  ),
  { key: "errors.settings.persistenceFailed", params: {} },
);

// @spec AC-006: Prompt targets keep 0–2 shortcuts — .specs/features/005-custom-ai-prompts/spec.md#ac-006
// Prompt targets (005): `prompt:<id>`, 0 to 2 shortcuts, the last one
// removable, no default to reset to.
assert.equal(promptTargetOf("abc"), "prompt:abc");
assert.equal(promptIdOf("prompt:abc"), "abc");
assert.equal(promptIdOf("transcribe"), null);
assert.deepEqual(limitsFor("prompt:abc"), PROMPT_SHORTCUT_LIMITS);
assert.deepEqual(limitsFor("transcribe"), {
  max: MAX_SHORTCUTS_PER_ACTION,
  min: 1,
  resettable: true,
});
assert.equal(canAddShortcut(1, PROMPT_SHORTCUT_LIMITS), true);
assert.equal(canAddShortcut(2, PROMPT_SHORTCUT_LIMITS), false);
assert.equal(canRemoveShortcut(1, PROMPT_SHORTCUT_LIMITS), true);
assert.equal(canRemoveShortcut(0, PROMPT_SHORTCUT_LIMITS), false);
assert.equal(PROMPT_SHORTCUT_LIMITS.resettable, false);
// A prompt target left without shortcuts has an empty first shortcut.
assert.deepEqual(
  shortcutsOf({ ...binding(), id: "prompt:abc", current_binding: "" }),
  [],
);
// Messages name a prompt target after its prompt.
const prompts = [{ id: "abc", name: "Notes" }];
assert.equal(promptNameOfTarget(prompts, "prompt:abc"), "Notes");
assert.equal(promptNameOfTarget(prompts, "prompt:gone"), "gone");
assert.equal(promptNameOfTarget(prompts, "transcribe"), null);
assert.deepEqual(
  describeEditError({ code: "limit_exceeded", limit: 2 }, actionName),
  { key: "settings.general.shortcut.list.addLimit", params: { limit: 2 } },
);

console.log("shortcut-list-model: all assertions passed");
