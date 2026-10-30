/**
 * Keyboard access to the settings dropdown (feature 007, is_keyboard_navigable behavior): arrows
 * and Enter pick an option, disabled options are skipped, Escape and Tab close without choosing.
 * Run with `bun test src/components/ui/dropdown-navigation.test.ts`.
 */
import { describe, expect, test } from "bun:test";
import {
  dropdownKeyAction,
  initialHighlight,
  nextEnabledIndex,
  type DropdownKeyAction,
  type NavigableOption,
} from "./dropdown-navigation";

const macOptions: NavigableOption[] = [
  { value: "smart_paste" },
  { value: "ctrl_v" },
  { value: "direct", disabled: true },
  { value: "none" },
];
const show = (action: DropdownKeyAction): string => JSON.stringify(action);

/** Press `keys` in order from a closed list; return the chosen value, if any. */
const press = (keys: string[], selected: string | null): string | null => {
  let isOpen = false;
  let highlighted = -1;
  for (const key of keys) {
    const action = dropdownKeyAction(
      key,
      isOpen,
      highlighted,
      macOptions,
      selected,
    );
    if (action.kind === "select") return action.value;
    if (action.kind === "open" || action.kind === "move") {
      isOpen = true;
      highlighted = action.highlighted;
    } else if (action.kind === "close") {
      isOpen = false;
    }
  }
  return null;
};

describe("change the method with the keyboard", () => {
  test("arrow keys then Enter pick smart paste", () => {
    expect(press(["ArrowDown", "ArrowUp", "Enter"], "ctrl_v")).toBe(
      "smart_paste",
    );
  });
  test("Space also picks the highlighted option", () => {
    expect(press(["Enter", "End", " "], "ctrl_v")).toBe("none");
  });
  test("disabled options are skipped in both directions", () => {
    expect(nextEnabledIndex(macOptions, 1, 1)).toBe(3);
    expect(nextEnabledIndex(macOptions, 3, -1)).toBe(1);
  });
  test("the highlight stays at either end", () => {
    expect(nextEnabledIndex(macOptions, 0, -1)).toBe(0);
    expect(nextEnabledIndex(macOptions, 3, 1)).toBe(3);
  });
  test("the list opens on the stored value, else on the first enabled option", () => {
    expect(initialHighlight(macOptions, "none")).toBe(3);
    expect(initialHighlight(macOptions, "direct")).toBe(0);
    expect(initialHighlight(macOptions, null)).toBe(0);
  });
});

describe("closing without choosing", () => {
  test("Escape and Tab close the open list", () => {
    expect(show(dropdownKeyAction("Escape", true, 1, macOptions, null))).toBe(
      show({ kind: "close" }),
    );
    expect(show(dropdownKeyAction("Tab", true, 1, macOptions, null))).toBe(
      show({ kind: "close" }),
    );
  });
  test("Tab on a closed list is left to the browser", () => {
    expect(dropdownKeyAction("Tab", false, -1, macOptions, null).kind).toBe(
      "ignore",
    );
  });
  test("no option is chosen when Escape comes before Enter", () => {
    expect(press(["ArrowDown", "Escape"], "ctrl_v")).toBe(null);
  });
  test("an empty list never selects", () => {
    expect(show(dropdownKeyAction("Enter", true, -1, [], null))).toBe(
      show({ kind: "close" }),
    );
  });
});
