// Toast of the backend's `recording-error` event: a recording that could not start
// (permission, no device, other failure). A recording cut short by a lost microphone is a
// notice of the recording indicator (`src/overlay/use-overlay-notices.ts`), shown as a toast
// only when the overlay style is none (`showInputLostToast`).
import { platform } from "@tauri-apps/plugin-os";
import type { TFunction } from "i18next";
import { toast } from "sonner";
import { INPUT_LOST_ERROR, NOTICE_MIN_MS } from "@/overlay/notice-model";
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
  } else if (error_type === INPUT_LOST_ERROR) {
    // Feature 001: the indicator (or `showInputLostToast` with the overlay style none) reports
    // it, so the settings window never reports it a second time.
    return;
  } else {
    toast.error(
      t("errors.recordingFailed", { error: detail ?? "Unknown error" }),
    );
  }
};

/**
 * Toast of a recording cut short by a lost microphone (001), for the overlay style none only:
 * the audio captured before the loss is still transcribed.
 * @param t Translation function of the caller.
 */
export const showInputLostToast = (t: TFunction): void => {
  toast.warning(t("errors.inputLostTitle"), {
    description: t("errors.inputLost"),
    duration: NOTICE_MIN_MS,
  });
};
