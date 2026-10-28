/**
 * Paste preview rules of the vocabulary editor: preview rows in input order, the summary pill,
 * the Apply state, label and note. Line validity comes from the backend preview.
 */
import type {
  ImportPreview,
  TermIssue,
  VocabularyImportPreview,
} from "@/bindings";

/** Behavioral AC visual states of Apply. */
export type ApplyState = "enabled" | "disabled" | "loading";

/**
 * Visual state of Apply for the paste preview.
 * @param preview latest preview, or null before the first answer
 * @param fresh true when the preview answers the current text
 * @param applying true while the import commit is pending
 * @returns "loading" during the commit, "enabled" for an applicable preview adding at least one
 * term, "disabled" otherwise
 */
export const applyState = (
  preview: VocabularyImportPreview | null,
  fresh: boolean,
  applying: boolean,
): ApplyState => {
  if (applying) return "loading";
  const usable =
    preview !== null &&
    fresh &&
    preview.applicable &&
    preview.preview.added.length > 0;
  return usable ? "enabled" : "disabled";
};

/** One preview line in input order. */
export interface PreviewRow {
  /** One-based line number in the pasted text. */
  line: number;
  kind: "add" | "duplicate" | "error";
  /** Validated text; null for an invalid line. */
  term: string | null;
  issue: TermIssue | null;
  /** Earlier spelling a duplicate repeats, when it can be shown. */
  sameAs: string | null;
}

/**
 * Display-only key used to name the earlier spelling of a duplicate. The backend decides
 * duplicates with a full Unicode case fold; this simpler fold can miss a match (for example
 * "ß"), in which case the row says "already in the list" instead.
 */
const displayKey = (term: string): string =>
  term.trim().replace(/\s+/g, " ").toLowerCase();

const sameAsOf = (
  term: string,
  candidate: readonly string[],
): string | null => {
  const key = displayKey(term);
  return candidate.find((entry) => displayKey(entry) === key) ?? null;
};

/**
 * Preview lines of every kind, merged back into input order.
 * @param preview backend preview of the pasted text
 * @returns one row per pasted line
 */
export const previewRows = (preview: ImportPreview): PreviewRow[] => {
  const added = preview.added.map<PreviewRow>((line) => ({
    line: line.index + 1,
    kind: "add",
    term: line.term,
    issue: null,
    sameAs: null,
  }));
  const duplicates = preview.duplicates.map<PreviewRow>((line) => ({
    line: line.index + 1,
    kind: "duplicate",
    term: line.term,
    issue: null,
    sameAs: sameAsOf(line.term, preview.candidate),
  }));
  const errors = preview.issues.map<PreviewRow>((line) => ({
    line: line.index + 1,
    kind: "error",
    term: null,
    issue: line.issue,
    sameAs: null,
  }));
  return [...added, ...duplicates, ...errors].sort((a, b) => a.line - b.line);
};

/**
 * Translation key of a refused preview line.
 * @param issue backend reason
 * @returns a key under `settings.vocabulary.pasteBox.line`
 */
export const lineIssueKey = (issue: TermIssue): string =>
  `settings.vocabulary.pasteBox.line.${issue}`;

/** Counts shown in the preview summary pill. */
export interface PreviewSummary {
  added: number;
  duplicates: number;
  errors: number;
  /** Pill tone: warning when Apply is blocked, success when terms can be added. */
  tone: "success" | "secondary" | "warning";
}

/**
 * Summary of a paste preview.
 * @param preview backend preview
 * @returns counts and the pill tone
 */
export const previewSummary = (preview: ImportPreview): PreviewSummary => {
  const blocked = preview.issues.length > 0 || preview.over_capacity;
  const tone = blocked
    ? "warning"
    : preview.added.length > 0
      ? "success"
      : "secondary";
  return {
    added: preview.added.length,
    duplicates: preview.duplicates.length,
    errors: preview.issues.length,
    tone,
  };
};

/** Translation key and values of the note beside Apply. */
export interface ApplyNote {
  key: string;
  values: Record<string, string>;
}

const NOTE_ROOT = "settings.vocabulary.pasteBox.note";

/**
 * Note beside Apply explaining what Apply would do or why it cannot.
 * @param text current paste text
 * @param preview latest preview, or null
 * @param fresh true when the preview answers `text`
 * @returns the note key with its interpolation values
 */
export const applyNote = (
  text: string,
  preview: VocabularyImportPreview | null,
  fresh: boolean,
): ApplyNote => {
  if (text === "") return { key: `${NOTE_ROOT}.empty`, values: {} };
  if (!preview || !fresh) return { key: `${NOTE_ROOT}.checking`, values: {} };
  const { issues, over_capacity: overCapacity, added } = preview.preview;
  if (issues.length > 0) {
    const lines = issues.map((issue) => String(issue.index + 1)).join(", ");
    const key = issues.length === 1 ? "fixOne" : "fixMany";
    return { key: `${NOTE_ROOT}.${key}`, values: { lines } };
  }
  if (overCapacity) return { key: `${NOTE_ROOT}.overCapacity`, values: {} };
  if (added.length === 0)
    return { key: `${NOTE_ROOT}.nothingToAdd`, values: {} };
  return { key: `${NOTE_ROOT}.ready`, values: {} };
};

/** Why a Retry of a failed Apply sends nothing, and how the message is shown. */
export interface ApplyRetryBlock {
  note: ApplyNote;
  /** "info" when every line is already saved, "warning" when the lines must change first. */
  tone: "info" | "warning";
}

/**
 * Whether a Retry of a failed Apply may send the commit again. The failure rereads the preview, so
 * Retry follows the same rule as Apply: an applicable, current preview that adds at least one
 * term. Otherwise nothing is sent and the note explains why (lines to fix, the 5,000-term limit, a
 * preview still checking, or nothing left to add because the terms are already saved).
 * @param text current paste text
 * @param preview preview reread after the failure, or null
 * @param fresh true when the preview answers `text`
 * @returns null when Retry may send, else the message to show
 */
export const applyRetryBlock = (
  text: string,
  preview: VocabularyImportPreview | null,
  fresh: boolean,
): ApplyRetryBlock | null => {
  if (applyState(preview, fresh, false) === "enabled") return null;
  const note = applyNote(text, preview, fresh);
  const tone = note.key === `${NOTE_ROOT}.nothingToAdd` ? "info" : "warning";
  return { note, tone };
};

/**
 * Translation key of the Apply label.
 * @param added terms Apply would add
 * @param applying true while the commit is pending
 * @returns a key under `settings.vocabulary.pasteBox`
 */
export const applyLabelKey = (added: number, applying: boolean): string => {
  if (applying) return "settings.vocabulary.pasteBox.applying";
  if (added === 1) return "settings.vocabulary.pasteBox.applyOne";
  return added > 1
    ? "settings.vocabulary.pasteBox.applyMany"
    : "settings.vocabulary.pasteBox.applyZero";
};
