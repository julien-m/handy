// What the "Return to the dictation's application" setting shows (006): the
// toggle is usable only where destination return is supported; elsewhere it
// is off and its description gives the reason (AC-007). With the
// `destination-return` rollout gate closed, the backend reports the
// platform as unsupported, so the setting shows as unavailable.
import type { DestinationSupport } from "@/bindings";

const KEY = "settings.advanced.returnToApp";

/** `loading` until the backend answered which support this instance has. */
export type ReturnToAppState = "loading" | "available" | "unavailable";

export interface ReturnToAppView {
  state: ReturnToAppState;
  /** Shown state of the toggle: off whenever the setting is unavailable. */
  checked: boolean;
  /** Translation key of the inline description (or of the reason). */
  descriptionKey: string;
}

/**
 * View of the setting for one support answer.
 * @param support Backend answer of `destination_support`; undefined while
 * loading, null when the instance has no destination manager.
 * @param stored Stored `return_to_dictation_app`, if loaded.
 * @returns The toggle state and the description to show.
 */
export const returnToAppView = (
  support: DestinationSupport | null | undefined,
  stored: boolean | undefined,
): ReturnToAppView => {
  if (support === undefined) {
    return {
      state: "loading",
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
