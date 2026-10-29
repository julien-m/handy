/**
 * Rendered row of smart paragraphs (016 AC-010): icon, translated label, information and an
 * independent switch, off unless stored on. Run with `bun run test:smart-paragraphs`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
let osType = "macos";
mock.module("@tauri-apps/plugin-os", () => ({ type: () => osType }));
let stored: Record<string, unknown> = {};
mock.module("../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) => stored[key],
    updateSetting: async () => true,
    isUpdating: () => false,
  }),
}));

const { SmartParagraphsToggle } = await import("./SmartParagraphsToggle");

/** The switch's input element, whatever its attribute order. */
const switchInput = (markup: string): string =>
  markup.match(/<input[^>]*smart-paragraphs-toggle[^>]*>/)?.[0] ?? "";

describe("smart paragraphs row", () => {
  test("shows the icon, the label, the information and an off switch by default", () => {
    stored = {};
    const markup = renderToStaticMarkup(<SmartParagraphsToggle grouped />);
    expect(markup).toContain('data-testid="smart-paragraphs-icon"');
    expect(markup).toContain("¶");
    expect(markup).toContain("settings.advanced.smartParagraphs.title");
    expect(switchInput(markup)).not.toBe("");
    expect(switchInput(markup)).not.toContain("checked");
  });
  test("reflects the stored value", () => {
    stored = { smart_paragraphs_enabled: true };
    const markup = renderToStaticMarkup(<SmartParagraphsToggle grouped />);
    expect(switchInput(markup)).toContain("checked");
  });
  test("the description sits behind the information button on every system", () => {
    stored = {};
    for (const system of ["macos", "windows", "linux"]) {
      osType = system;
      const markup = renderToStaticMarkup(<SmartParagraphsToggle grouped />);
      expect(markup).toContain('aria-label="More information"');
    }
    osType = "macos";
  });
});
