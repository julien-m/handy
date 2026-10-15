/**
 * Rendered states of the output settings (006 return to the dictation's application, 007 paste
 * method) and of the settings write errors (008): contract test ids, the unavailable reason, the
 * revision conflict alert and the history limit field. Run with `bun run test:output-settings`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { DestinationSupportState } from "../../hooks/use-destination-support";

mock.module("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
mock.module("@tauri-apps/plugin-os", () => ({ type: () => "macos" }));

let support: DestinationSupportState = undefined;
mock.module("../../hooks/use-destination-support", () => ({
  useDestinationSupport: () => support,
}));
mock.module("../../hooks/use-smart-paste-availability", () => ({
  useSmartPasteAvailability: () => true,
}));
let stored: Record<string, unknown> = {};
mock.module("../../hooks/useSettings", () => ({
  useSettings: () => ({
    getSetting: (key: string) => stored[key],
    updateSetting: async () => true,
    isUpdating: () => false,
  }),
}));
let revisionConflict = false;
let conflictReloadFailed = false;
mock.module("../../stores/settingsStore", () => ({
  useSettingsStore: (select: (state: object) => unknown) =>
    select({
      revisionConflict,
      conflictReloadFailed,
      reloadAfterConflict: async () => undefined,
    }),
}));

const { ReturnToAppSetting } = await import("./return-to-app-setting");
const { PasteMethodSetting } = await import("./PasteMethod");
const { SettingsConflictAlert } = await import("./settings-conflict-alert");
const { HistoryLimit } = await import("./HistoryLimit");
const { AutomaticEnter } = await import("./automatic-enter");

const TOGGLE = 'data-testid="output-settings-return-app-toggle"';
/** The toggle's input element, whatever its attribute order. */
const toggleOf = (markup: string): string => {
  const end = markup.indexOf(TOGGLE);
  return markup.slice(
    markup.lastIndexOf("<input", end),
    markup.indexOf(">", end),
  );
};

describe("return to the dictation's application (006)", () => {
  test("gate closed: no row at all, as on dev", () => {
    support = "not_offered";
    stored = { return_to_dictation_app: true };
    expect(renderToStaticMarkup(<ReturnToAppSetting grouped />)).toBe("");
  });

  test("support not answered yet: no row", () => {
    support = undefined;
    expect(renderToStaticMarkup(<ReturnToAppSetting grouped />)).toBe("");
  });

  test("Linux (gate open): unavailable, off, with the reason, info still offered", () => {
    support = { state: "unsupported", reason: "platform_unsupported" };
    stored = { return_to_dictation_app: true };
    const markup = renderToStaticMarkup(<ReturnToAppSetting grouped />);
    expect(markup.includes('data-testid="output-settings-return-app"')).toBe(
      true,
    );
    expect(
      markup.includes('data-testid="output-settings-return-app-title"'),
    ).toBe(true);
    expect(
      markup.includes("settings.advanced.returnToApp.unavailablePlatform"),
    ).toBe(true);
    expect(
      markup.includes('data-testid="output-settings-return-app-info"'),
    ).toBe(true);
    const toggle = toggleOf(markup);
    expect(toggle.includes("disabled")).toBe(true);
    expect(toggle.includes("checked")).toBe(false);
  });

  test("frontmost app unreadable: its own reason", () => {
    support = { state: "unsupported", reason: "frontmost_unreadable" };
    const markup = renderToStaticMarkup(<ReturnToAppSetting grouped />);
    expect(
      markup.includes("settings.advanced.returnToApp.unavailableUnreadable"),
    ).toBe(true);
  });

  test("supported: the stored value, usable", () => {
    support = { state: "supported" };
    stored = { return_to_dictation_app: true };
    const markup = renderToStaticMarkup(<ReturnToAppSetting grouped />);
    expect(markup.includes("settings.advanced.returnToApp.description")).toBe(
      true,
    );
    const toggle = toggleOf(markup);
    expect(toggle.includes("checked")).toBe(true);
    expect(toggle.includes("disabled")).toBe(false);
  });
});

