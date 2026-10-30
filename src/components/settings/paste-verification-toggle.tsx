// Experimental "Verify the paste" switch (007 AC-019), in the Experimental section: written
// through the settings service like every scalar; shown only while smart paste is selected.
import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { showsPasteVerificationToggle } from "./paste-verification-toggle-state";
import { useOutputScreen } from "./output-screen";

const KEY = "paste_verification";

interface PasteVerificationToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

// @spec FR-012: Experimental paste verification setting — .specs/features/007-smart-paste/spec.md#fr-012
/** The settings row, or nothing when smart paste is not the paste method. */
export const PasteVerificationToggle: React.FC<
  PasteVerificationToggleProps
> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  // The row's test id follows the settings screen shown (006).
  const screen = useOutputScreen();
  if (!showsPasteVerificationToggle(getSetting("paste_method"))) return null;
  return (
    <ToggleSwitch
      checked={getSetting(KEY) ?? false}
      onChange={(enabled) => void updateSetting(KEY, enabled)}
      isUpdating={isUpdating(KEY)}
      label={t("settings.advanced.pasteVerification.label")}
      description={t("settings.advanced.pasteVerification.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      testId={`${screen}-paste-verification`}
    />
  );
};
