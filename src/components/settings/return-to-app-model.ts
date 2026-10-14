// What the "Return to the dictation's application" setting shows (006): the
// toggle is usable only where destination return is supported; elsewhere it
// is off and its description gives the reason (AC-007). With the
// `destination-return` rollout gate closed, the backend does not offer the
// setting, so the row is hidden and the settings stay as on `dev`.
import type { DestinationSupportState } from "../../hooks/use-destination-support";

const KEY = "settings.advanced.returnToApp";

/**
 * `loading` until the backend answered which support this instance has,
 * `hidden` when this build does not offer destination return.
 */
export type ReturnToAppState =
  | "loading"
  | "hidden"
  | "available"
  | "unavailable";

export interface ReturnToAppView {
  state: ReturnToAppState;
  /** Shown state of the toggle: off whenever the setting is unavailable. */
  checked: boolean;
  /** Translation key of the inline description (or of the reason). */
  descriptionKey: string;
}

/**
 * View of the setting for one support answer.
 * @param support Support of `destination_support`; undefined while loading,
 * "not_offered" when the build does not offer it, null when the question
 * failed.
 * @param stored Stored `return_to_dictation_app`, if loaded.
 * @returns The toggle state and the description to show.
 */
export const returnToAppView = (
  support: DestinationSupportState,
  stored: boolean | undefined,
): ReturnToAppView => {
  if (support === undefined || support === "not_offered") {
    return {
      state: support === undefined ? "loading" : "hidden",
      checked: stored ?? false,
      descriptionKey: `${KEY}.description`,
    };
  }
  if (support?.state === "supported") {
    return {
      state: "available",
      checked: stored ?? false,
      descriptionKey: `${KEY}.description`,
    };
  }
  const reason = support?.reason ?? "platform_unsupported";
  return {
    state: "unavailable",
    checked: false,
    descriptionKey: `${KEY}.${reason === "frontmost_unreadable" ? "unavailableUnreadable" : "unavailablePlatform"}`,
  };
};
