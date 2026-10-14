// Validation of the history limit as typed (008 settings-write-errors): a
// whole number in the range the settings service accepts, checked before
// anything is sent so the field can show its error inline.

/** Mirrors the `history_limit` range in src-tauri/src/settings_service/descriptor_data.rs. */
export const HISTORY_LIMIT_RANGE = { min: 0, max: 1000 } as const;

/** A typed history limit: its value when accepted. */
export type HistoryLimitInput =
  | { isValid: true; value: number }
  | { isValid: false };

// Digits only: no sign, no decimal point, no exponent.
const WHOLE_NUMBER = /^\d+$/;

/**
 * Parses the text of the history limit field.
 * @param text Raw field text.
 * @returns The accepted value, or that the text is refused.
 */
export const parseHistoryLimit = (text: string): HistoryLimitInput => {
  const trimmed = text.trim();
  if (!WHOLE_NUMBER.test(trimmed)) return { isValid: false };
  const value = Number(trimmed);
  return value >= HISTORY_LIMIT_RANGE.min && value <= HISTORY_LIMIT_RANGE.max
    ? { isValid: true, value }
    : { isValid: false };
};
