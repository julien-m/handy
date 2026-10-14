import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import { useSettings } from "../../../hooks/useSettings";
import { useOsType } from "../../../hooks/useOsType";
import { useSmartPasteAvailability } from "../../../hooks/use-smart-paste-availability";
import { showsReliablePasteToggle } from "../paste-method-options";

interface ReliablePasteToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ReliablePasteToggle: React.FC<ReliablePasteToggleProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const osType = useOsType();
  const smartPasteAvailable = useSmartPasteAvailability();

  // The receipt-sequenced paste path is implemented for macOS and Windows; on macOS smart paste
  // replaces it once available (feature 007).
  if (!showsReliablePasteToggle(osType, smartPasteAvailable)) {
    return null;
  }

  return (
    <ToggleSwitch
      checked={getSetting("reliable_paste") ?? false}
      onChange={(enabled) => updateSetting("reliable_paste", enabled)}
      isUpdating={isUpdating("reliable_paste")}
      label={t("settings.debug.reliablePaste.title")}
      description={t("settings.debug.reliablePaste.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    />
  );
};
