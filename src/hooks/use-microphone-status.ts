// Microphone status of the audio settings (feature 001, AC-002): read when the view opens or
// becomes visible again, every 5 seconds while it stays visible (paused while the window is
// hidden in the tray, see status-polling.ts) and on demand (Refresh, Retry). Each read takes a
// generation ticket and only the newest read may update the view, so a late answer never
// overwrites a newer one and nothing is applied after the view is hidden or closes.
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type MicrophoneStatus } from "@/bindings";
import {
  EMPTY_STATUS,
  createReadFence,
  mergeStatusRead,
  type StatusView,
} from "@/components/settings/microphone-priority/availability";
import {
  documentVisibility,
  startStatusPolling,
  windowClock,
} from "@/components/settings/microphone-priority/status-polling";

/** Refresh period while the audio settings are open and visible, fixed by AC-002. */
export const STATUS_REFRESH_MS = 5000;

/** Status view plus its manual refresh. */
export interface MicrophoneStatusHandle extends StatusView {
  /** Read the status now (Refresh availability, Retry, after a Save). */
  refresh: () => Promise<void>;
}

/**
 * One status read; any failure is reported as null so the view shows unknown availability.
 * @returns the status, or null when the command failed
 */
const readStatus = async (): Promise<MicrophoneStatus | null> => {
  try {
    const result = await commands.getMicrophoneStatus();
    if (result.status === "ok") return result.data;
    console.warn("Microphone status read failed:", result.error);
  } catch (error) {
    console.warn("Microphone status read failed:", error);
  }
  return null;
};

/**
 * Keep the microphone status of the open settings view up to date.
 * Side effects: polls every 5 s while the calling component is mounted and the page is visible.
 * @returns the last status view and a manual refresh
 */
export const useMicrophoneStatus = (): MicrophoneStatusHandle => {
  const [view, setView] = useState<StatusView>(EMPTY_STATUS);
  const fence = useRef(createReadFence());

  const refresh = useCallback(async () => {
    const ticket = fence.current.begin();
    const read = await readStatus();
    // A newer read started (or the view closed) while this one was pending: drop it.
    if (!fence.current.isCurrent(ticket)) return;
    setView((previous) => mergeStatusRead(previous, read));
  }, []);

  useEffect(
    () =>
      // The returned stop function runs on unmount: timer cleared, pending read dropped.
      startStatusPolling({
        read: () => void refresh(),
        invalidate: () => fence.current.invalidate(),
        clock: windowClock,
        visibility: documentVisibility,
        periodMs: STATUS_REFRESH_MS,
      }),
    [refresh],
  );

  return { ...view, refresh };
};
