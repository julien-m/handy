// Standalone settings row of smart paragraphs (016): icon, label, information and switch.
import React from "react";
import { useTranslation } from "react-i18next";
import { type } from "@tauri-apps/plugin-os";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { smartParagraphsDescriptionKey } from "./smart-paragraphs-state";

const SETTING_KEY = "smart_paragraphs_enabled";
/** Pilcrow, the paragraph sign, shown before the label like the other text-formatting rows. */
const PARAGRAPH_ICON = "¶";

interface SmartParagraphsToggleProps {
  grouped?: boolean;
}

// @spec AC-010: Standalone row with icon, info, switch, FR-007: Translated row — .specs/features/016-smart-paragraphs/spec.md#ac-010
/**
 * The smart paragraphs switch, off unless the user turns it on; a change is saved through the
 * settings service and applies from the next dictation.
 * @param grouped whether the row sits inside a settings group
 */
export const SmartParagraphsToggle: React.FC<SmartParagraphsToggleProps> =
  React.memo(({ grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting(SETTING_KEY) ?? false;

    return (
      <div className="flex items-center" data-testid="smart-paragraphs-row">
        <span
          aria-hidden="true"
          className="ps-4 w-8 shrink-0 text-center text-sm text-mid-gray"
          data-testid="smart-paragraphs-icon"
        >
          {PARAGRAPH_ICON}
        </span>
        <div className="flex-1 min-w-0">
          <ToggleSwitch
            checked={enabled}
            onChange={(nextEnabled) => updateSetting(SETTING_KEY, nextEnabled)}
            isUpdating={isUpdating(SETTING_KEY)}
            label={t("settings.advanced.smartParagraphs.title")}
            description={t(smartParagraphsDescriptionKey(type()))}
            descriptionMode="tooltip"
            grouped={grouped}
            testId="smart-paragraphs"
          />
        </div>
      </div>
    );
  });
