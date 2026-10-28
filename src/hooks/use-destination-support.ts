/**
 * Whether this instance can return dictation text to its destination application (006,
 * AC-007), kept current by `destination-support-changed`: undefined until the backend answered,
 * "not_offered" when this build does not offer destination return (rollout gate closed, the
 * setting is hidden), null when the question itself failed.
 */
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands, type DestinationSupport } from "@/bindings";

/** Backend event of a runtime support change (`destination::app::DESTINATION_SUPPORT_CHANGED`). */
export const DESTINATION_SUPPORT_CHANGED_EVENT = "destination-support-changed";

/** What the settings know of destination return support. */
export type DestinationSupportState =
  | DestinationSupport
  | "not_offered"
  | null
  | undefined;

type SupportSetter = (
  update: (current: DestinationSupportState) => DestinationSupportState,
) => void;

/**
 * Support after a runtime change. Business rule: a build that does not offer destination
 * return keeps the setting hidden whatever the event says (the backend still reports its
 * platform support at startup), and a change before the first answer is ignored: that answer
 * reads the current support anyway, and showing the row early would flash it on such a build.
 * @param current What the settings know.
 * @param changed Support carried by the event.
 */
export const supportAfterChange = (
  current: DestinationSupportState,
  changed: DestinationSupport,
): DestinationSupportState =>
  current === undefined || current === "not_offered" ? current : changed;

/**
 * Asks the backend once; a `null` answer means the build does not offer the setting.
 * @param setSupport Receives the answer.
 * @returns Cleanup that ignores a late answer.
 */
export const readSupport = (setSupport: SupportSetter): (() => void) => {
  let mounted = true;
  commands
    .destinationSupport()
    .then((answer) => {
      if (mounted) setSupport(() => answer ?? "not_offered");
    })
    .catch((error: unknown) => {
      // Unanswered: show the setting as unavailable rather than guess support.
      console.warn("Destination support unavailable:", error);
      if (mounted) setSupport(() => null);
    });
  return () => {
    mounted = false;
  };
};

/**
 * Follows `destination-support-changed` (startup read, each dictation start).
 * @param setSupport Receives each change.
 * @returns Cleanup that stops listening.
 */
export const watchSupport = (setSupport: SupportSetter): (() => void) => {
  const unlisten = listen<DestinationSupport>(
    DESTINATION_SUPPORT_CHANGED_EVENT,
    (event) =>
      setSupport((current) => supportAfterChange(current, event.payload)),
  );
  return () => {
    void unlisten.then((stop) => stop());
  };
};

/**
 * Support of destination return for the settings window, read on mount and kept current.
 * @returns The support state, undefined while it is pending.
 */
export function useDestinationSupport(): DestinationSupportState {
  const [support, setSupport] = useState<DestinationSupportState>(undefined);
  useEffect(() => readSupport(setSupport), []);
  useEffect(() => watchSupport(setSupport), []);
  return support;
}
