// One switch row of the Text formatting section: icon, label, info button and an independent
// switch, written through the settings service; the next dictation uses the new value.
import React from "react";
import { useTranslation } from "react-i18next";
import { type } from "@tauri-apps/plugin-os";
import { useSettings } from "../../../hooks/useSettings";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import { useOutputScreen } from "../output-screen";
import type { TextFormattingOption } from "./text-formatting-options";

interface TextFormattingRowProps {
  option: TextFormattingOption;
}

/** The settings row of `option`, inside the section's group. */
export const TextFormattingRow: React.FC<TextFormattingRowProps> = ({
  option,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  // The row's test id follows the settings screen shown (006).
  const screen = useOutputScreen();
  const Icon = option.icon;
  return (
    <ToggleSwitch
      checked={getSetting(option.key) ?? false}
      onChange={(enabled) => void updateSetting(option.key, enabled)}
      isUpdating={isUpdating(option.key)}
      label={t(option.labelKey)}
      description={t(option.descriptionKey(type()))}
      descriptionMode="tooltip"
      grouped={true}
      testId={`${screen}-${option.testId}`}
      icon={<Icon className="w-4 h-4" />}
    />
  );
};
