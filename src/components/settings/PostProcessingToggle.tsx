import React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import {
  SettingsCommandFailure,
  settingsErrorKey,
} from "../../stores/settings-errors";

interface PostProcessingToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
  /** Title and description replacing the Advanced page's wording. */
  label?: string;
  description?: string;
  testId?: string;
}

/**
 * Turns post-processing on or off. Post-processing always turns on, as
 * before; a shortcut the system refused is reported here and flagged on its
 * row, where it can be changed; a refused write is shown translated.
 */
const usePostProcessSwitch = () => {
  const { t } = useTranslation();
  const { getSetting, setPostProcessEnabled, isUpdating } = useSettings();
  const toggle = async (next: boolean) => {
    const refused = await setPostProcessEnabled(next);
    for (const refusal of refused) {
      toast.warning(
        t("settings.debug.postProcessingToggle.shortcutRefused", {
          shortcut: refusal.shortcut,
        }),
        { description: refusal.reason },
      );
    }
  };
  const onChange = (next: boolean) => {
    toggle(next).catch((error) => {
      console.error("Failed to change post-processing:", error);
      if (error instanceof SettingsCommandFailure) {
        toast.error(t(settingsErrorKey(error.detail)));
      }
    });
  };
  const enabled = getSetting("post_process_enabled") || false;
  return { enabled, onChange, isUpdating: isUpdating("post_process_enabled") };
};

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
export const PostProcessingToggle: React.FC<PostProcessingToggleProps> =
  React.memo((props) => {
    const { descriptionMode = "tooltip", grouped = false } = props;
    const { t } = useTranslation();
    const { enabled, onChange, isUpdating } = usePostProcessSwitch();
    return (
      <ToggleSwitch
        checked={enabled}
        onChange={onChange}
        isUpdating={isUpdating}
        label={props.label ?? t("settings.debug.postProcessingToggle.label")}
        description={
          props.description ??
          t("settings.debug.postProcessingToggle.description")
        }
        descriptionMode={descriptionMode}
        grouped={grouped}
        testId={props.testId}
      />
    );
  });
