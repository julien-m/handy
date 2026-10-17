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
  automaticEnterOptions,
  automaticEnterTestIds,
  enterKeyOptions,
} from "./automatic-enter-state";
import { useReportedOutputScreen } from "./output-screen";

type AutomaticEnterTestIds = ReturnType<typeof automaticEnterTestIds>;

/** Props of the two selectors. */
interface SelectProps {
  ids: AutomaticEnterTestIds;
}

interface AutomaticEnterProps {
  grouped?: boolean;
}

/** The three-value selector, with the current value announced after a change. */
const ModeSelect: React.FC<
  SelectProps & { onOpenChange: (open: boolean) => void }
> = ({ ids, onOpenChange }) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const policy = getSetting("automatic_enter") ?? "never";
  const modes = automaticEnterOptions(policy).map((option) => ({
    value: option.value,
    label: t(option.labelKey),
    description: option.descriptionKey ? t(option.descriptionKey) : undefined,
    testId: option.testId,
  }));
  const current = modes.find((mode) => mode.value === policy)?.label ?? "";
  return (
    <>
      <Dropdown
        options={modes}
        selectedValue={policy}
        onSelect={(value) =>
          void updateSetting("automatic_enter", value as AutomaticEnterPolicy)
        }
        disabled={isUpdating("automatic_enter")}
        ariaLabel={t("settings.advanced.automaticEnter.title")}
        testId={ids.mode}
        menuTestId={ids.options}
        onOpenChange={onOpenChange}
      />
      <span className="sr-only" role="status" aria-live="polite">
        {t("settings.advanced.automaticEnter.announced", { value: current })}
      </span>
    </>
  );
};

/** The retained key selector (Enter, Ctrl+Enter, Cmd/Super+Enter). */
const KeySelect: React.FC<SelectProps> = ({ ids }) => {
  const { t } = useTranslation();
  const osType = useOsType();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const key = (getSetting("auto_submit_key") ?? "enter") as AutoSubmitKey;
  const keys = enterKeyOptions(osType).map((option) => ({
    value: option.value,
    label: t(option.labelKey),
  }));
  return (
    <Dropdown
      className="min-w-[140px]"
      options={keys}
      selectedValue={key}
      onSelect={(value) =>
        void updateSetting("auto_submit_key", value as AutoSubmitKey)
      }
      disabled={isUpdating("auto_submit_key")}
      ariaLabel={t("settings.advanced.automaticEnter.keyLabel")}
      testId={ids.key}
    />
  );
};

// @spec AC-004: Automatic Enter selector and retained key — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
/** The Automatic Enter selector, its key selector and the rule note. */
export const AutomaticEnter: React.FC<AutomaticEnterProps> = React.memo(
  ({ grouped = false }) => {
    const { t } = useTranslation();
    // The open option list is the output-settings-enter-options screen (009).
    const [isListOpen, setListOpen] = React.useState(false);
    const screen = useReportedOutputScreen(
      "automatic-enter",
      isListOpen ? "output-settings-enter-options" : null,
    );
    const ids = automaticEnterTestIds(screen);
    return (
      <div data-testid={ids.row}>
        <SettingContainer
          title={t("settings.advanced.automaticEnter.title")}
          description={t("settings.advanced.automaticEnter.description")}
          descriptionMode="inline"
          grouped={grouped}
          titleTestId={ids.title}
        >
          <div className="flex items-center gap-2">
            <ModeSelect ids={ids} onOpenChange={setListOpen} />
            <KeySelect ids={ids} />
          </div>
        </SettingContainer>
        <p className="px-4 pb-2 text-xs text-mid-gray" data-testid={ids.note}>
          {t("settings.advanced.automaticEnter.note")}
        </p>
      </div>
    );
  },
);
