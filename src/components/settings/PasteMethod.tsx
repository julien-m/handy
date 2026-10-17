import React from "react";
import { useTranslation } from "react-i18next";
import { Dropdown, type DropdownOption } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import { useSmartPasteAvailability } from "../../hooks/use-smart-paste-availability";
import type { PasteMethod } from "@/bindings";
import { pasteMethodChoices } from "./paste-method-options";
import { useReportedOutputScreen } from "./output-screen";

interface PasteMethodProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const PasteMethodSetting: React.FC<PasteMethodProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const osType = useOsType();
    // The open option list is the output-settings-paste-methods screen (007).
    const [isListOpen, setListOpen] = React.useState(false);
    const screen = useReportedOutputScreen(
      "paste-method",
      isListOpen ? "output-settings-paste-methods" : null,
    );
    const optionTestId = `${screen}-paste-method`;

    const smartPasteAvailable = useSmartPasteAvailability();
    // No fallback value while the settings load: the selector shows the stored method only.
    const selectedMethod: PasteMethod | null =
      getSetting("paste_method") ?? null;
    const modifier = osType === "macos" ? "Cmd" : "Ctrl";
    const pasteMethodOptions: DropdownOption[] = pasteMethodChoices(
      osType,
      selectedMethod,
      smartPasteAvailable,
    ).map((choice) => ({
      value: choice.value,
      label: t(choice.labelKey, { modifier }),
      description: choice.descriptionKey && t(choice.descriptionKey),
      disabled: choice.disabled,
      testId: `${optionTestId}-${choice.slug}`,
    }));
    // The description explains smart paste only where it is offered.
    const isSmartPasteOffered = pasteMethodOptions.some(
      (option) => option.value === "smart_paste" && !option.disabled,
    );

    const externalScriptPath = getSetting("external_script_path") || "";

    return (
      <SettingContainer
        title={t("settings.advanced.pasteMethod.title")}
        description={t(
          isSmartPasteOffered
            ? "settings.advanced.pasteMethod.descriptionSmart"
            : "settings.advanced.pasteMethod.description",
        )}
        descriptionMode={descriptionMode}
        grouped={grouped}
        tooltipPosition="bottom"
        testId={`${screen}-paste-method`}
      >
        <div className="flex flex-col gap-2">
          <Dropdown
            ariaLabel={t("settings.advanced.pasteMethod.title")}
            testId={`${screen}-paste-method-select`}
            menuTestId={`${optionTestId}-options`}
            onOpenChange={setListOpen}
            options={pasteMethodOptions}
            selectedValue={selectedMethod}
            onSelect={(value) =>
              updateSetting("paste_method", value as PasteMethod)
            }
            disabled={isUpdating("paste_method")}
          />
          {selectedMethod === "external_script" && (
            <Input
              type="text"
              value={externalScriptPath}
              onChange={(e) =>
                updateSetting("external_script_path", e.target.value)
              }
              placeholder={t(
                "settings.advanced.pasteMethod.externalScriptPlaceholder",
              )}
              disabled={isUpdating("external_script_path")}
            />
          )}
        </div>
      </SettingContainer>
    );
  },
);
