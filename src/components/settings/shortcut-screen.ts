// Contract screen shown by the shortcut settings (feature 004). Every test id of the page carries
// the prefix of the screen actually shown (`shortcut-settings`, `-conflict` or `-limit` in
// `penflow/flow-ui-contract/contract.json`), so each test id is unique in the tree. Each list
// reports its own state; the page shows the conflict screen while any list shows a conflict, a
// refusal or an inherited conflict, else the limit screen while any list is full.
import { createContext, useContext, useEffect } from "react";

/** Test-id prefix of a contract screen of the shortcut settings. */
export type ShortcutScreen =
  | "shortcut-settings"
  | "shortcut-settings-conflict"
  | "shortcut-settings-limit";

/** State of one list that selects a screen; null when it selects the base screen. */
export type ShortcutListState = "conflict" | "limit" | null;

/** The page's screen and how each list reports its own state. */
export interface ShortcutScreenScope {
  screen: ShortcutScreen;
  report: (actionId: string, state: ShortcutListState) => void;
}

/** Screen of the page that holds the lists; null outside it (a list then decides alone). */
export const ShortcutScreenContext = createContext<ShortcutScreenScope | null>(
  null,
);

/**
 * Screen shown for the reported list states: conflict first, then limit.
 * @param states state of each list
 * @returns the test-id prefix of the screen shown
 */
export const shortcutScreenOf = (
  states: readonly ShortcutListState[],
): ShortcutScreen => {
  if (states.includes("conflict")) return "shortcut-settings-conflict";
  if (states.includes("limit")) return "shortcut-settings-limit";
  return "shortcut-settings";
};

/**
 * Test-id prefix of the screen shown, for a part of the page that has no state of its own.
 * @returns the page's screen, or the base screen outside it
 */
export const useShortcutScreen = (): ShortcutScreen =>
  useContext(ShortcutScreenContext)?.screen ?? "shortcut-settings";

/**
 * Reports one list's state to the page and returns the screen shown. The list's own state also
 * counts at once, so its own test ids never wait for the page to render again.
 * Side effects: reports `state` for `actionId` while mounted, and null when unmounted.
 * @param actionId the list's action
 * @param state the list's own state
 * @returns the test-id prefix of the screen shown
 */
export const useReportedShortcutScreen = (
  actionId: string,
  state: ShortcutListState,
): ShortcutScreen => {
  const scope = useContext(ShortcutScreenContext);
  const report = scope?.report;
  useEffect(() => {
    report?.(actionId, state);
  }, [report, actionId, state]);
  useEffect(() => () => report?.(actionId, null), [report, actionId]);
  const page = scope?.screen ?? "shortcut-settings";
  return shortcutScreenOf([pageState(page), state]);
};

/**
 * State that selects `screen`.
 * @param screen a screen
 * @returns the list state that selects it
 */
const pageState = (screen: ShortcutScreen): ShortcutListState => {
  if (screen === "shortcut-settings-conflict") return "conflict";
  if (screen === "shortcut-settings-limit") return "limit";
  return null;
};
