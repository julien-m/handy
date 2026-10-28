/**
 * The settings dropdown exposes its role, name, state and current value to assistive
 * technologies and shows a visible keyboard focus (feature 007, is_keyboard_navigable behavior).
 * Run with `bun test src/components/ui/dropdown-aria.test.tsx`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
const { Dropdown } = await import("./Dropdown");

const markup = renderToStaticMarkup(
  <Dropdown
    ariaLabel="Paste Method"
    options={[
      { value: "smart_paste", label: "Smart paste" },
      { value: "ctrl_v", label: "Clipboard (Cmd+V)" },
    ]}
    selectedValue="smart_paste"
    onSelect={() => undefined}
  />,
);

describe("dropdown accessibility", () => {
  test("is a named, collapsed combobox controlling a listbox", () => {
    expect(markup.includes('role="combobox"')).toBe(true);
    expect(markup.includes('aria-label="Paste Method"')).toBe(true);
    expect(markup.includes('aria-haspopup="listbox"')).toBe(true);
    expect(markup.includes('aria-expanded="false"')).toBe(true);
    expect(markup.includes("aria-controls=")).toBe(true);
  });
  test("shows the current value as its text", () => {
    expect(markup.includes(">Smart paste</span>")).toBe(true);
  });
  test("has a visible keyboard focus and stays in the tab order", () => {
    expect(markup.includes("focus-visible:ring-2")).toBe(true);
    expect(markup.includes('tabindex="-1"')).toBe(false);
  });
});
