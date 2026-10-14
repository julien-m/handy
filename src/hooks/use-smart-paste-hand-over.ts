// Smart paste (feature 007) handed the transcript to Screen Sharing on the clipboard: show the
// translated message "Text placed on the clipboard for the remote screen" (AC-012) as a toast.
// The event carries identifiers only; its presentation on the recording indicator waits for the
// Penflow pass.
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

/** Backend event emitted by `smart_paste::app` after a Screen Sharing hand-over. */
export const SMART_PASTE_HAND_OVER_EVENT = "smart-paste-hand-over";

/** Shows the hand-over message while the calling component is mounted. */
export function useSmartPasteHandOver(): void {
  const { t } = useTranslation();
  useEffect(() => {
    const unlisten = listen(SMART_PASTE_HAND_OVER_EVENT, () => {
      toast.info(t("smartPaste.handOver"));
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [t]);
}
