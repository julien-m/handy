/**
 * Markup of the recording indicator stack with the rollout gates closed (no destination
 * session, no notice: compared with the pill of `dev`) and open (badge, menus, notices), with
 * the contract test IDs and accessible names, and the guard that the pill alone fits the base
 * window of its form. Run with `bun run test:indicator`.
 */
import { readFileSync } from "node:fs";
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
import { extentRequest } from "./use-overlay-extent";
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
    form?: "compact" | "live";
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
      form: options.form ?? "compact",
      showIndicator: options.showIndicator ?? true,
      recording: true,
      dictation: 1,
      destination: options.destination ?? destination(null),
      prompt: options.prompt ?? prompt(null),
      notices: options.notices ?? [],
      osType: "macos",
      children: pill,
    }),
  );
};

// ---- Gates closed: the rendered pill compared with the pill of dev ----

/**
 * The overlay of `dev` around the pill (RecordingOverlay.tsx before 005/006/007, commit
 * 69a6ee73^): compact `<div dir class="ov-stage <position> ov-fade show">` holding the card,
 * Live `<div dir class="ov-stage <position>">` holding it.
 */
const devOverlay = (form: "compact" | "live", card: string): string =>
  `<div dir="ltr" class="ov-stage bottom${form === "compact" ? " ov-fade show" : ""}">${card}</div>`;

/** Wrappers the stack adds that cannot change how a single child looks (checked below). */
const LAYOUT_NEUTRAL = new Set(["ov-stack"]);

/** Attributes of the elements still open where `end` starts (the ancestors of what is there). */
const openAncestors = (markup: string, end: number): string[] => {
  const open: string[] = [];
  for (const [tag, attributes] of markup
    .slice(0, end)
    .matchAll(/<(?:div|span)([^>]*)>|<\/(?:div|span)>/g)) {
    if (tag.startsWith("</")) open.pop();
    else open.push(attributes ?? "");
  }
  return open;
};

/** What decides how the pill looks: its own markup and its ancestors' direction and classes. */
const pillLook = (markup: string) => {
  const start = markup.indexOf(PILL);
  const ancestors = openAncestors(markup, start);
  const classes = ancestors
    .flatMap(
      (attributes) => /class="([^"]*)"/.exec(attributes)?.[1].split(" ") ?? [],
    )
    .filter((name) => name !== "" && !LAYOUT_NEUTRAL.has(name))
    .sort();
  const styled = ancestors.filter((attributes) => /style=/.test(attributes));
  const dir = ancestors.map(
    (attributes) => /dir="([^"]*)"/.exec(attributes)?.[1],
  );
  return { pill: start >= 0, classes, styled, dir: dir.filter(Boolean) };
};

/** Rules of a stylesheet: selector list and declared property names. */
const cssRules = (path: URL) =>
  [
    ...readFileSync(path, "utf8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .matchAll(/([^{}]+)\{([^{}]*)\}/g),
  ].map(([, selector, body]) => ({
    selector: selector.trim(),
    properties: body
      .split(";")
      .map((line) => line.split(":")[0].trim())
      .filter(Boolean),
  }));
const INDICATOR_CSS = cssRules(new URL("./indicator.css", import.meta.url));
const OVERLAY_CSS_PATH = new URL("./RecordingOverlay.css", import.meta.url);

describe("rollout gates closed (destination-return, smart-paste-dispatch)", () => {
  for (const form of ["compact", "live"] as const) {
    test(`${form}: the pill looks as on dev (same card, direction and classes around it)`, () => {
      const rendered = pillLook(render({ form }));
      expect(rendered.pill).toBe(true);
      expect(rendered).toEqual(pillLook(devOverlay(form, PILL)));
    });
  }

  test("the added stack wrapper only lays out its children", () => {
    const layout = new Set([
      "flex",
      "display",
      "flex-direction",
      "align-items",
      "gap",
    ]);
    const stackRules = INDICATOR_CSS.filter((rule) =>
      rule.selector.includes(".ov-stack"),
    );
    expect(stackRules.length).toBeLessThan(3);
    for (const rule of stackRules) {
      expect(rule.properties.filter((name) => !layout.has(name))).toEqual([]);
    }
  });

  test("no other indicator rule can reach the pill alone, and no overlay token is redefined", () => {
    const markup = render();
    const present = new Set(
      [...markup.matchAll(/class="([^"]*)"/g)].flatMap((match) =>
        match[1].split(" "),
      ),
    );
    for (const rule of INDICATOR_CSS) {
      if (rule.selector === ":root" || rule.selector.includes(".ov-stack"))
        continue;
      for (const selector of rule.selector.split(",")) {
        const classes = [...selector.matchAll(/\.([\w-]+)/g)].map(
          (match) => match[1],
        );
        expect(classes.some((name) => !present.has(name))).toBe(true);
      }
    }
    const overlayTokens = readFileSync(OVERLAY_CSS_PATH, "utf8");
    const root = INDICATOR_CSS.find((rule) => rule.selector === ":root");
    for (const token of root?.properties ?? []) {
      expect(token.startsWith("--")).toBe(true);
      expect(overlayTokens.includes(`${token}:`)).toBe(false);
    }
  });

  test("a session with the setting off shows no badge either", () => {
    const off = { ...session(chatgpt), return_enabled: false };
    expect(render({ destination: destination(off) })).not.toContain(
      "recording-indicator-destination",
    );
  });
});

