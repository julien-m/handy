// Notices of the recording indicator: AI failure (005), Screen Sharing hand-over (007), recording cut short by a lost microphone (001), failed insertion without a recovery bubble (N17) and « Send to the app in front » without a pin (014). Static cards without controls, announced to assistive technology.
import { AlertTriangle, Info } from "lucide-react";
import type React from "react";
import { useTranslation } from "react-i18next";
import type { OverlayNotice } from "./notice-model";

/** The live notices, oldest first. */
export const OverlayNotices: React.FC<{ notices: OverlayNotice[] }> = ({
  notices,
}) => {
  const { t } = useTranslation();
  return (
    <>
      {notices.map((notice) => (
        <div
          key={notice.kind}
          className={`snotice ${notice.kind}`}
          role="alert"
          data-testid={notice.testId}
        >
          {notice.kind === "hand_over" || notice.kind === "no_pin" ? (
            <Info className="snotice-icon" aria-hidden="true" />
          ) : (
            <AlertTriangle className="snotice-icon" aria-hidden="true" />
          )}
          <p className="snotice-text">
            {t(notice.key, {
              app: notice.appName ?? t("overlay.destination.unknownApp"),
            })}
          </p>
        </div>
      ))}
    </>
  );
};
