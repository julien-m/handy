// Closed-lid microphone row of the audio settings (feature 001, AC-005): the existing clamshell
// override, saved on its own as before; it never changes the priority order. Shown on laptops
// only, like the existing selector of the debug settings.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import { Dropdown } from "../../ui/Dropdown";
import { SettingContainer } from "../../ui/SettingContainer";
import { addableNames } from "./availability";
import { PRIORITY_KEY_ROOT } from "./draft-state";

// The settings store shows a missing override (null) as this value and stores it back as null.
const NO_OVERRIDE = "Default";

/**
 * Whether this computer has a lid, read once.
 * @returns true on a laptop; false until known or when the check fails
 */
const useIsLaptop = (): boolean => {
  const [isLaptop, setIsLaptop] = useState(false);
  useEffect(() => {
    let active = true;
    const check = async () => {
      try {
        const result = await commands.isLaptop();
        if (active) setIsLaptop(result.status === "ok" && result.data);
      } catch (error) {
        console.error("Failed to check if device is laptop:", error);
      }
    };
    void check();
    return () => {
      active = false;
    };
  }, []);
  return isLaptop;
};

/** Override dropdown: None (no override) or one of the enumerated devices. */
const ClamshellSelect: React.FC<{ title: string }> = ({ title }) => {
  const { t } = useTranslation();
  const settings = useSettings();
  const options = [
    { value: NO_OVERRIDE, label: t(`${PRIORITY_KEY_ROOT}.clamshell.none`) },
    ...addableNames(settings.audioDevices).map((name) => ({
      value: name,
      label: name,
    })),
  ];
  return (
    <div data-testid="microphone-settings-clamshell-select">
      <Dropdown
        options={options}
        selectedValue={
          settings.getSetting("clamshell_microphone") ?? NO_OVERRIDE
        }
        onSelect={(value) =>
          void settings.updateSetting("clamshell_microphone", value)
        }
        onOpen={() => void settings.refreshAudioDevices()}
        disabled={
          settings.isUpdating("clamshell_microphone") || settings.isLoading
        }
        ariaLabel={title}
      />
    </div>
  );
};

/** Closed-lid microphone override, tried first when the lid is closed. */
export const ClamshellRow: React.FC = () => {
  const { t } = useTranslation();
  const isLaptop = useIsLaptop();
  if (!isLaptop) return null;
  const title = t(`${PRIORITY_KEY_ROOT}.clamshell.title`);
  return (
    <SettingContainer
      title={title}
      description={t(`${PRIORITY_KEY_ROOT}.clamshell.description`)}
      descriptionMode="inline"
      grouped={true}
      testId="microphone-settings-clamshell"
      titleTestId="microphone-settings-clamshell-title"
    >
      <ClamshellSelect title={title} />
    </SettingContainer>
  );
};
