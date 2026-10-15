/**
 * Markup of the recording indicator stack with the rollout gates open: destination badge and
 * menu (006), prompt pill and menu (005) and notices (005, 006, 007), with the contract test IDs
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
  destinationNotice,
  handOverNotice,
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
  });
  test("without destination: Frontmost app and no cross", () => {
    const html = render({
      destination: destination(session({ kind: "frontmost_at_end" })),
    });
    expect(html).toContain('aria-label="Destination: Frontmost app"');
    expect(html).not.toContain("recording-indicator-destination-remove");
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
    expect(html.indexOf("Comet")).toBeLessThan(html.indexOf(">ChatGPT<"));
    expect(html).toContain('aria-checked="true"');
  });
  test("loading and error states of the menu", () => {
    expect(
      render({
        destination: destination(session(chatgpt), { status: "loading" }),
      }),
    ).toContain("Loading apps…");
    expect(
      render({
        destination: destination(session(chatgpt), { status: "error" }),
      }),
    ).toContain("The app list could not be loaded.");
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
    expect(html).toContain('data-testid="recording-indicator-destination"');
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
    expect(
      unpin >
        html.indexOf(
          'data-testid="recording-indicator-destination-menu-menu-frontmost-at-end"',
        ),
    ).toBe(true);
    expect(html).not.toContain("recording-indicator-pinned-menu-pin");
    expect(html).toContain(
      'data-testid="recording-indicator-pinned-menu-hint">This dictation only<',
    );
  });
  test("nothing pinned: Pin <destination> is listed above the application list", () => {
    const html = render({
      destination: destination(session(chatgpt), { status: "open", menu }),
    });
    const pin = html.indexOf(
      'data-testid="recording-indicator-pinned-menu-pin"',
    );
    expect(pin > -1).toBe(true);
    expect(html).toContain(">Pin ChatGPT<");
    expect(pin).toBeLessThan(html.indexOf(">Comet<"));
    expect(html).not.toContain("recording-indicator-pinned-menu-hint");
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
    expect(html).toContain(
      "Applies to this dictation only. Default stays Minimal correction.",
    );
    expect(html).not.toContain("recording-indicator-prompt-hint");
  });
});

describe("notices (005, 006, 007)", () => {
  const failure = {
    session_id: 3,
    prompt_name: null,
    reason: "timeout",
    notice_ms: 4000,
  } as const;
  const closed = destinationNotice(
    { dictationId: 7, failed: true, reason: "app_closed", historySaved: true },
    "Notes",
    0,
  );
  test("each notice has its test ID and text, and outlives the indicator", () => {
    const notices = [
      aiFailureNotice(failure, 0),
      handOverNotice(0),
      ...(closed ? [closed] : []),
    ];
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
    expect(html).toContain(
      "Notes is closed. Nothing was inserted; the transcript is kept in History.",
    );
    expect(html).not.toContain("recording-indicator-pill");
  });
});
