/**
 * Rendered Text formatting section (015, with the 016 smart paragraphs row and the 017 smart
 * spacing row first): one independent
 * switch row per option, in display order, with its icon, its info button, the stored value and
 * the screen-prefixed test ids. Run with `bun run test:text-formatting` (also part of
 * `bun run test:smart-paragraphs`).
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ReactElement } from "react";
import type { OutputScreen } from "../output-screen";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
let osType = "macos";
mock.module("@tauri-apps/plugin-os", () => ({ type: () => osType }));
let stored: Record<string, unknown> = {};
mock.module("../../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) => stored[key],
    updateSetting: async () => true,
    isUpdating: () => false,
  }),
}));

const { TextFormattingSection } = await import("./text-formatting-section");
const { TextFormattingRow } = await import("./text-formatting-row");
const { TEXT_FORMATTING_OPTIONS } = await import("./text-formatting-options");
const { OutputScreenContext } = await import("../output-screen");
const { smartParagraphsDescriptionKey } = await import(
  "../smart-paragraphs-state"
);

/** The option of `key`, failing the test when the section does not list it. */
const optionOf = (key: string) => {
  const option = TEXT_FORMATTING_OPTIONS.find((entry) => entry.key === key);
  if (!option) throw new Error(`${key} is not a text formatting option`);
  return option;
};

const renderIn = (screen: OutputScreen, node: ReactElement): string =>
  renderToStaticMarkup(
    <OutputScreenContext.Provider value={{ screen, report: () => undefined }}>
      {node}
    </OutputScreenContext.Provider>,
  );

/** The switch input of the row `testId`, whatever its attribute order. */
const switchOf = (markup: string, testId: string): string => {
  const end = markup.indexOf(`data-testid="${testId}-toggle"`);
  return markup.slice(
    markup.lastIndexOf("<input", end),
    markup.indexOf(">", end),
  );
};

describe("text formatting section (015)", () => {
  test("one row per option with its test ids, title and info button", () => {
    stored = {};
    const markup = renderIn("output-settings", <TextFormattingSection />);
    expect(markup).toContain('data-testid="output-settings-text-formatting"');
    expect(markup).toContain("settings.advanced.groups.textFormatting");
    for (const option of TEXT_FORMATTING_OPTIONS) {
      expect(markup).toContain(
        `data-testid="output-settings-${option.testId}"`,
      );
      expect(markup).toContain(option.labelKey);
    }
    expect(markup).toContain('aria-label="More information"');
    expect(markup).toContain('aria-hidden="true"');
  });

  test("smart capitalization reflects the stored value", () => {
    const option = optionOf("smart_capitalization");
    stored = { smart_capitalization: true };
    const on = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(switchOf(on, "output-settings-smart-capitalization")).toContain(
      "checked",
    );
    stored = { smart_capitalization: false };
    const off = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(switchOf(off, "output-settings-smart-capitalization")).not.toContain(
      "checked",
    );
  });

  test("test ids follow the screen shown", () => {
    const markup = renderIn("settings-write-errors", <TextFormattingSection />);
    expect(markup).toContain(
      'data-testid="settings-write-errors-smart-capitalization-toggle"',
    );
  });

  // @spec AC-014: First row of Text formatting — .specs/features/017-smart-spacing/spec.md#ac-014
  test("rows follow the planned order: spacing, capitalization, then paragraphs", () => {
    stored = {};
    const markup = renderIn("output-settings", <TextFormattingSection />);
    const spacing = markup.indexOf(
      'data-testid="output-settings-smart-spacing"',
    );
    const capitalization = markup.indexOf(
      'data-testid="output-settings-smart-capitalization"',
    );
    const paragraphs = markup.indexOf(
      'data-testid="output-settings-smart-paragraphs"',
    );
    expect(spacing >= 0).toBe(true);
    expect(capitalization > spacing).toBe(true);
    expect(paragraphs > capitalization).toBe(true);
    expect(TEXT_FORMATTING_OPTIONS[0]?.key).toBe("smart_spacing");
  });

  test("smart spacing: space icon, label, info and the stored value (017)", () => {
    const option = optionOf("smart_spacing");
    stored = { smart_spacing: true };
    const on = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(on).toContain("lucide-space");
    expect(on).toContain("settings.advanced.textFormatting.smartSpacing.label");
    expect(on).toContain('aria-label="More information"');
    expect(switchOf(on, "output-settings-smart-spacing")).toContain("checked");
    stored = { smart_spacing: false };
    const off = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(switchOf(off, "output-settings-smart-spacing")).not.toContain(
      "checked",
    );
    expect(option.descriptionKey("macos")).toBe(
      "settings.advanced.textFormatting.smartSpacing.description",
    );
  });

  test("smart paragraphs: pilcrow icon, label, off unless stored on (016)", () => {
    const option = optionOf("smart_paragraphs_enabled");
    stored = {};
    const off = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(off).toContain("lucide-pilcrow");
    expect(off).toContain("settings.advanced.smartParagraphs.title");
    expect(switchOf(off, "output-settings-smart-paragraphs")).not.toContain(
      "checked",
    );
    stored = { smart_paragraphs_enabled: true };
    const on = renderIn(
      "output-settings",
      <TextFormattingRow option={option} />,
    );
    expect(switchOf(on, "output-settings-smart-paragraphs")).toContain(
      "checked",
    );
  });

  test("smart paragraphs describe the platform limitation outside macOS (016)", () => {
    const option = optionOf("smart_paragraphs_enabled");
    stored = {};
    for (const system of ["macos", "windows", "linux"]) {
      osType = system;
      expect(option.descriptionKey(system)).toBe(
        smartParagraphsDescriptionKey(system),
      );
      const markup = renderIn(
        "output-settings",
        <TextFormattingRow option={option} />,
      );
      expect(markup).toContain('aria-label="More information"');
    }
    osType = "macos";
  });

  test("keys are unique, so each row has its own switch", () => {
    const keys = TEXT_FORMATTING_OPTIONS.map((option) => option.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});
