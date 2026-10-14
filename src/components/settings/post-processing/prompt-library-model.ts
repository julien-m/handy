// Pure rules of the AI prompt library settings (005): the library limits and
// field validation mirrored from `src-tauri/src/prompts/library.rs`, and the
// translation keys of prompt issues and readiness items.
import type {
  FailureReason,
  ListedPrompt,
  PromptIssue,
  ReadinessIssue,
} from "@/bindings";

/** Mirrors `MAX_PROMPTS` in src-tauri/src/prompts/types.rs (AC-001). */
export const MAX_PROMPTS = 50;
/** Mirrors `MAX_NAME_CHARS` in src-tauri/src/prompts/types.rs (AC-001). */
export const MAX_NAME_CHARS = 60;
/** Mirrors `MAX_TEXT_CHARS` in src-tauri/src/prompts/types.rs (AC-001). */
export const MAX_TEXT_CHARS = 8000;
/** Dropdown value of the "None" default prompt. */
export const NO_PROMPT_VALUE = "";

/** Translation namespace of the prompt settings. */
export const PROMPT_KEYS = "settings.postProcessing.promptLibrary";
const KEY = PROMPT_KEYS;

/** Name field issues, in the order the backend checks them. */
export type NameIssue = Extract<
  PromptIssue,
  "name_empty" | "name_too_long" | "name_duplicate"
>;
/** Text field issues. */
export type TextIssue = Extract<PromptIssue, "text_empty" | "text_too_long">;
/** Visual state of the name field (Behavioral AC `has_validation`). */
export type NameFieldState = "empty" | "invalid" | "valid";

/**
 * Number of characters as the backend counts them (Unicode scalar values,
 * Rust `chars().count()`), not UTF-16 code units.
 * @param value Text to measure.
 */
export const charCount = (value: string): number => [...value].length;

/** Comparable form of a name: trimmed and lowercased, like `name_key`. */
const nameKey = (name: string): string => name.trim().toLowerCase();

/**
 * First issue of a draft name.
 * @param name Name as typed.
 * @param otherNames Names of every other prompt of the library.
 * @returns The issue, or null for a valid name.
 */
export const nameIssueOf = (
  name: string,
  otherNames: ReadonlyArray<string>,
): NameIssue | null => {
  const length = charCount(name.trim());
  if (length === 0) return "name_empty";
  if (length > MAX_NAME_CHARS) return "name_too_long";
  const key = nameKey(name);
  return otherNames.some((other) => nameKey(other) === key)
    ? "name_duplicate"
    : null;
};

/**
 * First issue of a draft text; the text is stored verbatim, so only its
 * trimmed emptiness and its full length count.
 * @param text Text as typed.
 */
export const textIssueOf = (text: string): TextIssue | null => {
  if (text.trim().length === 0) return "text_empty";
  return charCount(text) > MAX_TEXT_CHARS ? "text_too_long" : null;
};

/**
 * Visual state of the name field.
 * @param issue Current issue of the name.
 */
export const nameFieldState = (issue: NameIssue | null): NameFieldState => {
  if (issue === "name_empty") return "empty";
  return issue ? "invalid" : "valid";
};

/**
 * Names of the library's prompts other than `editingId`.
 * @param prompts Listed prompts.
 * @param editingId Prompt being edited, or null for a new prompt.
 */
export const otherPromptNames = (
  prompts: ReadonlyArray<ListedPrompt>,
  editingId: string | null,
): string[] =>
  prompts.filter((prompt) => prompt.id !== editingId).map((p) => p.name);

/** Translation key of a name or text issue shown under its field. */
export const fieldIssueKey = (issue: NameIssue | TextIssue): string =>
  `${KEY}.editor.fieldErrors.${issue}`;

/** Translation key of a prompt issue shown in the list ("Invalid: …"). */
export const promptIssueKey = (issue: PromptIssue): string =>
  `${KEY}.issues.${issue}`;

/**
 * Translation key of a readiness item or AI failure reason; readiness items
 * are failure reasons too, so both share one wording.
 */
export const reasonKey = (reason: FailureReason | ReadinessIssue): string =>
  `${KEY}.reasons.${reason}`;

/** What a list row flags next to its shortcuts, if anything. */
export interface RowStatus {
  kind: "invalid" | "not_ready";
  /** Translation key of the reason. */
  reasonKey: string;
}

// @spec AC-002: Invalid legacy prompts flagged — .specs/features/005-custom-ai-prompts/spec.md#ac-002
/**
 * Flag of a list row: an invalid prompt first (it cannot be used until
 * edited), else a provider that is not ready (AC-004).
 * @param prompt Listed prompt with its issues and readiness.
 */
export const rowStatusOf = (prompt: ListedPrompt): RowStatus | null => {
  if (prompt.issues.length > 0) {
    return { kind: "invalid", reasonKey: promptIssueKey(prompt.issues[0]) };
  }
  if (!prompt.readiness.ready && prompt.readiness.missing) {
    return {
      kind: "not_ready",
      reasonKey: reasonKey(prompt.readiness.missing),
    };
  }
  return null;
};

/** Focus target of the New prompt button. */
export const NEW_PROMPT_FOCUS = "new";

/**
 * Control that takes the focus back when the editor closes: the edit button
 * of the prompt that was edited while it is listed, else New prompt (a new
 * prompt cancelled, or a prompt deleted).
 * @param editedId Prompt the editor had open, or null for a new prompt.
 * @param prompts Listed prompts.
 * @returns The prompt id, or `NEW_PROMPT_FOCUS`.
 */
export const focusReturnOf = (
  editedId: string | null,
  prompts: ReadonlyArray<ListedPrompt>,
): string =>
  editedId !== null && prompts.some((prompt) => prompt.id === editedId)
    ? editedId
    : NEW_PROMPT_FOCUS;