describe("paste method selector (007)", () => {
  test("smart paste offered: its description inline and the contract ids", () => {
    stored = { paste_method: "smart_paste" };
    const markup = renderToStaticMarkup(
      <PasteMethodSetting descriptionMode="inline" grouped />,
    );
    expect(markup.includes('data-testid="output-settings-paste-method"')).toBe(
      true,
    );
    expect(
      markup.includes('data-testid="output-settings-paste-method-select"'),
    ).toBe(true);
    expect(
      markup.includes("settings.advanced.pasteMethod.descriptionSmart"),
    ).toBe(true);
    expect(
      markup.includes(
        ">settings.advanced.pasteMethod.options.smartPaste</span>",
      ),
    ).toBe(true);
  });
});

describe("settings write errors (008)", () => {
  test("no conflict: nothing shown", () => {
    revisionConflict = false;
    expect(renderToStaticMarkup(<SettingsConflictAlert />)).toBe("");
  });

  test("revision conflict: the alert with its title and Reload", () => {
    revisionConflict = true;
    const markup = renderToStaticMarkup(<SettingsConflictAlert />);
    expect(
      markup.includes('data-testid="settings-write-errors-conflict"'),
    ).toBe(true);
    expect(
      markup.includes('data-testid="settings-write-errors-conflict-title"'),
    ).toBe(true);
    expect(markup.includes('data-testid="settings-write-errors-reload"')).toBe(
      true,
    );
    expect(markup.includes("errors.settings.conflictBody")).toBe(true);
    expect(markup.includes("errors.settings.reloadFailed")).toBe(false);
  });

  test("a failed Reload: the alert stays with the failure, no false success", () => {
    revisionConflict = true;
    conflictReloadFailed = true;
    const markup = renderToStaticMarkup(<SettingsConflictAlert />);
    expect(
      markup.includes('data-testid="settings-write-errors-conflict"'),
    ).toBe(true);
    expect(
      markup.includes(
        'data-testid="settings-write-errors-conflict-reload-failed"',
      ),
    ).toBe(true);
    expect(markup.includes("errors.settings.reloadFailed")).toBe(true);
    conflictReloadFailed = false;
  });

  test("a stored history limit shows no error", () => {
    stored = { history_limit: 5 };
    const markup = renderToStaticMarkup(<HistoryLimit grouped />);
    expect(
      markup.includes(
        'data-testid="settings-write-errors-history-limit-input"',
      ),
    ).toBe(true);
    expect(markup.includes("settings-write-errors-history-limit-error")).toBe(
      false,
    );
    expect(markup.includes("aria-invalid")).toBe(false);
  });
});

// @spec AC-004: One Automatic Enter control with its key — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
describe("Automatic Enter (009, gate destination-return open)", () => {
  test("the selector, the key selector and the note carry the contract test ids", () => {
    stored = { automatic_enter: "pin_only", auto_submit_key: "ctrl_enter" };
    const markup = renderToStaticMarkup(<AutomaticEnter grouped />);
    for (const id of [
      "output-settings-enter-options-auto-enter",
      "output-settings-enter-options-auto-enter-title",
      "output-settings-enter-options-auto-enter-mode",
      "output-settings-enter-options-auto-enter-key",
      "output-settings-enter-options-auto-enter-note",
    ]) {
      expect(markup.includes(`data-testid="${id}"`)).toBe(true);
    }
    expect(
      markup.includes("settings.advanced.automaticEnter.options.pinOnly"),
    ).toBe(true);
    expect(
      markup.includes("settings.advanced.autoSubmit.options.ctrlEnter"),
    ).toBe(true);
    expect(markup.includes("settings.advanced.autoSubmit.title")).toBe(false);
  });
  test("both selectors are focusable comboboxes in reading order", () => {
    stored = { automatic_enter: "never", auto_submit_key: "enter" };
    const markup = renderToStaticMarkup(<AutomaticEnter grouped />);
    const mode = markup.indexOf(
      "output-settings-enter-options-auto-enter-mode",
    );
    const key = markup.indexOf("output-settings-enter-options-auto-enter-key");
    expect(mode > -1).toBe(true);
    expect(key > mode).toBe(true);
    expect((markup.match(/role="combobox"/g) ?? []).length).toBe(2);
    expect(markup.includes('role="status"')).toBe(true);
  });
});
