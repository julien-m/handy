// Contract screen shown by the shortcut settings (feature 004). Every test id of the page carries
// the prefix of the screen actually shown (`shortcut-settings`, `-conflict` or `-limit` in
// `penflow/flow-ui-contract/contract.json`), so each test id is unique in the tree. Each list
// reports its own state; the page shows the conflict screen while any list shows a conflict, a
// refusal or an inherited conflict, else the limit screen while any list is full.
import { createContext, useContext } from "react";
import {
  type ScreenOrder,
  type ScreenScope,
  firstReported,
  useReportedScreen,
} from "./page-screen";

/** Test-id prefix of a contract screen of the shortcut settings. */
export type ShortcutScreen =
  | "shortcut-settings"
  | "shortcut-settings-conflict"
  | "shortcut-settings-limit";

/** State screen one list shows; null when it shows nothing beyond the base screen. */
export type ShortcutListState = Exclude<
  ShortcutScreen,
  "shortcut-settings"
> | null;

/** Conflict first, then limit. */
export const SHORTCUT_SCREENS: ScreenOrder<ShortcutScreen> = {
  priority: ["shortcut-settings-conflict", "shortcut-settings-limit"],
  base: "shortcut-settings",
};

/** Screen of the page that holds the lists; null outside it (a list then decides alone). */
export const ShortcutScreenContext =
  createContext<ScreenScope<ShortcutScreen> | null>(null);

/**
 * Screen shown for the reported list states.
 * @param states state of each list
 * @returns the test-id prefix of the screen shown
 */
export const shortcutScreenOf = (
  states: readonly ShortcutListState[],
): ShortcutScreen => firstReported(SHORTCUT_SCREENS, states);

/**
 * Test-id prefix of the screen shown, for a part of the page that has no state of its own.
 * @returns the page's screen, or the base screen outside it
 */
export const useShortcutScreen = (): ShortcutScreen =>
  useContext(ShortcutScreenContext)?.screen ?? "shortcut-settings";

/**
 * Reports one list's state to the page and returns the screen shown.
 * Side effects: reports `state` for `actionId` while mounted, and null when unmounted.
 * @param actionId the list's action
 * @param state the list's own state
 * @returns the test-id prefix of the screen shown
 */
export const useReportedShortcutScreen = (
  actionId: string,
  state: ShortcutListState,
): ShortcutScreen =>
  useReportedScreen(ShortcutScreenContext, SHORTCUT_SCREENS, actionId, state);
