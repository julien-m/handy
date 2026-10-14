// Alerts under a shortcut list, inside its settings card: a stored shortcut
// that conflicts since before the update (with "Change"), and a shortcut the
// system refused to register (with "Try again").
import React from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "../ui/Alert";
import { Button } from "../ui/Button";
import { useOsType } from "../../hooks/useOsType";
import {
  type LegacyWarning,
  type MessageNames,
  type ShortcutTestIds,
  type SystemRefusal,
  systemRefusalTitle,
} from "./shortcut-list-feedback";

const LIST_KEY = "settings.general.shortcut.list";
// An alert that ends the settings card follows the card's rounded corners.
const LAST_IN_CARD = "last:rounded-b-lg";

interface LegacyConflictAlertProps {
  warning: LegacyWarning;
  ids: ShortcutTestIds;
  busy: boolean;
  onChange: (shortcut: string) => void;
}

// @spec AC-002: Inherited conflicts flagged on every involved row — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/** An inherited conflict: it still works as before; "Change" records it again. */
export const LegacyConflictAlert: React.FC<LegacyConflictAlertProps> = ({
  warning,
  ids,
  busy,
  onChange,
}) => {
  const { t } = useTranslation();
  return (
    <Alert
      variant="warning"
      contained
      className={LAST_IN_CARD}
      role="status"
      testId={ids.legacy}
      title={t(warning.title.key, warning.title.params)}
      action={
        <Button
          variant="warning"
          data-testid={ids.change}
          disabled={busy}
          onClick={() => onChange(warning.shortcut)}
        >
          {t(`${LIST_KEY}.changeButton`)}
        </Button>
      }
    >
      {t(`${LIST_KEY}.legacyBody`)}
    </Alert>
  );
};

interface SystemRefusalAlertProps {
  refusal: SystemRefusal;
  ids: ShortcutTestIds;
  names: MessageNames;
  busy: boolean;
  onRetry: () => void;
}

// @spec AC-004: OS refusal with the system's reason — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
/** The system refused the shortcut: nothing was saved; the edit can be retried. */
export const SystemRefusalAlert: React.FC<SystemRefusalAlertProps> = ({
  refusal,
  ids,
  names,
  busy,
  onRetry,
}) => {
  const { t } = useTranslation();
  const os = useOsType();
  const shortcut = refusal.shortcut && names.shortcutLabel(refusal.shortcut);
  const title = systemRefusalTitle(os, shortcut || null);
  return (
    <Alert
      variant="error"
      contained
      className={LAST_IN_CARD}
      role="alert"
      testId={ids.system}
      title={t(title.key, title.params)}
      action={
        <Button
          variant="secondary"
          data-testid={ids.retry}
          disabled={busy}
          onClick={onRetry}
        >
          {t(`${LIST_KEY}.retry`)}
        </Button>
      }
    >
      <span className="block">{t(`${LIST_KEY}.systemRefusedBody`)}</span>
      <span className="block text-xs">
        {t(`${LIST_KEY}.systemReason`, { error: refusal.reason })}
      </span>
    </Alert>
  );
};
