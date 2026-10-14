/**
 * Paste method selector entries per platform and smart paste availability (feature 007,
 * has_dropdown behavior). Run with `bun test src/components/settings/paste-method-options.test.ts`.
 */
import { describe, expect, test } from "bun:test";
import {
  pasteMethodChoices,
  showsReliablePasteToggle,
} from "./paste-method-options";

const values = (...args: Parameters<typeof pasteMethodChoices>): string =>
  pasteMethodChoices(...args)
    .map((choice) => `${choice.value}${choice.disabled ? "(disabled)" : ""}`)
    .join(",");

describe("smart paste listed on macOS", () => {
  test("listed first and selectable once the backend offers it", () => {
    expect(values("macos", "ctrl_v", true)).toBe("smart_paste,ctrl_v,none");
  });
  test("the current value is the stored one, not a loading fallback", () => {
    expect(values("macos", null, true)).toBe("smart_paste,ctrl_v,none");
  });
  test("its label is the translated smart paste option", () => {
    const [first] = pasteMethodChoices("macos", "smart_paste", true);
    expect(first.labelKey).toBe(
      "settings.advanced.pasteMethod.options.smartPaste",
    );
  });
  test("not listed while dictation cannot deliver through it", () => {
    expect(values("macos", "ctrl_v", false)).toBe("ctrl_v,none");
  });
  test("a stored smart paste stays visible but disabled when unavailable", () => {
    expect(values("macos", "smart_paste", false)).toBe(
      "smart_paste(disabled),ctrl_v,none",
    );
  });
});

describe("smart paste absent elsewhere", () => {
  test("Windows keeps its methods even if the backend said available", () => {
    expect(values("windows", "ctrl_v", true)).toBe(
      "ctrl_v,direct,none,ctrl_shift_v,shift_insert",
    );
  });
  test("Linux keeps its methods", () => {
    expect(values("linux", "direct", true)).toBe(
      "ctrl_v,direct,none,ctrl_shift_v,shift_insert,external_script",
    );
  });
  test("direct stays visible but disabled when stored on macOS", () => {
    expect(values("macos", "direct", false)).toBe(
      "ctrl_v,direct(disabled),none",
    );
  });
});

describe("option labels, descriptions and test ids (mockup)", () => {
  test("smart paste and None carry their second line", () => {
    const choices = pasteMethodChoices("macos", "smart_paste", true);
    const described = choices.map((entry) => entry.descriptionKey ?? "-");
    expect(described).toEqual([
      "settings.advanced.pasteMethod.options.smartPasteDescription",
      "-",
      "settings.advanced.pasteMethod.options.noneDescription",
    ]);
  });
  test("macOS shows the Command symbol, other systems Ctrl+V", () => {
    const [, mac] = pasteMethodChoices("macos", "ctrl_v", true);
    expect(mac.labelKey).toBe(
      "settings.advanced.pasteMethod.options.clipboardMac",
    );
    const [windows] = pasteMethodChoices("windows", "ctrl_v", true);
    expect(windows.labelKey).toBe(
      "settings.advanced.pasteMethod.options.clipboard",
    );
  });
  test("slugs follow the contract option ids", () => {
    const linux = pasteMethodChoices("linux", "ctrl_v", false);
    expect(linux.map((entry) => entry.slug)).toEqual([
      "clipboard",
      "direct",
      "none",
      "clipboard-ctrl-shift-v",
      "clipboard-shift-insert",
      "script",
    ]);
    expect(pasteMethodChoices("macos", null, true)[0].slug).toBe("smart");
  });
});

describe("beta reliable paste switch", () => {
  test("hidden on macOS only once smart paste replaces it", () => {
    expect(showsReliablePasteToggle("macos", true)).toBe(false);
    expect(showsReliablePasteToggle("macos", false)).toBe(true);
  });
  test("kept on Windows and absent on Linux", () => {
    expect(showsReliablePasteToggle("windows", true)).toBe(true);
    expect(showsReliablePasteToggle("linux", false)).toBe(false);
  });
});
