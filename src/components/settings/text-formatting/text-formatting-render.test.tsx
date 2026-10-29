/**
 * Rendered Text formatting section (015): one independent switch row per option, with its icon,
 * its info button, the stored value and the screen-prefixed test ids. Run with
 * `bun run test:text-formatting`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ReactElement } from "react";
import type { OutputScreen } from "../output-screen";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
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
    const option = TEXT_FORMATTING_OPTIONS[0];
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

  test("keys are unique, so each row has its own switch", () => {
    const keys = TEXT_FORMATTING_OPTIONS.map((option) => option.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});
