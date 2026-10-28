/**
 * Keyboard behavior of the settings dropdown (select-only combobox pattern): arrows open the list
 * and move a highlight over enabled options, Enter or Space picks the highlighted option, Escape
 * and Tab close without choosing. Pure, so every key path is tested without a DOM.
 */

/** The part of an option that navigation needs. */
export interface NavigableOption {
  value: string;
  disabled?: boolean;
}

/** What the dropdown does for one key press. */
export type DropdownKeyAction =
  | { kind: "open"; highlighted: number }
  | { kind: "move"; highlighted: number }
  | { kind: "select"; value: string }
  | { kind: "close" }
  | { kind: "ignore" };

const isEnabled = (option: NavigableOption | undefined): boolean =>
  option !== undefined && !option.disabled;

/**
 * Next enabled index from `from` in direction `step`, staying on `from` at either end.
 * @returns -1 when no option is enabled
 */
export const nextEnabledIndex = (
  options: NavigableOption[],
  from: number,
  step: 1 | -1,
): number => {
  for (
    let index = from + step;
    index >= 0 && index < options.length;
    index += step
  ) {
    // First enabled option in the direction of travel wins.
    if (isEnabled(options[index])) return index;
  }
  return isEnabled(options[from]) ? from : -1;
};

/** Highlight when the list opens: the selected option, else the first enabled one. */
export const initialHighlight = (
  options: NavigableOption[],
  selectedValue: string | null,
): number => {
  const selected = options.findIndex(
    (option) => option.value === selectedValue,
  );
  return isEnabled(options[selected])
    ? selected
    : nextEnabledIndex(options, -1, 1);
};

const openKeys = new Set(["ArrowDown", "ArrowUp", "Enter", " "]);

/** Action for `key` while the list is shown. */
const openListAction = (
  key: string,
  highlighted: number,
  options: NavigableOption[],
): DropdownKeyAction => {
  switch (key) {
    case "ArrowDown":
      return {
        kind: "move",
        highlighted: nextEnabledIndex(options, highlighted, 1),
      };
    case "ArrowUp":
      return {
        kind: "move",
        highlighted: nextEnabledIndex(options, highlighted, -1),
      };
    case "Home":
      return { kind: "move", highlighted: nextEnabledIndex(options, -1, 1) };
    case "End":
      return {
        kind: "move",
        highlighted: nextEnabledIndex(options, options.length, -1),
      };
    case "Enter":
    case " ":
      return isEnabled(options[highlighted])
        ? { kind: "select", value: options[highlighted].value }
        : { kind: "close" };
    case "Escape":
    case "Tab":
      return { kind: "close" };
    default:
      return { kind: "ignore" };
  }
};

/**
 * Action for `key` in the current state.
 * @param key KeyboardEvent.key
 * @param isOpen whether the list is shown
 * @param highlighted highlighted index (-1 for none)
 * @param options displayed options
 * @param selectedValue stored value, or null while loading
 */
export const dropdownKeyAction = (
  key: string,
  isOpen: boolean,
  highlighted: number,
  options: NavigableOption[],
  selectedValue: string | null,
): DropdownKeyAction => {
  if (isOpen) return openListAction(key, highlighted, options);
  return openKeys.has(key)
    ? { kind: "open", highlighted: initialHighlight(options, selectedValue) }
    : { kind: "ignore" };
};