// ---- The pill alone fits the base window of its form ----

/** A pixel value of RecordingOverlay.css, by custom property or by rule and property. */
const overlayCss = readFileSync(OVERLAY_CSS_PATH, "utf8");
const cssVar = (name: string): number =>
  Number(new RegExp(`${name}:\\s*(\\d+)px`).exec(overlayCss)?.[1]);
const ruleValue = (selector: string, property: string): number => {
  const body =
    new RegExp(`\\n${selector.replace(".", "\\.")} \\{([^}]*)\\}`).exec(
      overlayCss,
    )?.[1] ?? "";
  return Number(new RegExp(`${property}:\\s*(\\d+)px`).exec(body)?.[1]);
};
/** A window size constant of src-tauri/src/overlay.rs (logical points). */
const overlayRs = readFileSync(
  new URL("../../src-tauri/src/overlay.rs", import.meta.url),
  "utf8",
);
const rustSize = (name: string): number =>
  Number(new RegExp(`const ${name}: f64 = ([\\d.]+);`).exec(overlayRs)?.[1]);

describe("indicator alone in the base window (no growth, as on dev)", () => {
  // .scard has a 1px border on each side (content-box sizing).
  const border = 2 * ruleValue(".scard", "border");
  const baseRow = cssVar("--ov-base-h");

  test("geometry is read from the stylesheet and overlay.rs", () => {
    for (const value of [
      border,
      baseRow,
      cssVar("--ov-open-w"),
      rustSize("OVERLAY_WIDTH"),
      rustSize("OVERLAY_STREAM_HEIGHT"),
    ]) {
      expect(Number.isFinite(value) && value > 0).toBe(true);
    }
  });

  test("compact pill, resting or working, fits OVERLAY_WIDTH x OVERLAY_HEIGHT", () => {
    const widest = Math.max(cssVar("--ov-rest-w"), cssVar("--ov-work-w"));
    const request = extentRequest(widest + border, baseRow + border);
    expect(request.width).toBeLessThan(rustSize("OVERLAY_WIDTH") + 1);
    expect(request.height).toBeLessThan(rustSize("OVERLAY_HEIGHT") + 1);
  });

  test("Live card, open with its text region full, fits the Live window", () => {
    const windowHeight = rustSize("OVERLAY_STREAM_HEIGHT");
    // .stext-cap: max-height min(--ov-cap-max-h, 100vh - --ov-base-h - 20px), plus its padding.
    const reserve = Number(
      /100vh - var\(--ov-base-h\) - (\d+)px/.exec(overlayCss)?.[1],
    );
    const cap = Math.min(
      cssVar("--ov-cap-max-h"),
      windowHeight - baseRow - reserve,
    );
    const padding =
      ruleValue(".stext-cap", "padding-top") +
      ruleValue(".stext-cap", "padding-bottom");
    const widest = Math.max(
      cssVar("--ov-pill-w"),
      cssVar("--ov-work-w"),
      cssVar("--ov-open-w"),
    );
    const request = extentRequest(
      widest + border,
      baseRow + cap + padding + border,
    );
    expect(Number.isFinite(reserve)).toBe(true);
    expect(request.width).toBeLessThan(rustSize("OVERLAY_STREAM_WIDTH") + 1);
    expect(request.height).toBeLessThan(windowHeight + 1);
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
