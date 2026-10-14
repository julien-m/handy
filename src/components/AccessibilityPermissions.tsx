// macOS accessibility permission alert at the top of the settings: shown
// while the permission is off. With smart paste selected (007) it says that
// smart paste can neither insert nor paste. Its button asks macOS for the
// permission, then checks again on the next click.
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type } from "@tauri-apps/plugin-os";
import {
  checkAccessibilityPermission,
  requestAccessibilityPermission,
} from "tauri-plugin-macos-permissions-api";
import { useSettings } from "../hooks/useSettings";
import { Alert } from "./ui/Alert";
import { Button } from "./ui/Button";

/** Whether the next click requests the permission or checks it again. */
type PermissionStep = "request" | "verify";

const TEST_ID = "output-settings-paste-methods-permission";

interface AccessibilityPermission {
  /** Null until the first check answered. */
  hasPermission: boolean | null;
  openSettings: () => Promise<void>;
}

/**
 * Reads the permission once on macOS; the returned action requests it the
 * first time (system prompt), then re-checks it.
 * @param isMacOS Whether the permission applies on this platform.
 */
const useAccessibilityPermission = (
  isMacOS: boolean,
): AccessibilityPermission => {
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [step, setStep] = useState<PermissionStep>("request");
  useEffect(() => {
    if (!isMacOS) return;
    checkAccessibilityPermission()
      .then(setHasPermission)
      .catch((error: unknown) => {
        console.error("Error checking accessibility permission:", error);
      });
  }, [isMacOS]);
  const openSettings = async (): Promise<void> => {
    if (step === "verify") {
      setHasPermission(await checkAccessibilityPermission());
      return;
    }
    try {
      await requestAccessibilityPermission();
    } catch (error) {
      console.error("Error requesting permissions:", error);
    }
    setStep("verify");
  };
  return { hasPermission, openSettings };
};

const AccessibilityPermissions: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const isMacOS = type() === "macos";
  const { hasPermission, openSettings } = useAccessibilityPermission(isMacOS);
  if (!isMacOS || hasPermission !== false) return null;
  const isSmartPaste = getSetting("paste_method") === "smart_paste";
  return (
    <Alert
      variant="warning"
      role="status"
      className="w-full"
      testId={TEST_ID}
      title={t("accessibility.offTitle")}
      action={
        <Button
          variant="warning"
          data-testid={`${TEST_ID}-open`}
          onClick={() => void openSettings()}
        >
          {t("accessibility.openSettings")}
        </Button>
      }
    >
      {t(
        isSmartPaste
          ? "accessibility.smartPasteOff"
          : "accessibility.permissionsDescription",
      )}
    </Alert>
  );
};

export default AccessibilityPermissions;
