// Toast of the backend's `recording-error` event: a recording that could not start
// (permission, no device, other failure) or one cut short by a lost microphone.
import { platform } from "@tauri-apps/plugin-os";
import type { TFunction } from "i18next";
import { toast } from "sonner";
import type { RecordingErrorEvent } from "./types/events";

/**
 * Show the translated toast of one recording error.
 * @param event Payload emitted by the backend.
 * @param t Translation function of the caller.
 */
export const showRecordingErrorToast = (
  { error_type, detail }: RecordingErrorEvent,
  t: TFunction,
): void => {
  if (error_type === "microphone_permission_denied") {
    const platformKey = `errors.micPermissionDenied.${platform()}`;
    const description = t(platformKey, {
      defaultValue: t("errors.micPermissionDenied.generic"),
    });
    toast.error(t("errors.micPermissionDeniedTitle"), { description });
  } else if (error_type === "no_input_device") {
    toast.error(t("errors.noInputDeviceTitle"), {
      description: t("errors.noInputDevice"),
    });
  } else if (error_type === "input_lost") {
    // Feature 001: the audio captured before the loss is still transcribed.
    toast.warning(t("errors.inputLostTitle"), {
      description: t("errors.inputLost"),
    });
  } else {
    toast.error(
      t("errors.recordingFailed", { error: detail ?? "Unknown error" }),
    );
  }
};
