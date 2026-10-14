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

/** The field: the text being typed, or null to show the stored value. */
export interface HistoryLimitField {
  draft: string | null;
}

/**
 * Typing changes the draft; Enter commits it, and so does leaving the field
 * for another place of the window (`withinWindow`). A blur caused by the
 * window losing focus (another application, the window closed to the tray)
 * commits nothing: the draft stays for when the user comes back.
 */
export type HistoryLimitEvent =
  | { type: "typed"; text: string }
  | { type: "committed" }
  | { type: "left"; withinWindow: boolean };

/** The field after an event, and the value to save, if any. */
export interface HistoryLimitStep {
  field: HistoryLimitField;
  save: number | null;
}

export const SHOWN_STORED: HistoryLimitField = { draft: null };

/**
 * Next state of the field. Business rule: nothing is saved while typing
 * (every history_limit write runs the history cleanup, so saving the "1" of
 * "12" would delete entries); a commit saves a valid value that differs from
 * the stored one, keeps a refused text in the field with its error and never
 * sends it.
 * @param field Current field.
 * @param event What the user did.
 * @param stored Stored history limit.
 */
export const historyLimitStep = (
  field: HistoryLimitField,
  event: HistoryLimitEvent,
  stored: number,
): HistoryLimitStep => {
  if (event.type === "typed") {
    return { field: { draft: event.text }, save: null };
  }
  // Business rule: a partial value ("1" on the way to "12") must never reach
  // the history cleanup because the window lost focus.
  if (event.type === "left" && !event.withinWindow)
    return { field, save: null };
  const parsed = parseHistoryLimit(field.draft ?? String(stored));
  if (!parsed.isValid) return { field, save: null };
  const save = parsed.value === stored ? null : parsed.value;
  return { field: SHOWN_STORED, save };
};

/**
 * Whether a blur moved the focus elsewhere in the window: the document still
 * has the focus, or another element received it. False when the window
 * itself lost the focus.
 * @param documentHasFocus `document.hasFocus()` when the blur fires.
 * @param relatedTarget The blur's `relatedTarget` (element receiving focus).
 */
export const isBlurWithinWindow = (
  documentHasFocus: boolean,
  relatedTarget: unknown,
): boolean => documentHasFocus || relatedTarget != null;

/**
 * Whether a key commits the field (Enter, like leaving it).
 * @param key `KeyboardEvent.key`.
 */
export const isCommitKey = (key: string): boolean => key === "Enter";
