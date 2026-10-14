/**
 * Markup of the recording indicator stack with the rollout gates closed (no destination
 * session, no notice: the Handy pill alone, as on `dev`) and open (badge, menus, notices), with
 * the contract test IDs and accessible names. Run with `bun run test:indicator`.
 */
import { beforeAll, describe, expect, test } from "bun:test";
import i18n from "i18next";
import { createElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { initReactI18next } from "react-i18next";
import type {
  DestinationMenu,
  DestinationSnapshot,
  DictationPromptView,
  PromptLibraryView,
} from "@/bindings";
import en from "@/i18n/locales/en/translation.json";
import { badgeView } from "./destination-model";
import { IndicatorStack } from "./indicator-stack";
import {
  aiFailureNotice,
  destinationNotice,
  handOverNotice,
  type OverlayNotice,
} from "./notice-model";
import type {
  DestinationControls,
  DestinationMenuState,
} from "./use-destination";
import type { LibraryState, PromptControls } from "./use-dictation-prompt";

beforeAll(async () => {
  await i18n.use(initReactI18next).init({
    lng: "en",
    resources: { en: { translation: en } },
    interpolation: { escapeValue: false },
  });
});

const noop = () => undefined;
const PILL =
  '<div class="scard compact" data-testid="recording-indicator-pill">pill</div>';

const session = (
  selection: DestinationSnapshot["selection"],
  phase = "recording",
) =>
  ({
    dictation_id: 7,
    destination_generation: 1,
    selection_version: 1,
    phase,
    return_enabled: true,
    support: { state: "supported" },
    selection,
    disposition: "pending",
  }) as DestinationSnapshot;
const chatgpt = {
  kind: "application",
  display_name: "ChatGPT",
  icon_ref: null,
} as const;

const destination = (
  snapshot: DestinationSnapshot | null,
  menu: DestinationMenuState = { status: "closed" },
): DestinationControls => ({
  snapshot,
  latest: { current: snapshot },
  badge: badgeView(snapshot),
  menu,
  feedbackKey: null,
  toggleMenu: noop,
  closeMenu: noop,
  choose: noop,
  remove: noop,
});

const library: PromptLibraryView = {
  default_prompt_id: "min",
  post_process_enabled: true,
  prompts: [
    {
      id: "min",
      name: "Minimal correction",
      prompt: "text",
      provider_override: null,
      starter: true,
      issues: [],
      shortcuts: ["ctrl+option+m"],
      readiness: {
        provider_id: null,
        model: null,
        is_override: false,
        ready: true,
        missing: null,
      },
    },
  ],
};
const promptView: DictationPromptView = {
  session_id: 3,
  prompt_id: "min",
  prompt_name: "Minimal correction",
  selectable: true,
};
const prompt = (
  view: DictationPromptView | null,
  menuOpen = false,
  state: LibraryState = { status: "ready", library },
): PromptControls => ({
  view,
  library: state,
  menuOpen,
  feedbackKey: null,
  openMenu: noop,
  closeMenu: noop,
  select: noop,
});

const render = (
  options: {
    destination?: DestinationControls;
    prompt?: PromptControls;
    notices?: OverlayNotice[];
    showIndicator?: boolean;
  } = {},
): string => {
  const pill: ReactNode = createElement(
    "div",
    { className: "scard compact", "data-testid": "recording-indicator-pill" },
    "pill",
  );
  return renderToStaticMarkup(
    createElement(IndicatorStack, {
      position: "bottom",
      direction: "ltr",
      form: "compact",
      showIndicator: options.showIndicator ?? true,
      recording: true,
      destination: options.destination ?? destination(null),
      prompt: options.prompt ?? prompt(null),
      notices: options.notices ?? [],
      osType: "macos",
      children: pill,
    }),
  );
};

describe("rollout gates closed (destination-return, smart-paste-dispatch)", () => {
  test("no session, no prompt, no notice: the Handy pill alone, as on dev", () => {
    expect(render()).toBe(
      '<div dir="ltr" class="ov-stage bottom" data-testid="recording-indicator-screen">' +
        '<div class="ov-stack" data-testid="recording-indicator-bar">' +
        `<div class="ov-fade show">${PILL}</div></div></div>`,
    );
  });
  test("a session with the setting off shows no badge either", () => {
    const off = { ...session(chatgpt), return_enabled: false };
    expect(render({ destination: destination(off) })).not.toContain(
      "recording-indicator-destination",
    );
  });
});

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
        },
        {
          token: 2,
          display_name: "ChatGPT",
          icon_ref: null,
          current: true,
          frontmost: false,
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
