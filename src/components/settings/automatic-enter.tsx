// Automatic Enter row of the output settings (009, gate `destination-return` open): one
// three-value selector (Never, Always, Only in PIN mode) and the retained key selector, both
// keyboard-operable select-only comboboxes in reading order. It replaces the auto-submit control
// in an open build; a closed build keeps `AutoSubmit` (see `automatic-enter-state.ts`).
import React from "react";
import { useTranslation } from "react-i18next";
import type { AutoSubmitKey, AutomaticEnterPolicy } from "@/bindings";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import {
  AUTOMATIC_ENTER_TEST_IDS,
  automaticEnterOptions,
  enterKeyOptions,
} from "./automatic-enter-state";

interface AutomaticEnterProps {
  grouped?: boolean;
}

// @spec AC-004: Automatic Enter selector and retained key — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
/** The Automatic Enter selector, its key selector and the rule note. */
export const AutomaticEnter: React.FC<AutomaticEnterProps> = React.memo(
  ({ grouped = false }) => {
    const { t } = useTranslation();
    const osType = useOsType();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const policy = getSetting("automatic_enter") ?? "never";
    const key = (getSetting("auto_submit_key") ?? "enter") as AutoSubmitKey;
    const title = t("settings.advanced.automaticEnter.title");
    const modes = automaticEnterOptions(policy).map((option) => ({
      value: option.value,
      label: t(option.labelKey),
      description: option.descriptionKey ? t(option.descriptionKey) : undefined,
      testId: option.testId,
    }));
    const keys = enterKeyOptions(osType).map((option) => ({
      value: option.value,
      label: t(option.labelKey),
    }));
    const current = modes.find((mode) => mode.value === policy)?.label ?? "";
    return (
      <div data-testid={AUTOMATIC_ENTER_TEST_IDS.row}>
        <SettingContainer
          title={title}
          description={t("settings.advanced.automaticEnter.description")}
          descriptionMode="inline"
          grouped={grouped}
          titleTestId={AUTOMATIC_ENTER_TEST_IDS.title}
        >
          <div className="flex items-center gap-2">
            <Dropdown
              options={modes}
              selectedValue={policy}
              onSelect={(value) =>
                void updateSetting(
                  "automatic_enter",
                  value as AutomaticEnterPolicy,
                )
              }
              disabled={isUpdating("automatic_enter")}
              ariaLabel={title}
              testId={AUTOMATIC_ENTER_TEST_IDS.mode}
              menuTestId={AUTOMATIC_ENTER_TEST_IDS.options}
            />
            <Dropdown
              className="min-w-[140px]"
              options={keys}
              selectedValue={key}
              onSelect={(value) =>
                void updateSetting("auto_submit_key", value as AutoSubmitKey)
              }
              disabled={isUpdating("auto_submit_key")}
              ariaLabel={t("settings.advanced.automaticEnter.keyLabel")}
              testId={AUTOMATIC_ENTER_TEST_IDS.key}
            />
          </div>
          <span className="sr-only" role="status" aria-live="polite">
            {t("settings.advanced.automaticEnter.announced", {
              value: current,
            })}
          </span>
        </SettingContainer>
        <p
          className="px-4 pb-2 text-xs text-mid-gray"
          data-testid={AUTOMATIC_ENTER_TEST_IDS.note}
        >
          {t("settings.advanced.automaticEnter.note")}
        </p>
      </div>
    );
  },
);
