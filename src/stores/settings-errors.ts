/**
 * Typed settings command errors for the settings UI: a refused value is always reported, never
 * replaced silently. Translation keys are chosen from the typed outcome, reason and code — never
 * by parsing the English message.
 */
import type { SettingsCommandError } from "@/bindings";

/** Translation keys under `errors.settings`, one per user-facing situation. */
export const SETTINGS_ERROR_KEYS = {
  committedNotApplied: "errors.settings.committedNotApplied",
  gpuDeviceRequired: "errors.settings.gpuDeviceRequired",
  outOfRange: "errors.settings.outOfRange",
  validationFailed: "errors.settings.validationFailed",
  revisionConflict: "errors.settings.revisionConflict",
  unavailable: "errors.settings.unavailable",
  shortcutSessionBusy: "errors.settings.shortcutSessionBusy",
  persistenceFailed: "errors.settings.persistenceFailed",
  generic: "errors.settings.generic",
  readOnly: "errors.settings.readOnly",
} as const;

export type SettingsErrorKey =
  (typeof SETTINGS_ERROR_KEYS)[keyof typeof SETTINGS_ERROR_KEYS];

/**
 * Translation key for a refused settings command.
 * @param error typed error returned by the backend
 * @returns the most specific key: a stored-but-not-applied outcome first, then the validation
 * reason, then the error code, then the generic message
 */
export const settingsErrorKey = (
  error: SettingsCommandError,
): SettingsErrorKey => {
  if (error.outcome === "committed")
    return SETTINGS_ERROR_KEYS.committedNotApplied;
  if (error.reason === "gpu_device_required") {
    return SETTINGS_ERROR_KEYS.gpuDeviceRequired;
  }
  if (error.reason === "out_of_range") return SETTINGS_ERROR_KEYS.outOfRange;
  if (error.reason === "shortcut_session_busy") {
    return SETTINGS_ERROR_KEYS.shortcutSessionBusy;
  }
  switch (error.code) {
    case "validation_failed":
      return SETTINGS_ERROR_KEYS.validationFailed;
    case "revision_conflict":
      return SETTINGS_ERROR_KEYS.revisionConflict;
    case "unavailable":
      return SETTINGS_ERROR_KEYS.unavailable;
    case "persistence_failed":
      return SETTINGS_ERROR_KEYS.persistenceFailed;
    default:
      return SETTINGS_ERROR_KEYS.generic;
  }
};

/**
 * Narrow an unknown command error to the typed settings error.
 * @param value error payload of a command result
 * @returns true when `value` carries a settings error code and message
 */
export const isSettingsCommandError = (
  value: unknown,
): value is SettingsCommandError =>
  typeof value === "object" &&
  value !== null &&
  typeof (value as { code?: unknown }).code === "string" &&
  typeof (value as { message?: unknown }).message === "string";

/** Thrown by settings store actions when the backend refused a command. */
export class SettingsCommandFailure extends Error {
  readonly detail: SettingsCommandError;

  constructor(detail: SettingsCommandError) {
    super(detail.message);
    this.name = "SettingsCommandFailure";
    this.detail = detail;
  }
}

/**
 * Throw the typed refusal carried by a command result, if any.
 * @param result value returned by a settings command (results without a status pass through)
 * @throws SettingsCommandFailure when the result is a typed settings error
 */
export const throwIfSettingsError = (result: unknown): void => {
  if (
    typeof result === "object" &&
    result !== null &&
    (result as { status?: unknown }).status === "error"
  ) {
    const error = (result as { error?: unknown }).error;
    if (isSettingsCommandError(error)) throw new SettingsCommandFailure(error);
    throw new Error(String(error));
  }
};

/**
 * Revision ordering for settings refreshes: a response older than the newest revision already
 * applied is stale (an older refresh answered after a newer one) and must be discarded.
 * @param incoming revision of the response
 * @param current newest revision already applied, or null before the first load
 * @returns true when the response may replace the current settings
 */
export const isCurrentRevision = (
  incoming: number,
  current: number | null,
): boolean => current === null || incoming >= current;

/** How the settings UI recovers its optimistic value after a failed command. */
export type FailedCommandRecovery = "rollback" | "refresh";

/**
 * Recovery after a failed settings command. The store is authoritative: when the value was
 * committed (a live effect failed afterwards) or may have been (unknown outcome), or when another
 * writer committed while the command was in flight, the UI rereads instead of restoring a value
 * captured before; only a refusal with nothing newer applied rolls the optimistic value back.
 * @param error error thrown by the command
 * @param revisionAtStart UI revision when the command started
 * @param currentRevision UI revision now (a refresh may have applied a newer one)
 * @returns "refresh" or "rollback"
 */
export const failedCommandRecovery = (
  error: unknown,
  revisionAtStart: number | null,
  currentRevision: number | null,
): FailedCommandRecovery => {
  const outcome =
    error instanceof SettingsCommandFailure ? error.detail.outcome : null;
  if (outcome === "committed" || outcome === "unknown") return "refresh";
  return revisionAtStart === currentRevision ? "rollback" : "refresh";
};
