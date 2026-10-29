/**
 * Markup of the recording indicator stack with the rollout gates open: destination badge and
 * menu (006), prompt pill and menu (005) and notices (005, 007), with the contract test IDs
 * and accessible names. The closed-gate markup is in `indicator-gates-render.test.tsx`. Run with
 * `bun run test:indicator`.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import type { DestinationMenu } from "@/bindings";
import {
  chatgpt,
  destination,
  initIndicatorI18n,
  prompt,
  promptView,
  render,
  session,
} from "./indicator-test-fixtures";
import {
  aiFailureNotice,
  handOverNotice,
  inputLostNotice,
  pasteFailedNotice,
} from "./notice-model";

beforeAll(initIndicatorI18n);

describe("destination badge (006)", () => {
  test("an application shows icon, name, accessible name and the remove cross", () => {
    const html = render({ destination: destination(session(chatgpt)) });
    expect(html).toContain('data-testid="recording-indicator-destination"');
    expect(html).toContain('aria-label="Destination: ChatGPT"');
    expect(html).toContain(
      'data-testid="recording-indicator-destination-name">ChatGPT<',
    );
    expect(html).toContain('aria-label="Remove destination"');
    expect(html).toContain("sindicator with-dest compact");
    expect(html).toContain('data-testid="recording-indicator-screen"');
    expect(html).toContain('data-testid="recording-indicator-bar"');
  });
  test("without destination: Frontmost app and no cross", () => {
    const html = render({
      destination: destination(session({ kind: "frontmost_at_end" })),
    });
    expect(html).toContain('aria-label="Destination: Frontmost app"');
    expect(html).not.toContain("destination-remove");
    // D1: every test id carries the no-destination screen's prefix.
    for (const id of ["screen", "bar", "destination", "destination-name"]) {
      expect(html).toContain(
        `data-testid="recording-indicator-no-destination-${id}"`,
      );
    }
    expect(html).not.toContain('data-testid="recording-indicator-destination"');
  });
  test("once insertion begins the badge is frozen and the cross is gone", () => {
    const html = render({
      destination: destination(session(chatgpt, "inserting")),
    });
    expect(html).toMatch(
      /data-testid="recording-indicator-destination"[^>]*disabled=""|disabled=""[^>]*data-testid="recording-indicator-destination"/,
    );
    expect(html).not.toContain("recording-indicator-destination-remove");
  });
  test("the open menu lists frontmost now, other apps and Frontmost app at the end", () => {
    const menu: DestinationMenu = {
      dictation_id: 7,
      selection_version: 1,
      frontmost_at_end_current: false,
      entries: [
        {
          token: 1,
          display_name: "Comet",
          icon_ref: null,
          current: false,
          frontmost: true,
          pinned: false,
        },
        {
          token: 2,
          display_name: "ChatGPT",
          icon_ref: null,
          current: true,
          frontmost: false,
          pinned: false,
        },
      ],
    };
    const html = render({
      destination: destination(session(chatgpt), { status: "open", menu }),
    });
    const id = "recording-indicator-destination-menu-menu";
    for (const suffix of [
      "",
      "-now-label",
      "-frontmost",
      "-others-label",
      "-app",
      "-frontmost-at-end",
    ]) {
      expect(html).toContain(`data-testid="${id}${suffix}"`);
    }
    expect(html).toContain('data-instance-id="app-2"');
    // D1: the badge and the bar carry the destination-menu screen's prefix too.
    for (const shared of [
      "screen",
      "bar",
      "destination",
      "destination-remove",
    ]) {
      expect(html).toContain(
        `data-testid="recording-indicator-destination-menu-${shared}"`,
      );
    }
    expect(html).not.toContain('data-testid="recording-indicator-destination"');
    expect(html.indexOf("Comet")).toBeLessThan(html.indexOf(">ChatGPT<"));
    expect(html).toContain('aria-checked="true"');
  });
  test("loading and error states of the menu", () => {
    const loading = render({
      destination: destination(session(chatgpt), { status: "loading" }),
    });
    expect(loading).toContain("Loading apps…");
    expect(loading).toContain(
      'data-testid="recording-indicator-destination-menu-menu-loading"',
    );
    const failed = render({
      destination: destination(session(chatgpt), { status: "error" }),
    });
    expect(failed).toContain("The app list could not be loaded.");
    expect(failed).toContain(
      'data-testid="recording-indicator-destination-menu-menu-failed"',
    );
  });
  test("a refused choice shows its feedback in the open menu", () => {
    const html = render({
      destination: {
        ...destination(session(chatgpt), { status: "loading" }),
        feedbackKey: "overlay.destination.menu.failed",
      },
    });
    expect(html).toContain(
      'data-testid="recording-indicator-destination-menu-menu-feedback"',
    );
  });
});

// @spec AC-001: Pinned badge and pin entries render — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-001
describe("pinned destination (009)", () => {
  const pinnedSession = (returnEnabled = true) => ({
    ...session(chatgpt),
    return_enabled: returnEnabled,
    source: "pinned" as const,
    pin: { display_name: "ChatGPT", icon_ref: null },
  });
  const menu: DestinationMenu = {
    dictation_id: 7,
    selection_version: 1,
    frontmost_at_end_current: false,
    entries: [
      {
        token: 1,
        display_name: "Comet",
        icon_ref: null,
        current: false,
        frontmost: true,
        pinned: false,
      },
    ],
  };
  test("the pinned badge shows the pin mark and announces it", () => {
    const html = render({ destination: destination(pinnedSession()) });
    expect(html).toContain('aria-label="Destination: ChatGPT, pinned"');
    expect(html).toContain(
      'data-testid="recording-indicator-pinned-destination-mark"',
    );
    expect(html).toContain('aria-label="Remove destination"');
  });
  test("with return off, an active pin still shows the badge", () => {
    const html = render({ destination: destination(pinnedSession(false)) });
    expect(html).toContain(
      'data-testid="recording-indicator-pinned-destination"',
    );
  });
  test("the open menu lists Unpin at the bottom and marks the other apps", () => {
    const html = render({
      destination: destination(pinnedSession(), { status: "open", menu }),
    });
    const unpin = html.indexOf(
      'data-testid="recording-indicator-pinned-menu-unpin"',
    );
    expect(unpin > -1).toBe(true);
    expect(html).toContain(">Unpin ChatGPT<");
    expect(unpin > html.indexOf(">Comet<")).toBe(true);
    const atEnd = html.indexOf(
      'data-testid="recording-indicator-pinned-menu-frontmost-at-end"',
    );
    expect(atEnd > -1 && unpin > atEnd).toBe(true);
    // D1: the menu, the badge and the bar carry the pinned screen's prefix.
    for (const shared of [
      "screen",
      "bar",
      "menu",
      "menu-frontmost",
      "destination",
    ]) {
      expect(html).toContain(
        `data-testid="recording-indicator-pinned-${shared}"`,
      );
    }
    expect(html).not.toContain("recording-indicator-destination-menu-menu");
    expect(html).not.toContain("recording-indicator-pinned-menu-pin");
    expect(html).toContain(
      'data-testid="recording-indicator-pinned-menu-hint">This dictation only<',
    );
  });
  test("nothing pinned: Pin <destination> is listed above the application list", () => {
    const html = render({
      destination: destination(session(chatgpt), { status: "open", menu }),
    });
    // 009-01: the contract's Pin entry of the destination menu screen.
    const pin = html.indexOf(
      'data-testid="recording-indicator-destination-menu-menu-pin"',
    );
    expect(pin > -1).toBe(true);
    expect(html).toContain(">Pin ChatGPT<");
    expect(pin).toBeLessThan(html.indexOf(">Comet<"));
    expect(html).not.toContain("recording-indicator-pinned");
  });
});

describe("prompt pill (005)", () => {
  test("the pill shows the prompt, its shortcut badge and the hover hint", () => {
    const html = render({ prompt: prompt(promptView) });
    expect(html).toContain('data-testid="recording-indicator-prompt"');
    expect(html).toContain(
      'data-testid="recording-indicator-prompt-shortcut">⌃ ⌥ M<',
    );
    expect(html).toContain(
      'data-testid="recording-indicator-prompt-hint">Hover to see options<',
    );
    expect(html.indexOf("recording-indicator-prompt")).toBeLessThan(
      html.indexOf("recording-indicator-pill"),
    );
  });
  test("AI processing off: no pill", () => {
    expect(
      render({ prompt: prompt({ ...promptView, selectable: false }) }),
    ).not.toContain("recording-indicator-prompt");
  });
  test("the open menu lists None and the prompts with the current one checked", () => {
    const html = render({ prompt: prompt(promptView, true) });
    const id = "recording-indicator-prompt-menu-menu";
    for (const suffix of [
      "",
      "-label",
      "-none",
      "-prompt",
      "-prompt-shortcut",
      "-note",
    ]) {
      expect(html).toContain(`data-testid="${id}${suffix}"`);
    }
    expect(html).toContain('data-instance-id="prompt-min"');
    // D1: the pill and the bar carry the prompt-menu screen's prefix.
    for (const shared of ["screen", "bar", "prompt", "prompt-shortcut"]) {
      expect(html).toContain(
        `data-testid="recording-indicator-prompt-menu-${shared}"`,
      );
    }
    expect(html).not.toContain('data-testid="recording-indicator-prompt"');
    expect(html).toContain(
      "Applies to this dictation only. Default stays Minimal correction.",
    );
    expect(html).not.toContain("recording-indicator-prompt-hint");
  });
  // @spec AC-009: the menu of a plain dictation says the choice is kept — .specs/features/005-custom-ai-prompts/spec.md#ac-009
  test("a plain dictation's menu says the choice is kept for the next plain dictations", () => {
    const html = render({
      prompt: prompt({ ...promptView, remembered: true }, true),
    });
    expect(html).toContain(
      "Kept for your next plain dictations until you choose None.",
    );
    expect(html).not.toContain("Applies to this dictation only.");
  });
});

describe("notices (005, 007)", () => {
  const failure = {
    session_id: 3,
    prompt_name: null,
    reason: "timeout",
    notice_ms: 4000,
  } as const;
  test("each notice has its test ID and text, and outlives the indicator", () => {
    const notices = [aiFailureNotice(failure, 0), handOverNotice(0)];
    const html = render({ notices, showIndicator: false });
    expect(html).toContain(
      'data-testid="recording-indicator-ai-failed-notice"',
    );
    expect(html).toContain("AI processing failed. The original text was used.");
    expect(html).toContain(
      'data-testid="recording-indicator-hand-over-notice"',
    );
    expect(html).toContain(
      "Text placed on the clipboard for the remote screen",
    );
    expect(html).not.toContain("recording-indicator-pill");
    // Shown alone, the newest notice names the screen.
    expect(html).toContain(
      'data-testid="recording-indicator-hand-over-screen"',
    );
  });
});

describe("input-lost notice (001, native session N1)", () => {
  test("the cut-short notice shows next to the indicator with its test ID and text", () => {
    const html = render({ notices: [inputLostNotice(0)] });
    expect(html).toContain(
      'data-testid="recording-indicator-input-lost-notice"',
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain(
      "The microphone was disconnected. The audio recorded until then is being transcribed.",
    );
    expect(html).toContain('data-testid="recording-indicator-pill"');
  });
  test("shown alone after the dictation, it names its own screen", () => {
    const html = render({
      notices: [inputLostNotice(0)],
      showIndicator: false,
    });
    expect(html).toContain(
      'data-testid="recording-indicator-input-lost-screen"',
    );
    expect(html).not.toContain("recording-indicator-pill");
  });
});

describe("paste-failed notice (010, native session N17)", () => {
  test("a failed insertion without a bubble shows on the indicator with its test ID and text", () => {
    const html = render({
      notices: [pasteFailedNotice({ history_saved: false }, 0)],
    });
    expect(html).toContain(
      'data-testid="recording-indicator-paste-failed-notice"',
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain("The text could not be pasted or saved to history.");
  });
  test("shown alone after the dictation, it names its own screen", () => {
    const html = render({
      notices: [pasteFailedNotice(null, 0)],
      showIndicator: false,
    });
    expect(html).toContain(
      'data-testid="recording-indicator-paste-failed-screen"',
    );
    expect(html).toContain(
      "Text could not be pasted into the active application.",
    );
    expect(html).not.toContain("recording-indicator-pill");
  });
});
