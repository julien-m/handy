/**
 * Markup of the recovery bubble (010) with the contract test IDs and accessible names: open
 * state with reason, preview, drag handle (macOS), Copy, Close and Why?; the explanation shown
 * ("Hide details", expanded); no drag handle without drag; the closed state renders nothing.
 * Run with `bun run test:recovery-bubble`.
 *
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: dismissible_layer (close-button), has_drag_drop (drag-start), has_tooltip
 * (tooltip-show, tooltip-hide), is_keyboard_navigable (keyboard-focus)
 */
import { beforeAll, describe, expect, test } from "bun:test";
import i18n from "i18next";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { initReactI18next } from "react-i18next";
import type { RecoveryBubbleView } from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import { RecoveryBubble } from "./recovery-bubble";
import { recoveryScreen } from "./recovery-preview";

const noop = () => undefined;
const view = (patch: Partial<RecoveryBubbleView> = {}): RecoveryBubbleView => ({
  bubble_id: 3,
  destination_generation: 9,
  reason: "app_closed",
  app_name: "Notes",
  preview: "Hi Marie, the release moves to Thursday.",
  truncated: true,
  detail: "not_inserted",
  late_read: false,
  history: "kept",
  can_drag: true,
  ...patch,
});
const render = (shown: RecoveryBubbleView | null, explained = false) =>
  renderToStaticMarkup(
    createElement(RecoveryBubble, {
      view: shown,
      explained,
      onToggleExplanation: noop,
      onCopy: noop,
      onClose: noop,
      onDragPointer: noop,
    }),
  );

beforeAll(async () => {
  await i18n.use(initReactI18next).init({
    lng: "en",
    resources: { en: { translation: en } },
    interpolation: { escapeValue: false },
  });
});

describe("open state (dismissible_layer: open)", () => {
  test("reason, preview and every control carry their test IDs and names", () => {
    const html = render(view());
    expect(html).toContain('data-testid="recovery-bubble-bubble"');
    expect(html).toContain('data-state="open"');
    expect(html).toContain('data-testid="recovery-bubble-reason"');
    expect(html).toContain(">Notes is no longer open<");
    expect(html).toContain('data-testid="recovery-bubble-preview"');
    expect(html).toContain("Hi Marie, the release moves to Thursday.");
    expect(html).toContain('data-truncated="true"');
    expect(html).toContain('data-testid="recovery-bubble-drag-handle"');
    expect(html).toContain('data-testid="recovery-bubble-copy"');
    expect(html).toContain('data-testid="recovery-bubble-why"');
    expect(html).toContain('aria-label="Close"');
    expect(html).toContain('aria-expanded="false"');
    expect(html).toContain(">Why?<");
  });
  test("every reason of AC-003 is worded as specified", () => {
    const reasons: [RecoveryBubbleView["reason"], string][] = [
      ["pinned_app_not_running", "Notes is pinned but not open"],
      ["activation_failed", "Could not bring Notes forward"],
      ["insertion_failed", "Insertion failed"],
      ["no_text_field", "No text field is focused"],
    ];
    for (const [reason, text] of reasons) {
      expect(render(view({ reason }))).toContain(`>${text}<`);
    }
  });
});

describe("explanation (has_tooltip: tooltip-show / tooltip-hide)", () => {
  test("shown on request with history, advice and Hide details", () => {
    const html = render(view(), true);
    expect(html).toContain(
      'data-testid="recovery-bubble-explanation-explanation"',
    );
    expect(html).toContain(
      "Handy didn&#x27;t insert anything and sent no Enter.",
    );
    expect(html).toContain("The transcript is kept in History.");
    expect(html).toContain("Open Notes, then drag or copy the text.");
    expect(html).toContain('aria-expanded="true"');
    expect(html).toContain(">Hide details<");
    // D1: explained, every element carries the explanation screen's prefix.
    for (const id of ["bubble", "reason", "close", "preview", "copy", "why"]) {
      expect(html).toContain(`data-testid="recovery-bubble-explanation-${id}"`);
    }
    expect(html).toContain(
      'data-semantic-id="recovery-bubble-explanation.close"',
    );
    expect(render(view(), false)).not.toContain(
      "recovery-bubble-explanation-explanation",
    );
  });
  test("an insertion failure discloses possible partial text, never 'kept' without a save", () => {
    const failed = view({
      reason: "insertion_failed",
      detail: "may_have_reached",
      history: "not_kept",
      late_read: true,
    });
    const html = render(failed, true);
    expect(html).toContain("Some text may have reached the application.");
    expect(html).toContain("may have been pasted after all");
    expect(html).toContain("The transcript is not in History.");
    expect(html).not.toContain("kept in History");
    expect(html).toContain(
      'data-testid="recovery-bubble-insertion-failed-explanation"',
    );
    expect(html).toContain(
      'data-testid="recovery-bubble-insertion-failed-explanation-text"',
    );
  });
  test("a late read is visible and announced without opening the explanation", () => {
    const late = view({
      reason: "insertion_failed",
      detail: "may_have_reached",
      late_read: true,
    });
    const html = render(late, false);
    // D1: the late read shows on the insertion-failed screen, with its prefix.
    expect(html).toContain(
      'data-testid="recovery-bubble-insertion-failed-late-read"',
    );
    expect(html).toContain('role="status"');
    expect(html).toContain("may have been pasted after all");
    expect(render(view(), false)).not.toContain("late-read");
  });
});

describe("platform actions (has_drag_drop: drag-start, AC-006)", () => {
  test("without drag the handle is absent and Copy, Why? and Close remain", () => {
    const html = render(view({ can_drag: false }));
    expect(html).not.toContain("recovery-bubble-drag-handle");
    expect(html).toContain('data-testid="recovery-bubble-copy"');
    expect(html).toContain('data-testid="recovery-bubble-close"');
  });
});

describe("closed state (dismissible_layer: closed)", () => {
  test("nothing is rendered without a bubble", () => {
    expect(render(null)).toBe("");
  });
});

describe("keyboard and assistive technology (is_keyboard_navigable: keyboard-focus)", () => {
  test("every action is a native button reachable by Tab and the accessibility focus", () => {
    const html = render(view());
    const buttons = html.match(/<button /g) ?? [];
    expect(buttons.length).toBe(4);
    expect(html).not.toContain('tabindex="-1"');
  });
});

describe("contract screen shown (D1)", () => {
  test("each reason and the explanation pick their screen", () => {
    expect(recoveryScreen(null, false)).toBe("recovery-bubble");
    expect(recoveryScreen(view(), false)).toBe("recovery-bubble");
    expect(recoveryScreen(view(), true)).toBe("recovery-bubble-explanation");
    expect(recoveryScreen(view({ reason: "no_text_field" }), true)).toBe(
      "recovery-bubble-no-field",
    );
    expect(recoveryScreen(view({ reason: "insertion_failed" }), false)).toBe(
      "recovery-bubble-insertion-failed",
    );
  });

  test("no text field: every element carries the no-field prefix", () => {
    const html = render(view({ reason: "no_text_field" }));
    for (const id of ["bubble", "reason", "drag-handle", "copy", "why"]) {
      expect(html).toContain(`data-testid="recovery-bubble-no-field-${id}"`);
    }
    expect(html).not.toContain('data-testid="recovery-bubble-bubble"');
  });
});
