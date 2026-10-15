// Root of the recovery window (010): the bubble of the current projection, sized and reported
// to the native side by `useRecoveryBubble`. Right-to-left languages mirror the layout.
import React from "react";
import { useTranslation } from "react-i18next";
import { getLanguageDirection } from "@/lib/utils/rtl";
import { RecoveryBubble } from "./recovery-bubble";
import { useRecoveryBubble } from "./use-recovery-bubble";

/** The recovery window content. */
export const RecoveryWindow: React.FC = () => {
  const bubble = useRecoveryBubble();
  const { i18n } = useTranslation();
  return (
    <div
      ref={bubble.ref}
      className="rb-stage"
      data-testid="recovery-bubble-screen"
      data-semantic-id="recovery-bubble"
      dir={getLanguageDirection(i18n.language)}
    >
      <RecoveryBubble
        view={bubble.view}
        explained={bubble.explained}
        onToggleExplanation={bubble.onToggleExplanation}
        onCopy={bubble.onCopy}
        onClose={bubble.onClose}
        onDragPointer={bubble.onDragPointer}
      />
    </div>
  );
};
