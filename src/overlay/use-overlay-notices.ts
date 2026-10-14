// Transient notices of the recording indicator: the AI failure (005 `prompt-processing-failed`),
// the destination reason (006 `insertion-outcome`) and the Screen Sharing hand-over (007
// `smart-paste-hand-over`). Each stays at least four seconds, and the overlay window is held up
// meanwhile even when the dictation already ended (`hold_overlay_for_notice`).
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState, type MutableRefObject } from "react";
import { commands, events, type DestinationSnapshot } from "@/bindings";
import { selectionName } from "./destination-model";
import {
  addNotice,
  aiFailureNotice,
  destinationNotice,
  handOverNotice,
  holdRemaining,
  liveNotices,
  nextExpiry,
  parseOutcome,
  type OverlayNotice,
} from "./notice-model";

/** 006 terminal outcome event (`destination::delivery::INSERTION_OUTCOME`). */
export const INSERTION_OUTCOME_EVENT = "insertion-outcome";
/** 007 hand-over event (`smart_paste::app::HAND_OVER_EVENT`). */
export const HAND_OVER_EVENT = "smart-paste-hand-over";

type Push = (build: (now: number) => OverlayNotice | null) => void;

const holdWindow = (milliseconds: number) => {
  commands.holdOverlayForNotice(Math.ceil(milliseconds)).catch((error) => {
    console.warn("Could not hold the indicator for a notice", error);
  });
};

const useNoticeSources = (
  push: Push,
  snapshotRef: MutableRefObject<DestinationSnapshot | null>,
) => {
  useEffect(() => {
    const stops = [
      events.promptProcessingFailed.listen((event) =>
        push((now) => aiFailureNotice(event.payload, now)),
      ),
      listen<unknown>(INSERTION_OUTCOME_EVENT, (event) => {
        const facts = parseOutcome(event.payload);
        if (!facts) return;
        const name = selectionName(snapshotRef.current, facts.dictationId);
        push((now) => destinationNotice(facts, name, now));
      }),
      listen(HAND_OVER_EVENT, () => push(handOverNotice)),
    ];
    return () => {
      stops.forEach((stop) => void stop.then((unlisten) => unlisten()));
    };
  }, [push, snapshotRef]);
};

/**
 * Notices currently shown on the indicator. `snapshotRef` holds the newest destination
 * session, used to name the application of a destination notice.
 */
export const useOverlayNotices = (
  snapshotRef: MutableRefObject<DestinationSnapshot | null>,
): OverlayNotice[] => {
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
  useNoticeSources(pushRef.current, snapshotRef);

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
