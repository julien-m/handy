/**
 * Opening the settings on one list from the Quick Add window (018): the backend shows the window
 * and emits `settings-reveal` with the list kind; the window goes to the Advanced section (asking
 * first when a section holds unsaved edits) and scrolls the list's anchor into view.
 */
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { QuickAddKind } from "@/bindings";

/** Event emitted by `quick_add::reveal_in_settings`. */
export const SETTINGS_REVEAL_EVENT = "settings-reveal";

/** Section holding both lists. */
export const REVEAL_SECTION = "advanced";

/**
 * Attribute marking where each list starts in the Advanced section.
 * @param kind list kind
 * @returns the attribute to spread on the list's wrapper
 */
export const revealAnchor = (kind: QuickAddKind) => ({
  "data-settings-anchor": kind,
});

// @spec AC-009: Settings opened on the list — .specs/features/018-quick-add-from-tray/spec.md#ac-009
/**
 * Follow `settings-reveal`: go to the Advanced section, then bring the list into view once that
 * section is shown.
 * @param current section shown
 * @param navigate guarded section navigation
 */
export const useSettingsReveal = (
  current: string,
  navigate: (section: typeof REVEAL_SECTION) => void,
): void => {
  const [target, setTarget] = useState<QuickAddKind | null>(null);
  const navigateRef = useRef(navigate);
  navigateRef.current = navigate;
  useEffect(() => {
    const stop = listen<QuickAddKind>(SETTINGS_REVEAL_EVENT, (event) => {
      setTarget(event.payload);
      navigateRef.current(REVEAL_SECTION);
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, []);
  useEffect(() => {
    if (target === null || current !== REVEAL_SECTION) return;
    // After the section change reset the scroll to the top (App layout effect).
    const frame = requestAnimationFrame(() => {
      document
        .querySelector(`[data-settings-anchor="${target}"]`)
        ?.scrollIntoView({ block: "start", behavior: "smooth" });
      setTarget(null);
    });
    return () => cancelAnimationFrame(frame);
  }, [target, current]);
};
