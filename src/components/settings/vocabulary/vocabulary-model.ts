/**
 * Presentation rules of the vocabulary editor (feature 003): which mockup screen the editor shows
 * (it prefixes the test IDs), the term input and chip states, and the commit error wording. Term
 * validity and duplicate keys always come from the backend; nothing here validates a term.
 */
import type {
  Occurrence,
  SettingsCommandError,
  VocabularyStatus,
  VocabularyStatusReport,
} from "@/bindings";
import { settingsErrorKey } from "../../../stores/settings-errors";

/** Mirrors `MAX_TERMS` (`recognition_vocabulary/validation.rs`); only used for the count label. */
export const VOCABULARY_MAX_TERMS = 500;

/** Test ID prefixes, one per approved mockup screen of the contract. */
export const VOCABULARY_SCREENS = {
  default: "vocabulary-settings",
  unsupported: "vocabulary-settings-unsupported",
  empty: "vocabulary-settings-empty",
  paste: "vocabulary-settings-paste",
} as const;

export type VocabularyScreen =
  (typeof VOCABULARY_SCREENS)[keyof typeof VOCABULARY_SCREENS];

/**
 * Mockup screen the list view matches, which also prefixes its test IDs.
 * @param termCount stored occurrences
 * @param status current status, or null while it loads or failed
 * @returns the empty screen for an empty list, the unsupported one for a model without
 * recognition context, the default screen otherwise
 */
export const listScreen = (
  termCount: number,
  status: VocabularyStatus | null,
): VocabularyScreen => {
  if (termCount === 0) return VOCABULARY_SCREENS.empty;
  if (status === "not_supported") return VOCABULARY_SCREENS.unsupported;
  return VOCABULARY_SCREENS.default;
};

/** Behavioral AC visual states of the term input. */
export type TermInputState = "empty" | "invalid" | "valid";

/**
 * Visual state of the term input.
 * @param draft current input text
 * @param refused true while the backend refusal of the last Add is shown
 * @returns "empty" for blank input, "invalid" while a refusal is shown, "valid" otherwise
 */
export const termInputState = (
  draft: string,
  refused: boolean,
): TermInputState => {
  if (draft.trim() === "") return "empty";
  return refused ? "invalid" : "valid";
};

/**
 * Whether Add may dispatch: blank input is the only case decided here, the backend judges the
 * rest.
 * @param draft current input text
 * @param busy true while a vocabulary commit is pending
 * @returns true when Add is enabled
 */
export const canAddTerm = (draft: string, busy: boolean): boolean =>
  !busy && draft.trim() !== "";

/**
 * Stored positions the next request leaves out because the model limit is reached.
 * @param report status, or null when unknown
 * @returns indices of over-budget occurrences (shown dimmed)
 */
export const overBudgetIndices = (
  report: VocabularyStatusReport | null,
): Set<number> =>
  new Set(
    (report?.terms_not_sent ?? [])
      .filter((term) => term.reason === "over_budget")
      .map((term) => term.index),
  );

/**
 * Translation key explaining a flagged chip.
 * @param occurrence stored occurrence
 * @param overBudget true when the model limit leaves it out
 * @returns a key under `settings.vocabulary.flag`, or null for a term that is sent
 */
export const chipFlagKey = (
  occurrence: Occurrence,
  overBudget: boolean,
): string | null => {
  if (occurrence.issue) return `settings.vocabulary.flag.${occurrence.issue}`;
  return overBudget ? "settings.vocabulary.flag.over_budget" : null;
};

const ITEM_KEYS: Readonly<Record<string, string>> = {
  term_empty: "settings.vocabulary.errors.term_empty",
  term_too_long: "settings.vocabulary.errors.term_too_long",
  term_line_break: "settings.vocabulary.errors.term_line_break",
  term_forbidden_character:
    "settings.vocabulary.errors.term_forbidden_character",
  term_duplicate: "settings.vocabulary.errors.term_duplicate",
};

/**
 * Translation key for a refused vocabulary commit.
 * @param error typed backend refusal, or null when the command failed without one
 * @returns the term reason first, then the vocabulary-specific code, then the shared settings key
 */
export const commitErrorKey = (error: SettingsCommandError | null): string => {
  if (!error) return "errors.settings.generic";
  const item = error.items?.[0];
  if (item && ITEM_KEYS[item.code]) return ITEM_KEYS[item.code];
  if (error.outcome === "unknown") return "settings.vocabulary.errors.unknown";
  switch (error.code) {
    case "limit_exceeded":
      return "settings.vocabulary.errors.limit";
    case "revision_conflict":
      return "settings.vocabulary.errors.conflict";
    case "not_found":
      return "settings.vocabulary.errors.notFound";
    default:
      return settingsErrorKey(error);
  }
};

/**
 * Whether a refusal concerns the typed term (shown as an invalid input) rather than the list.
 * @param error typed backend refusal
 * @returns true for a refusal carrying a term reason
 */
export const isTermRefusal = (error: SettingsCommandError | null): boolean =>
  Boolean(error?.items?.some((item) => ITEM_KEYS[item.code]));

/**
 * Whether a failed commit offers Retry: a refused term, a full list or an occurrence already
 * removed would fail again unchanged, so the user edits instead.
 * @param error typed refusal, or null for a failure without one
 * @returns true when retrying the same operation can succeed
 */
export const canRetry = (error: SettingsCommandError | null): boolean =>
  !isTermRefusal(error) &&
  error?.code !== "limit_exceeded" &&
  error?.code !== "not_found";
