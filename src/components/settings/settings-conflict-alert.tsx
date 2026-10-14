// "Settings changed elsewhere" (008): shown at the top of the settings when a
// write expected an older revision than the one another writer (the command
// line, another window) committed. Reload rereads the stored settings.
import React from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../../stores/settingsStore";
import { Alert } from "../ui/Alert";
import { Button } from "../ui/Button";

const TEST_ID = "settings-write-errors-conflict";

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
      {t("errors.settings.conflictBody")}
    </Alert>
  );
};
