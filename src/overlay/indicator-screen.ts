// Contract screen shown by the recording indicator (features 005, 006, 009). Every test id of the
// stack carries the prefix of the screen actually shown (`recording-indicator`, `-prompt-menu`,
// `-destination-menu`, `-pinned`, `-no-destination`, or the screen of a notice shown alone in
// `penflow/flow-ui-contract/contract.json`), so each test id is unique in the tree.
import { createContext, useContext } from "react";
import type { DictationPromptView } from "@/bindings";
import type { BadgeView } from "./destination-model";
import type { OverlayNotice } from "./notice-model";
import { pillVisible } from "./prompt-model";

/** Test-id prefix of a contract screen of the indicator. */
export type IndicatorScreen =
  | "recording-indicator"
  | "recording-indicator-prompt-menu"
  | "recording-indicator-destination-menu"
  | "recording-indicator-pinned"
  | "recording-indicator-no-destination"
  | "recording-indicator-ai-failed"
  | "recording-indicator-hand-over";

/** Indicator facts that choose the screen. */
export interface IndicatorScreenInput {
  showIndicator: boolean;
  recording: boolean;
  promptView: DictationPromptView | null;
  promptMenuOpen: boolean;
  /** The destination menu is open (loading, error or listed). */
  destinationMenuOpen: boolean;
  /** An application is pinned for the session (009). */
  pinned: boolean;
  badge: BadgeView;
  notices: readonly OverlayNotice[];
}

/** Screen of each notice shown alone (its test id is `<screen>-notice`). */
const NOTICE_SCREENS: Record<OverlayNotice["kind"], IndicatorScreen> = {
  ai_failed: "recording-indicator-ai-failed",
  hand_over: "recording-indicator-hand-over",
};

/**
 * Contract screen of the indicator. An open menu wins (only one is open at a time); a pin
 * shows the pinned screen, its menu open or not; a badge without application shows the
 * no-destination screen. Without the indicator, the newest notice names the screen.
 * @param input indicator facts
 * @returns the test-id prefix of the screen shown
 */
export const indicatorScreen = (
  input: IndicatorScreenInput,
): IndicatorScreen => {
  if (!input.showIndicator) {
    const notice = input.notices[input.notices.length - 1];
    return notice ? NOTICE_SCREENS[notice.kind] : "recording-indicator";
  }
  const promptShown = pillVisible(input.promptView, input.recording);
  if (promptShown && input.promptMenuOpen) {
    return "recording-indicator-prompt-menu";
  }
  // The destination menu and the pin belong to the badge: without it, neither is shown.
  if (input.badge.kind === "hidden") return "recording-indicator";
  if (input.pinned) return "recording-indicator-pinned";
  if (input.destinationMenuOpen) return "recording-indicator-destination-menu";
  if (input.badge.kind === "frontmost") {
    return "recording-indicator-no-destination";
  }
  return "recording-indicator";
};

/** Screen shown, for every part of the stack; the base screen outside it. */
export const IndicatorScreenContext = createContext<IndicatorScreen>(
  "recording-indicator",
);

/**
 * Test-id prefix of the screen shown.
 * @returns the screen of the closest indicator stack
 */
export const useIndicatorScreen = (): IndicatorScreen =>
  useContext(IndicatorScreenContext);
