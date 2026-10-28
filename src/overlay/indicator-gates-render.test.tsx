/**
 * The recording indicator with the rollout gates closed (`destination-return`,
 * `smart-paste-dispatch`): no destination session, and no notice but the recording cut short by
 * a lost microphone (001, native session N1). The rendered pill is compared with
 * the pill of `dev`, and the pill alone must fit the base window of its form (compact and Live).
 * Run with `bun run test:indicator`.
 */
import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, test } from "bun:test";
import {
  PILL,
  chatgpt,
  destination,
  initIndicatorI18n,
  render,
  session,
} from "./indicator-test-fixtures";
import { inputLostNotice } from "./notice-model";
import { extentRequest } from "./use-overlay-extent";

beforeAll(initIndicatorI18n);

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

  test("the input-lost notice is the only addition, beside the unchanged pill", () => {
    const html = render({ notices: [inputLostNotice(0)] });
    expect(html).toContain(PILL);
    expect([...html.matchAll(/class="snotice /g)].length).toBe(1);
    expect(html).toContain(
      'data-testid="recording-indicator-input-lost-notice"',
    );
    expect(html).not.toContain("recording-indicator-destination");
    expect(html).not.toContain("recording-indicator-prompt");
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
