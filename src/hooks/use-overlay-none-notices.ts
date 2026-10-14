// Notices of the recording indicator when the overlay style is "none" (no indicator window):
// the AI step failure (005 AC-005) and the Screen Sharing hand-over (007 AC-012) fall back to a
// toast of the settings window, kept at least as long as on the indicator. With any other
// style the indicator shows them (src/overlay/use-overlay-notices.ts) and no toast appears.
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { events, type OverlayStyle } from "@/bindings";

/** Backend event emitted by `smart_paste::app` after a Screen Sharing hand-over. */
export const SMART_PASTE_HAND_OVER_EVENT = "smart-paste-hand-over";
/** Minimum display time of a fallback notice, as on the indicator. */
const NOTICE_MIN_MS = 4000;

// @spec AC-005: Overlay none falls back to a toast — .specs/features/005-custom-ai-prompts/spec.md#ac-005
/** Shows the fallback toasts while the calling component is mounted. */
export function useOverlayNoneNotices(style: OverlayStyle | undefined): void {
  const { t } = useTranslation();
  const styleRef = useRef(style);
  styleRef.current = style;
  useEffect(() => {
    const indicatorHidden = () => styleRef.current === "none";
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
    ];
    return () => {
      stops.forEach((stop) => void stop.then((unlisten) => unlisten()));
    };
  }, [t]);
}
