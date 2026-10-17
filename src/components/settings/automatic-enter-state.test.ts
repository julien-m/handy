/**
 * Automatic Enter selector of the output settings (009 AC-004, Behavioral AC).
 * Behavioral patterns from: system/testing/ui-behavioral-taxonomy.md
 * Traits: has_dropdown, is_keyboard_navigable
 * Run with `bun run test:output-settings`.
 */
import { describe, expect, test } from "bun:test";
import en from "@/i18n/locales/en/translation.json";
import { dropdownKeyAction } from "../ui/dropdown-navigation";
import {
  automaticEnterTestIds,
  automaticEnterOptions,
  enterControl,
  enterKeyOptions,
} from "./automatic-enter-state";

const translationOf = (key: string): unknown =>
  key
    .split(".")
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown> | undefined)?.[part],
      en,
    );

describe("which Enter control the output settings show", () => {
  test("gate open (automatic_enter in the snapshot): the three-value selector only", () => {
    expect(enterControl({ automatic_enter: "never" })).toBe("automatic_enter");
  });
  test("gate closed (no automatic_enter): dev's auto-submit control only", () => {
    expect(enterControl({})).toBe("auto_submit");
    expect(enterControl({ automatic_enter: null })).toBe("auto_submit");
  });
});

describe("Automatic Enter selector (has_dropdown)", () => {
  test("Never, Always and Only in PIN mode are listed with the current value marked", () => {
    const options = automaticEnterOptions("pin_only");
    expect(options.map((option) => option.value)).toEqual([
      "never",
      "always",
      "pin_only",
    ]);
    expect(options.map((option) => translationOf(option.labelKey))).toEqual([
      "Never",
      "Always",
      "Only in PIN mode",
    ]);
    expect(options.filter((option) => option.selected).length).toBe(1);
    expect(options.find((option) => option.selected)?.value).toBe("pin_only");
    expect(options.map((option) => option.testId)).toEqual([
      "output-settings-enter-options-auto-enter-never",
      "output-settings-enter-options-auto-enter-always",
      "output-settings-enter-options-auto-enter-pin-only",
    ]);
  });
  test("the key selector keeps Enter, Ctrl+Enter and Cmd/Super+Enter", () => {
    expect(enterKeyOptions("macos").map((option) => option.value)).toEqual([
      "enter",
      "ctrl_enter",
      "cmd_enter",
    ]);
    expect(enterKeyOptions("windows")[2].labelKey).toBe(
      "settings.advanced.autoSubmit.options.superEnter",
    );
  });
  test("contract test ids follow the output screen shown", () => {
    const base = automaticEnterTestIds("output-settings");
    expect(base.row).toBe("output-settings-auto-enter");
    expect(base.mode).toBe("output-settings-auto-enter-mode");
    expect(base.key).toBe("output-settings-auto-enter-key");
    const open = automaticEnterTestIds("output-settings-enter-options");
    expect(open.mode).toBe("output-settings-enter-options-auto-enter-mode");
    expect(open.options).toBe(
      "output-settings-enter-options-auto-enter-options",
    );
  });
});

describe("keyboard access to Automatic Enter (is_keyboard_navigable)", () => {
  test("arrow keys then Enter pick Only in PIN mode", () => {
    const options = automaticEnterOptions("never").map((option) => ({
      value: option.value,
      label: option.value,
    }));
    const opened = dropdownKeyAction("ArrowDown", false, -1, options, "never");
    expect(opened.kind).toBe("open");
    const highlighted = opened.kind === "open" ? opened.highlighted : -1;
    const down1 = dropdownKeyAction(
      "ArrowDown",
      true,
      highlighted,
      options,
      "never",
    );
    const index1 = down1.kind === "move" ? down1.highlighted : -1;
    const down2 = dropdownKeyAction(
      "ArrowDown",
      true,
      index1,
      options,
      "never",
    );
    const index2 = down2.kind === "move" ? down2.highlighted : -1;
    expect(dropdownKeyAction("Enter", true, index2, options, "never")).toEqual({
      kind: "select",
      value: "pin_only",
    });
  });
  test("Tab keeps its default so focus moves on in reading order", () => {
    const options = automaticEnterOptions("never").map((option) => ({
      value: option.value,
      label: option.value,
    }));
    expect(dropdownKeyAction("Tab", false, -1, options, "never").kind).toBe(
      "ignore",
    );
  });
});
