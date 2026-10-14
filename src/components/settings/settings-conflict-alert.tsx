// "Settings changed elsewhere" (008): shown at the top of the settings when a
// write expected an older revision than the one another writer (the command
// line, another window) committed. Reload rereads the stored settings; if
// that read fails, the alert stays with the failure under its text.
import React from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../../stores/settingsStore";
import { Alert } from "../ui/Alert";
import { Button } from "../ui/Button";

const TEST_ID = "settings-write-errors-conflict";

/** Text of the alert and, after a failed Reload, why the window is not current yet. */
const ConflictAlertBody: React.FC = () => {
  const { t } = useTranslation();
  const reloadFailed = useSettingsStore((state) => state.conflictReloadFailed);
  return (
    <>
      <span className="block">{t("errors.settings.conflictBody")}</span>
      {reloadFailed && (
        <span
          className="block text-xs"
          data-testid={`${TEST_ID}-reload-failed`}
        >
          {t("errors.settings.reloadFailed")}
        </span>
      )}
    </>
  );
};

// @spec AC-004: Revision conflict refused and shown — .specs/features/008-settings-cli/spec.md#ac-004
/** The revision conflict alert, with Reload; nothing when there is no conflict. */
export const SettingsConflictAlert: React.FC = () => {
  const { t } = useTranslation();
  const hasConflict = useSettingsStore((state) => state.revisionConflict);
  const reload = useSettingsStore((state) => state.reloadAfterConflict);
  if (!hasConflict) return null;
  return (
    <Alert
      variant="warning"
      role="alert"
      className="w-full"
      testId={TEST_ID}
      title={t("errors.settings.conflictTitle")}
      action={
        <Button
          variant="warning"
          data-testid="settings-write-errors-reload"
          onClick={() => void reload()}
        >
          {t("errors.settings.reload")}
        </Button>
      }
    >
      <ConflictAlertBody />
    </Alert>
  );
};
