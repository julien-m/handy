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

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
export const PostProcessingToggle: React.FC<PostProcessingToggleProps> =
  React.memo(
    ({
      descriptionMode = "tooltip",
      grouped = false,
      label,
      description,
      testId,
    }) => {
      const { t } = useTranslation();
      const { getSetting, setPostProcessEnabled, isUpdating } = useSettings();

      const enabled = getSetting("post_process_enabled") || false;

      // Post-processing always turns on, as before; a shortcut the system
      // refused is reported here and flagged on its row, where it can be
      // changed.
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

      return (
        <ToggleSwitch
          checked={enabled}
          onChange={(next) => {
            toggle(next).catch((error) => {
              console.error("Failed to change post-processing:", error);
              if (error instanceof SettingsCommandFailure) {
                toast.error(t(settingsErrorKey(error.detail)));
              }
            });
          }}
          isUpdating={isUpdating("post_process_enabled")}
          label={label ?? t("settings.debug.postProcessingToggle.label")}
          description={
            description ?? t("settings.debug.postProcessingToggle.description")
          }
          descriptionMode={descriptionMode}
          grouped={grouped}
          testId={testId}
        />
      );
    },
  );
