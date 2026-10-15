/**
 * Pure model of the recovery bubble (010): the reason line and its icon per reason, named or
 * not, the explanation sentences (history kept only when saved, the partial-insertion detail
 * only for method failures, "nothing was sent" for a missing permission, the late read, the
 * platform advice) and the drag gesture threshold. Run with `bun run test:recovery-bubble`.
 *
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: dismissible_layer, has_drag_drop, has_tooltip, is_keyboard_navigable
 */
import { describe, expect, test } from "bun:test";
import type { RecoveryBubbleView } from "@/bindings";
import { bubbleTexts, dragStarts, DRAG_THRESHOLD_PX } from "./recovery-preview";

const view = (patch: Partial<RecoveryBubbleView> = {}): RecoveryBubbleView => ({
  bubble_id: 3,
  destination_generation: 9,
  reason: "app_closed",
  app_name: "Notes",
  preview: "Hi Marie, the release moves to Thursday.",
  truncated: false,
  detail: "not_inserted",
  late_read: false,
  history: "kept",
  can_drag: true,
  ...patch,
});
const keys = (patch: Partial<RecoveryBubbleView>) =>
  bubbleTexts(view(patch)).explanation.map((line) => line.key);

describe("reason line (AC-003)", () => {
  test("each reason has its sentence, named when the app is known", () => {
    const cases: [RecoveryBubbleView["reason"], string, string][] = [
      ["app_closed", "recoveryBubble.reason.appClosed", "warning"],
      [
        "pinned_app_not_running",
        "recoveryBubble.reason.pinnedAppNotRunning",
        "warning",
      ],
      [
        "activation_failed",
        "recoveryBubble.reason.activationFailed",
        "warning",
      ],
      ["insertion_failed", "recoveryBubble.reason.insertionFailed", "error"],
      ["no_text_field", "recoveryBubble.reason.noTextField", "info"],
    ];
    for (const [reason, key, icon] of cases) {
      const texts = bubbleTexts(view({ reason }));
      expect(texts.reasonKey).toBe(key);
      expect(texts.icon).toBe(icon);
    }
    expect(bubbleTexts(view()).reasonValues).toEqual({ app: "Notes" });
  });
  test("without a name the application noun is used, never an invented identity", () => {
    const texts = bubbleTexts(view({ app_name: null }));
    expect(texts.reasonKey).toBe("recoveryBubble.reason.appClosedUnnamed");
    expect(texts.reasonValues).toEqual({});
  });
});

describe("explanation (AC-001, AC-005)", () => {
  test("a reason decided before insertion says nothing was inserted, never 'some text'", () => {
    expect(keys({})).toEqual([
      "recoveryBubble.explanation.notInserted",
      "recoveryBubble.explanation.historyKept",
      "recoveryBubble.explanation.openAppDrag",
    ]);
    expect(keys({ reason: "no_text_field", detail: "not_inserted" })).toEqual([
      "recoveryBubble.explanation.notInserted",
      "recoveryBubble.explanation.historyKept",
      "recoveryBubble.explanation.focusFieldDrag",
      "recoveryBubble.explanation.noFieldSetting",
    ]);
  });
  test("history is said kept only when the save succeeded", () => {
    expect(keys({ history: "not_kept" })).toContain(
      "recoveryBubble.explanation.historyNotKept",
    );
    expect(keys({ history: "not_kept" })).not.toContain(
      "recoveryBubble.explanation.historyKept",
    );
  });
  test("method failures may have reached the app; a missing permission sent nothing", () => {
    const failed = {
      reason: "insertion_failed",
      detail: "may_have_reached",
    } as const;
    expect(keys(failed)).toEqual([
      "recoveryBubble.explanation.mayHaveReached",
      "recoveryBubble.explanation.historyKeptFull",
    ]);
    expect(keys({ ...failed, detail: "nothing_sent" })).toEqual([
      "recoveryBubble.explanation.nothingSent",
      "recoveryBubble.explanation.historyKeptFull",
    ]);
  });
  test("a late read adds that the text may have been pasted after all", () => {
    const late = keys({
      reason: "insertion_failed",
      detail: "may_have_reached",
      late_read: true,
    });
    expect(late).toContain("recoveryBubble.explanation.lateRead");
  });
  test("without drag (Windows, Linux) the advice only mentions Copy", () => {
    expect(keys({ can_drag: false })).toContain(
      "recoveryBubble.explanation.openAppCopy",
    );
    expect(keys({ can_drag: false, app_name: null })).toContain(
      "recoveryBubble.explanation.openUnnamedCopy",
    );
  });
});

describe("drag gesture (has_drag_drop: drag-start)", () => {
  test("a drag starts only with the primary button held past the threshold", () => {
    const start = { x: 10, y: 10 };
    const beyond = { x: 10 + DRAG_THRESHOLD_PX + 1, y: 10 };
    expect(dragStarts(start, beyond, 1)).toBe(true);
    expect(dragStarts(start, { x: 11, y: 11 }, 1)).toBe(false);
    expect(dragStarts(start, beyond, 0)).toBe(false);
    expect(dragStarts(null, beyond, 1)).toBe(false);
  });
});
