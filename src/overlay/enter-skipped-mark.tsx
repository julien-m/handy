// Mark of the recording indicator (014): « Insert without Enter » was pressed during this
// dictation, so its text is inserted without the automatic Enter. Informative only: it never
// takes focus and has no action (pressing the key again removes it).
import { CornerDownLeft } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import { useIndicatorScreen } from "./indicator-screen";
import "./enter-skipped-mark.css";

// @spec AC-004: Indicator mark with an accessible name — .specs/features/014-in-dictation-keys/spec.md#ac-004
/** Struck-through Enter glyph with its accessible name. */
export const EnterSkippedMark: React.FC = () => {
  const { t } = useTranslation();
  const screen = useIndicatorScreen();
  const label = t("overlay.dictationKeys.enterSkipped");
  return (
    <span
      className="senter-skipped"
      role="img"
      aria-label={label}
      title={label}
      data-testid={`${screen}-enter-skipped`}
    >
      <CornerDownLeft aria-hidden="true" />
    </span>
  );
};
