// "Show the bubble when no text field is focused" (010), in the output settings: a switch
// written through the settings service like every scalar; absent when the build does not
// offer it (gate closed) and off macOS.
import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { showsNoFieldBubbleToggle } from "./recovery-bubble-toggle-state";

const KEY = "show_recovery_bubble_without_text_field";

interface NoFieldBubbleToggleProps {
  grouped?: boolean;
}

// @spec FR-005: The no-text-field setting in the output settings — .specs/features/010-recovery-bubble/spec.md#fr-005
/** The settings row, or nothing where the setting does not apply. */
export const NoFieldBubbleToggle: React.FC<NoFieldBubbleToggleProps> = ({
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting(KEY);
  if (!showsNoFieldBubbleToggle(value, useOsType())) return null;
  return (
    <ToggleSwitch
      checked={value === true}
      onChange={(enabled) => void updateSetting(KEY, enabled)}
      isUpdating={isUpdating(KEY)}
      label={t("settings.advanced.noFieldBubble.title")}
      description={t("settings.advanced.noFieldBubble.description")}
      descriptionMode="inline"
      grouped={grouped}
      testId="output-settings-no-field-bubble"
    />
  );
};
