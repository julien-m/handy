/**
 * Rendered lists of the keys used during a dictation (014): one key or none, with Change,
 * Remove, Add (when empty) and Reset, and no hint. Run with `bun run test:shortcut-states`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ShortcutBinding } from "@/bindings";

// Translations render as "key params" so the markup shows which text is used.
mock.module("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: unknown) =>
      typeof params === "object" ? `${key} ${JSON.stringify(params)}` : key,
  }),
}));
mock.module("@tauri-apps/plugin-os", () => ({ type: () => "macos" }));
mock.module("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
mock.module("@/bindings", () => ({
  commands: { getShortcutConflicts: async () => [] },
}));

const binding = (
  id: string,
  current: string,
  defaultKey: string,
): ShortcutBinding => ({
  id,
  name: id,
  description: `${id} description`,
  default_binding: defaultKey,
  current_binding: current,
  additional_bindings: [],
});
const bindings: Record<string, ShortcutBinding> = {
  transcribe: binding("transcribe", "alt+space", "alt+space"),
  skip_automatic_enter: binding(
    "skip_automatic_enter",
    "control_left",
    "control_left",
  ),
  retarget_to_active_app: binding("retarget_to_active_app", "", "option_left"),
};
mock.module("../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) =>
      key === "bindings"
        ? bindings
        : key === "keyboard_implementation"
          ? "handy_keys"
          : undefined,
    editBindingShortcuts: async () => null,
    refusedShortcuts: [],
    isUpdating: () => false,
    isLoading: false,
  }),
}));

const { ShortcutList } = await import("./shortcut-list");
const { shortcutTestIds } = await import("./shortcut-list-feedback");

const count = (markup: string, text: string): number =>
  markup.split(text).length - 1;

// @spec AC-005: In-dictation keys in the shortcut settings — .specs/features/014-in-dictation-keys/spec.md#ac-005
describe("keys used during a dictation (shortcut-settings)", () => {
  test("one key: Change, Remove and Reset, Add unavailable, no hint", () => {
    const markup = renderToStaticMarkup(
      <ShortcutList actionId="skip_automatic_enter" grouped />,
    );
    const ids = shortcutTestIds("skip_automatic_enter");
    expect(count(markup, `data-testid="${ids.shortcut}"`)).toBe(1);
    expect(markup.includes(`data-testid="${ids.remove}"`)).toBe(true);
    expect(markup.includes(`data-testid="${ids.reset}"`)).toBe(true);
    expect(
      markup.includes(`data-testid="${ids.add}" data-state="disabled"`),
    ).toBe(true);
    expect(markup.includes(`data-testid="${ids.last}"`)).toBe(false);
    expect(markup.includes(`data-testid="${ids.limit}"`)).toBe(false);
  });

  test("a removed key leaves Add and Reset", () => {
    const markup = renderToStaticMarkup(
      <ShortcutList actionId="retarget_to_active_app" grouped />,
    );
    const ids = shortcutTestIds("retarget_to_active_app");
    expect(markup.includes(`data-testid="${ids.shortcut}"`)).toBe(false);
    expect(
      markup.includes(`data-testid="${ids.add}" data-state="enabled"`),
    ).toBe(true);
    expect(markup.includes(`data-testid="${ids.reset}"`)).toBe(true);
  });
});
