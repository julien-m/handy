// Contract screen shown by the Advanced settings page, which draws the output screens of 006, 007
// and 009 and the History group of 008 (`penflow/flow-ui-contract/contract.json`). Every test id
// of the Output and History groups carries the prefix of the screen actually shown, so each one
// is unique in the tree: the open paste method list (`output-settings-paste-methods`), the open
// Automatic Enter list (`output-settings-enter-options`), a refused write
// (`settings-write-errors`) or destination return unavailable (`output-settings-unavailable`),
// in that order, else `output-settings`.
import { createContext, useContext } from "react";
import {
  type ScreenOrder,
  type ScreenScope,
  useReportedScreen,
} from "./page-screen";

/** Test-id prefix of a contract screen of the Advanced settings page. */
export type OutputScreen =
  | "output-settings"
  | "output-settings-paste-methods"
  | "output-settings-enter-options"
  | "settings-write-errors"
  | "output-settings-unavailable";

/** An open list first (it is transient), then a refused write, then the unavailable return. */
export const OUTPUT_SCREENS: ScreenOrder<OutputScreen> = {
  priority: [
    "output-settings-paste-methods",
    "output-settings-enter-options",
    "settings-write-errors",
    "output-settings-unavailable",
  ],
  base: "output-settings",
};

/** Screen of the Advanced settings page; null outside it (a row then decides alone). */
export const OutputScreenContext =
  createContext<ScreenScope<OutputScreen> | null>(null);

/**
 * Test-id prefix of the screen shown, for a row that has no state screen of its own.
 * @returns the page's screen, or the base screen outside it
 */
export const useOutputScreen = (): OutputScreen =>
  useContext(OutputScreenContext)?.screen ?? "output-settings";

/**
 * Reports one row's state screen to the page and returns the screen shown.
 * Side effects: reports `own` for `source` while mounted, and null when unmounted.
 * @param source a stable name of the row
 * @param own the state screen the row shows, or null
 * @returns the test-id prefix of the screen shown
 */
export const useReportedOutputScreen = (
  source: string,
  own: OutputScreen | null,
): OutputScreen =>
  useReportedScreen(OutputScreenContext, OUTPUT_SCREENS, source, own);
