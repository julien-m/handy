import React from "react";
import { useTranslation } from "react-i18next";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import type { ClipboardHandling } from "@/bindings";
import { useOutputScreen } from "./output-screen";

interface ClipboardHandlingProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ClipboardHandlingSetting: React.FC<ClipboardHandlingProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    // The row's test ids follow the output screen shown (006, 007).
    const screen = useOutputScreen();

    const clipboardHandlingOptions = [
      {
        value: "dont_modify",
        label: t("settings.advanced.clipboardHandling.options.dontModify"),
      },
      {
        value: "copy_to_clipboard",
        label: t("settings.advanced.clipboardHandling.options.copyToClipboard"),
      },
    ];

    const selectedHandling = (getSetting("clipboard_handling") ||
      "dont_modify") as ClipboardHandling;

    return (
      <SettingContainer
        title={t("settings.advanced.clipboardHandling.title")}
        description={t("settings.advanced.clipboardHandling.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
        testId={`${screen}-clipboard`}
      >
        <Dropdown
          options={clipboardHandlingOptions}
          testId={`${screen}-clipboard-select`}
          ariaLabel={t("settings.advanced.clipboardHandling.title")}
          selectedValue={selectedHandling}
          onSelect={(value) =>
            updateSetting("clipboard_handling", value as ClipboardHandling)
          }
          disabled={isUpdating("clipboard_handling")}
        />
      </SettingContainer>
    );
  });
