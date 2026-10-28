// Notices of the recording indicator when the overlay style is "none" (no indicator window):
// the AI step failure (005 AC-005), the Screen Sharing hand-over (007 AC-012) and the recording
// cut short by a lost microphone (001 FR-004) fall back to a toast of the settings window, kept
// at least as long as on the indicator. With any other style the indicator shows them
// (src/overlay/use-overlay-notices.ts) and no toast appears.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { events, type OverlayStyle } from "@/bindings";
import { showInputLostToast } from "@/lib/recording-error-toast";
import type { RecordingErrorEvent } from "@/lib/types/events";
import {
  INPUT_LOST_ERROR,
  noticeFallsBackToToast,
  RECORDING_ERROR_EVENT,
} from "@/overlay/notice-model";

/** Backend event emitted by `smart_paste::app` after a Screen Sharing hand-over. */
export const SMART_PASTE_HAND_OVER_EVENT = "smart-paste-hand-over";
/** Minimum display time of a fallback notice, as on the indicator. */
const NOTICE_MIN_MS = 4000;

// @spec AC-005: Overlay none falls back to a toast — .specs/features/005-custom-ai-prompts/spec.md#ac-005
// @spec FR-004: Cut-short toast only without indicator — .specs/features/001-microphone-priority/spec.md#fr-004
/** Shows the fallback toasts while the calling component is mounted. */
export function useOverlayNoneNotices(style: OverlayStyle | undefined): void {
  const { t } = useTranslation();
  const styleRef = useRef(style);
  styleRef.current = style;
  useEffect(() => {
    const indicatorHidden = () => noticeFallsBackToToast(styleRef.current);
    const stops = [
      events.promptProcessingFailed.listen((event) => {
        if (!indicatorHidden()) return;
        const duration = Math.max(event.payload.notice_ms, NOTICE_MIN_MS);
        toast.warning(t("overlay.fallbackNotice"), { duration });
      }),
      listen(SMART_PASTE_HAND_OVER_EVENT, () => {
        if (!indicatorHidden()) return;
        toast.info(t("smartPaste.handOver"), { duration: NOTICE_MIN_MS });
      }),
      listen<RecordingErrorEvent>(RECORDING_ERROR_EVENT, (event) => {
        if (event.payload.error_type !== INPUT_LOST_ERROR) return;
        if (indicatorHidden()) showInputLostToast(t);
      }),
    ];
    return () => {
      stops.forEach((stop) => void stop.then((unlisten) => unlisten()));
    };
  }, [t]);
}
