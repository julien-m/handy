import React from "react";
import { useTranslation } from "react-i18next";
import { type } from "@tauri-apps/plugin-os";
import { ChannelSelector } from "../ChannelSelector";
import { ShortcutList } from "../shortcut-list";
import { DICTATION_KEY_ACTIONS } from "../shortcut-list-model";
import { ShortcutScreenRoot } from "../shortcut-screen-root";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { ShortcutActivationSetting } from "../ShortcutActivation";
import { AudioFeedback } from "../AudioFeedback";
import { useSettings } from "../../../hooks/useSettings";
import { VolumeSlider } from "../VolumeSlider";
import { MuteWhileRecording } from "../MuteWhileRecording";
import { ModelSettingsCard } from "./ModelSettingsCard";
import {
  MicrophonePriorityRows,
  MicrophonePrioritySettings,
} from "../microphone-priority/microphone-priority-settings";

// Feature 001: the microphone priority editor wraps the Sound group (conflict banner above it,
// Unsaved changes bar below it); the other Sound rows are unchanged. Feature 004: the root's test
// ids follow the shortcut screen shown (`ShortcutScreenRoot`).
export const GeneralSettings: React.FC = () => {
  const { t } = useTranslation();
  const { audioFeedbackEnabled, getSetting } = useSettings();
  const hasDictationKeys = getSetting("automatic_enter") !== undefined;
  const isLinux = type() === "linux";
  return (
    <ShortcutScreenRoot
      general={
        <SettingsGroup title={t("settings.general.title")}>
          <ShortcutList actionId="transcribe" grouped={true} />
          <ShortcutActivationSetting descriptionMode="tooltip" grouped={true} />
          {/* 014: keys used during a dictation, in builds with Automatic Enter (destination return). */}
          {hasDictationKeys &&
            DICTATION_KEY_ACTIONS.map((actionId) => (
              <ShortcutList key={actionId} actionId={actionId} grouped />
            ))}
          {/* Cancel shortcut remains hidden on Linux because of dynamic shortcut instability. */}
          {!isLinux && <ShortcutList actionId="cancel" grouped single />}
        </SettingsGroup>
      }
    >
      <ModelSettingsCard />
      <MicrophonePrioritySettings>
        <SettingsGroup title={t("settings.sound.title")}>
          <MicrophonePriorityRows />
          <ChannelSelector descriptionMode="tooltip" grouped={true} />
          <MuteWhileRecording descriptionMode="tooltip" grouped={true} />
          <AudioFeedback descriptionMode="tooltip" grouped={true} />
          <OutputDeviceSelector
            descriptionMode="tooltip"
            grouped={true}
            disabled={!audioFeedbackEnabled}
          />
          <VolumeSlider disabled={!audioFeedbackEnabled} />
        </SettingsGroup>
      </MicrophonePrioritySettings>
    </ShortcutScreenRoot>
  );
};
