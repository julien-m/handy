/**
 * Draft rules of the microphone priority editor (feature 001): edits change only the draft,
 * duplicates are refused, emptying the list changes only the draft, a change saved elsewhere raises the
 * conflict and keeps the draft, and typed refusals are translated.
 * Run with `bun run test:microphone-priority`.
 */
import { describe, expect, test } from "bun:test";
import type { SettingsCommandError } from "@/bindings";
import {
  EMPTY_DRAFT,
  PRIORITY_KEYS,
  addChoice,
  canSave,
  editOrder,
  failSave,
  isDirty,
  moveChoice,
  orderError,
  rebaseDraft,
  removeChoice,
  saveErrorMessage,
  shownOrder,
  startSave,
  type PriorityDraft,
  type SavedPriority,
} from "./draft-state";

const saved: SavedPriority = { order: ["USB headset", "MacBook"], revision: 4 };
const json = (value: unknown) => JSON.stringify(value);

const refusal = (
  overrides: Partial<SettingsCommandError>,
): SettingsCommandError => ({
  code: "validation_failed",
  message: "Setting value failed validation.",
  field: "microphone_priority",
  reason: "canonical_value",
  outcome: null,
  current_revision: null,
  ...overrides,
});

const edited = (order: string[]): PriorityDraft =>
  editOrder(EMPTY_DRAFT, saved, order);

// @spec AC-001: Draft edits, duplicate refusal and Save guard — .specs/features/001-microphone-priority/spec.md#ac-001
describe("draft edits", () => {
  test("a reorder changes only the draft and records its base", () => {
    const draft = edited(moveChoice(saved.order, 1, 0));
    expect(json(shownOrder(draft, saved))).toBe(
      json(["MacBook", "USB headset"]),
    );
    expect(json(saved.order)).toBe(json(["USB headset", "MacBook"]));
    expect(draft.base).toBe(saved);
    expect(isDirty(draft, saved)).toBe(true);
    expect(canSave(draft, saved)).toBe(true);
  });

  test("dragging the first of three below the third", () => {
    const order = moveChoice(["A", "B", "C"], 0, 2);
    expect(json(order)).toBe(json(["B", "C", "A"]));
    expect(json(moveChoice(["A", "B"], 0, 5))).toBe(json(["A", "B"]));
  });

  test("an unavailable choice is removed like any other", () => {
    expect(json(removeChoice(saved.order, 0))).toBe(json(["MacBook"]));
  });

  test("a duplicate is refused, keeps the order and clears once corrected", () => {
    const refused = addChoice(EMPTY_DRAFT, saved, "USB headset");
    expect(refused.order).toBe(null);
    expect(refused.addError?.name).toBe("USB headset");
    expect(refused.addError?.message.key).toBe(PRIORITY_KEYS.duplicateChoice);
    const corrected = addChoice(refused, saved, "Studio Display");
    expect(corrected.addError).toBe(null);
    expect(json(corrected.order)).toBe(
      json(["USB headset", "MacBook", "Studio Display"]),
    );
  });

  test("a blank name is refused", () => {
    expect(addChoice(EMPTY_DRAFT, saved, "  ").addError?.message.key).toBe(
      PRIORITY_KEYS.empty,
    );
  });

  test("an invalid draft cannot be saved", () => {
    expect(orderError(["A", "A"])?.key).toBe(PRIORITY_KEYS.duplicate);
    expect(orderError(["A", " "])?.key).toBe(PRIORITY_KEYS.empty);
    expect(canSave(edited(["A", "A"]), saved)).toBe(false);
  });

  test("removing every choice empties only the draft and Save is enabled", () => {
    const draft = edited(removeChoice(removeChoice(saved.order, 0), 0));
    expect(json(draft.order)).toBe(json([]));
    expect(json(saved.order)).toBe(json(["USB headset", "MacBook"]));
    expect(canSave(draft, saved)).toBe(true);
  });

  test("an edit back to the committed order is not dirty", () => {
    expect(isDirty(edited([...saved.order]), saved)).toBe(false);
    expect(canSave(edited([...saved.order]), saved)).toBe(false);
  });

  test("edits are ignored while a write is in flight", () => {
    const saving = startSave(edited(["MacBook"]));
    expect(canSave(saving, saved)).toBe(false);
    expect(editOrder(saving, saved, [])).toBe(saving);
    expect(addChoice(saving, saved, "Dock")).toBe(saving);
  });
});

// @spec AC-007: Stale revision keeps the draft and offers Reload — .specs/features/001-microphone-priority/spec.md#ac-007
describe("revision changes while editing", () => {
  const draft = edited(["MacBook", "USB headset"]);

  test("another setting changed: only the base revision advances", () => {
    const next = { order: saved.order, revision: 5 };
    const rebased = rebaseDraft(draft, next);
    expect(rebased.conflict).toBe(false);
    expect(rebased.base?.revision).toBe(5);
    expect(json(rebased.order)).toBe(json(draft.order));
  });

  test("our own Save committed the draft: no conflict", () => {
    const next = { order: ["MacBook", "USB headset"], revision: 5 };
    expect(rebaseDraft(draft, next).conflict).toBe(false);
  });

  test("the order changed elsewhere: conflict, draft kept, base current", () => {
    const next = { order: ["Dock"], revision: 6 };
    const rebased = rebaseDraft(draft, next);
    expect(rebased.conflict).toBe(true);
    expect(json(rebased.order)).toBe(json(draft.order));
    expect(rebased.base?.revision).toBe(6);
    expect(canSave(rebased, next)).toBe(true);
  });

  test("nothing to rebase without a draft", () => {
    expect(rebaseDraft(EMPTY_DRAFT, { order: [], revision: 9 })).toBe(
      EMPTY_DRAFT,
    );
  });

  test("a revision_conflict refusal keeps the draft and raises the conflict", () => {
    const failed = failSave(
      startSave(draft),
      refusal({ code: "revision_conflict", current_revision: 7 }),
    );
    expect(failed.conflict).toBe(true);
    expect(failed.savePhase).toBe("idle");
    expect(json(failed.order)).toBe(json(draft.order));
  });
});

// @spec FR-007: Typed refusals translated, retry offered — .specs/features/001-microphone-priority/spec.md#fr-007
describe("failed writes", () => {
  test("a persistence failure keeps the draft with an error and a retry", () => {
    const failed = failSave(
      startSave(edited(["MacBook"])),
      refusal({ code: "persistence_failed", reason: null }),
    );
    expect(failed.savePhase).toBe("failed");
    expect(failed.saveError?.key).toBe("errors.settings.persistenceFailed");
    expect(canSave(failed, saved)).toBe(true);
  });

  test("refused microphone names use the editor's messages", () => {
    const item = (
      code: "microphone_name_duplicate" | "microphone_name_empty",
    ) => refusal({ items: [{ index: 1, code, message: "" }] });
    expect(saveErrorMessage(item("microphone_name_duplicate")).key).toBe(
      PRIORITY_KEYS.duplicate,
    );
    expect(saveErrorMessage(item("microphone_name_empty")).key).toBe(
      PRIORITY_KEYS.empty,
    );
  });

  test("other refusals use the shared settings messages", () => {
    expect(saveErrorMessage(refusal({ reason: null })).key).toBe(
      "errors.settings.validationFailed",
    );
    expect(saveErrorMessage(null).key).toBe("errors.settings.generic");
  });
});
