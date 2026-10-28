// Transient notices of the recording indicator: the AI failure (005 `prompt-processing-failed`),
// the Screen Sharing hand-over (007 `smart-paste-hand-over`) and the recording cut short by a
// lost microphone (001 `recording-error` of type `input_lost`). Each stays at least four
// seconds, and the overlay window is held up meanwhile even when the dictation already ended
// (`hold_overlay_for_notice`). The destination reason of 006 (`insertion-outcome`, emitted only
// with the `destination-return` gate open) is presented by 010's recovery bubble instead, so a
// failure is never reported twice.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { commands, events } from "@/bindings";
import type { RecordingErrorEvent } from "@/lib/types/events";
import {
  addNotice,
  aiFailureNotice,
  handOverNotice,
  holdRemaining,
  INPUT_LOST_ERROR,
  inputLostNotice,
  RECORDING_ERROR_EVENT,
  liveNotices,
  nextExpiry,
  type OverlayNotice,
} from "./notice-model";

/** 007 hand-over event (`smart_paste::app::HAND_OVER_EVENT`). */
export const HAND_OVER_EVENT = "smart-paste-hand-over";

/** Adds the notice that `build` makes at the current time (none when it returns null). */
export type Push = (build: (now: number) => OverlayNotice | null) => void;

const holdWindow = (milliseconds: number) => {
  commands.holdOverlayForNotice(Math.ceil(milliseconds)).catch((error) => {
    console.warn("Could not hold the indicator for a notice", error);
  });
};

/**
 * Subscribe to the backend events that raise an indicator notice (the effect body of
 * `useOverlayNotices`, exported for its tests).
 * @param push adds a notice
 * @returns the pending unsubscriptions
 */
export const subscribeNoticeSources = (push: Push): Promise<() => void>[] => [
  events.promptProcessingFailed.listen((event) =>
    push((now) => aiFailureNotice(event.payload, now)),
  ),
  listen(HAND_OVER_EVENT, () => push(handOverNotice)),
  // Only the cut-short recording is an indicator notice; start errors stay toasts.
  listen<RecordingErrorEvent>(RECORDING_ERROR_EVENT, (event) => {
    if (event.payload.error_type === INPUT_LOST_ERROR) push(inputLostNotice);
  }),
];

// @spec FR-002: The bubble replaces the indicator's destination notice — .specs/features/010-recovery-bubble/spec.md#fr-002
const useNoticeSources = (push: Push) => {
  useEffect(() => {
    const stops = subscribeNoticeSources(push);
    return () => {
      stops.forEach((stop) => void stop.then((unlisten) => unlisten()));
    };
  }, [push]);
};

/** Notices currently shown on the indicator. */
export const useOverlayNotices = (): OverlayNotice[] => {
  const [notices, setNotices] = useState<OverlayNotice[]>([]);
  const noticesRef = useRef<OverlayNotice[]>([]);
  const pushRef = useRef<Push>((build) => {
    const now = Date.now();
    const notice = build(now);
    if (!notice) return;
    const next = addNotice(noticesRef.current, notice, now);
    noticesRef.current = next;
    setNotices(next);
    holdWindow(holdRemaining(next, now));
  });
  useNoticeSources(pushRef.current);

  useEffect(() => {
    const wait = nextExpiry(notices, Date.now());
    if (wait === null) return;
    const timer = setTimeout(() => {
      const remaining = liveNotices(noticesRef.current, Date.now());
      noticesRef.current = remaining;
      setNotices(remaining);
      if (remaining.length === 0) holdWindow(0);
    }, wait);
    return () => clearTimeout(timer);
  }, [notices]);
  return notices;
};
