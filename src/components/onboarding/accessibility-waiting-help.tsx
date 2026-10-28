// Way out of a stalled accessibility request on the "Permissions Required" screen (native
// session finding N10): macOS shows its request once, and a locally re-signed build can stay
// listed (checked) for its old signature, so the screen may wait forever. After a few seconds
// of waiting, it offers to open Privacy & Security › Accessibility, to try again, and says how
// to clear a stale entry.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui/Button";

/** Privacy & Security › Accessibility pane of System Settings (allowed in the opener scope). */
export const ACCESSIBILITY_SETTINGS_URL =
  "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/**
 * How long "Waiting…" may last before the way out shows. The system request answers within a
 * second or two when it appears; five seconds leaves time to click it without hiding the help
 * from a user who saw no request.
 */
export const STALE_WAIT_MS = 5000;

const KEY = "onboarding.permissions.accessibility";

/**
 * Whether a wait started at `since` has lasted long enough to offer the way out.
 * @param since when the wait started (ms), or null when not waiting
 * @param now current time (ms)
 * @returns true once STALE_WAIT_MS has passed
 */
export const isWaitStale = (since: number | null, now: number): boolean =>
  since !== null && now - since >= STALE_WAIT_MS;

/**
 * True once the current wait has lasted STALE_WAIT_MS; false again when a new wait starts.
 * Side effects: one timer per wait.
 * @param since when the wait started (ms), or null when not waiting
 */
export const useStaleWait = (since: number | null): boolean => {
  const [stale, setStale] = useState(false);
  useEffect(() => {
    setStale(isWaitStale(since, Date.now()));
    if (since === null) return;
    const remaining = Math.max(0, since + STALE_WAIT_MS - Date.now());
    const timer = setTimeout(() => setStale(true), remaining);
    return () => clearTimeout(timer);
  }, [since]);
  return stale;
};

interface AccessibilityWaitingHelpProps {
  onOpenSettings: () => void;
  onRetry: () => void;
}

/** Open System Settings and Try again, with the hint for an entry that is already checked. */
export const AccessibilityWaitingHelp: React.FC<
  AccessibilityWaitingHelpProps
> = ({ onOpenSettings, onRetry }) => {
  const { t } = useTranslation();
  return (
    <div
      className="flex flex-col gap-2 mt-3"
      data-testid="onboarding-accessibility-help"
    >
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          size="sm"
          data-testid="onboarding-accessibility-open-settings"
          onClick={onOpenSettings}
        >
          {t(`${KEY}.openSettings`)}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          data-testid="onboarding-accessibility-retry"
          onClick={onRetry}
        >
          {t(`${KEY}.retry`)}
        </Button>
      </div>
      <p className="text-xs text-text/60">{t(`${KEY}.alreadyCheckedHint`)}</p>
    </div>
  );
};
