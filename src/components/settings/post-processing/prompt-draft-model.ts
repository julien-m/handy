// Pure rules of the prompt editor's draft (005 AC-001): what is edited, when
// Save is available, which field a backend refusal belongs to, and how a
// change made elsewhere (CLI, another window) is detected while editing.
import type { ListedPrompt, SettingsCommandError } from "@/bindings";
import {
  type NameIssue,
  type TextIssue,
  nameIssueOf,
  textIssueOf,
} from "./prompt-library-model";
import {
  type ProviderDraft,
  DEFAULT_PROVIDER_VALUE,
  providerDraftOf,
  sameProviderDraft,
} from "./prompt-provider-model";

/** The prompt being edited, or a new one. */
export type EditorTarget = { mode: "new" } | { mode: "edit"; id: string };

/** Values of the editor form. */
export interface PromptDraft {
  name: string;
  text: string;
  provider: ProviderDraft;
}

/** Validation of a draft, field by field. */
export interface DraftValidation {
  nameIssue: NameIssue | null;
  textIssue: TextIssue | null;
  valid: boolean;
}

/** Visual state of Save (Behavioral AC `is_submittable`). */
export type SaveState = "enabled" | "disabled" | "loading";

/** Field a backend refusal is shown under. */
export type RefusalField = "name" | "text" | "provider" | "form";

/** A saved prompt changed or vanished while it was being edited. */
export type ExternalChange = "none" | "changed" | "deleted";

/** Draft of a new prompt. */
export const EMPTY_DRAFT: PromptDraft = {
  name: "",
  text: "",
  provider: { providerId: DEFAULT_PROVIDER_VALUE, model: "" },
};

/**
 * Draft of a saved prompt.
 * @param prompt Listed prompt.
 */
export const draftOf = (prompt: ListedPrompt): PromptDraft => ({
  name: prompt.name,
  text: prompt.prompt,
  provider: providerDraftOf(prompt.provider_override),
});

/**
 * Whether the name or text differs from the saved values (the name is
 * stored trimmed, the text verbatim).
 * @param draft Values of the form.
 * @param saved Saved values.
 */
export const isContentDirty = (
  draft: PromptDraft,
  saved: PromptDraft,
): boolean =>
  draft.name.trim() !== saved.name.trim() || draft.text !== saved.text;

/**
 * Whether the draft differs from the saved values.
 * @param draft Values of the form.
 * @param saved Saved values.
 */
export const isDraftDirty = (draft: PromptDraft, saved: PromptDraft): boolean =>
  isContentDirty(draft, saved) ||
  !sameProviderDraft(draft.provider, saved.provider);

/**
 * Field validation of a draft against the rest of the library.
 * @param draft Values of the form.
 * @param otherNames Names of the other prompts.
 */
export const validateDraft = (
  draft: PromptDraft,
  otherNames: ReadonlyArray<string>,
): DraftValidation => {
  const nameIssue = nameIssueOf(draft.name, otherNames);
  const textIssue = textIssueOf(draft.text);
  return { nameIssue, textIssue, valid: !nameIssue && !textIssue };
};

/** Inputs of the Save state. */
export interface SaveStateInput {
  valid: boolean;
  dirty: boolean;
  saving: boolean;
  /** The edited prompt was deleted elsewhere. */
  deleted: boolean;
}

/**
 * State of Save: loading while committing; enabled for a valid draft with
 * something to save; disabled otherwise (nothing changed, invalid fields, or
 * the prompt no longer exists).
 * @param input Draft validity, changes and progress.
 */
export const saveStateOf = (input: SaveStateInput): SaveState => {
  if (input.saving) return "loading";
  return input.valid && input.dirty && !input.deleted ? "enabled" : "disabled";
};

const NAME_REASONS = new Set([
  "prompt_name_empty",
  "prompt_name_too_long",
  "prompt_name_duplicate",
]);
const TEXT_REASONS = new Set(["prompt_text_empty", "prompt_text_too_long"]);
const PROVIDER_REASONS = new Set([
  "prompt_override_incomplete",
  "prompt_unknown_provider",
  "unsupported_platform",
]);

/**
 * Field a refusal of a prompt command is shown under, from its typed
 * reason; every other refusal (busy, persistence, limit) concerns the form.
 * @param error Typed refusal.
 */
export const refusalFieldOf = (error: SettingsCommandError): RefusalField => {
  const reason = error.reason ?? "";
  if (NAME_REASONS.has(reason)) return "name";
  if (TEXT_REASONS.has(reason)) return "text";
  return PROVIDER_REASONS.has(reason) ? "provider" : "form";
};

/**
 * Whether the edited prompt changed elsewhere since it was opened or last
 * saved here: deleted, or saved with other values.
 * @param saved Values the editor last loaded or saved.
 * @param current The prompt in the latest library, if still there.
 */
export const externalChangeOf = (
  saved: PromptDraft,
  current: ListedPrompt | undefined,
): ExternalChange => {
  if (!current) return "deleted";
  return isDraftDirty(draftOf(current), saved) ? "changed" : "none";
};

/** Test-id prefixes of the two editor screens of the contract. */
export const EDITOR_TEST_IDS = {
  edit: "prompt-settings-editor",
  new: "prompt-settings-invalid",
} as const;

/**
 * Test-id prefix of the editor: the contract's `prompt-settings-editor`
 * screen edits a saved prompt and its `prompt-settings-invalid` screen is a
 * new prompt, so the prefix follows the mode, never the field state.
 * @param target Prompt being edited, or a new one.
 */
export const editorTestIdOf = (target: EditorTarget): string =>
  target.mode === "edit" ? EDITOR_TEST_IDS.edit : EDITOR_TEST_IDS.new;

/**
 * The draft as the backend stores it: the name trimmed, the text verbatim.
 * @param draft Values of the form.
 */
export const normalizedDraft = (draft: PromptDraft): PromptDraft => ({
  ...draft,
  name: draft.name.trim(),
  provider: {
    providerId: draft.provider.providerId,
    model: draft.provider.model.trim(),
  },
});

/** What one save committed, as reported by its commands. */
export interface CommittedParts {
  content: boolean;
  provider: boolean;
}

/**
 * Saved values after a save: each committed part takes the draft's value,
 * the others keep the previous saved value, so a partial save never shows
 * as fully saved.
 * @param draft Values of the form when saving.
 * @param saved Values saved before.
 * @param committed Parts the commands committed.
 */
export const savedAfterCommit = (
  draft: PromptDraft,
  saved: PromptDraft,
  committed: CommittedParts,
): PromptDraft => {
  const next = normalizedDraft(draft);
  return {
    name: committed.content ? next.name : saved.name,
    text: committed.content ? next.text : saved.text,
    provider: committed.provider ? next.provider : saved.provider,
  };
};

/** A backend refusal shown under its field. */
export interface EditorRefusal {
  field: RefusalField;
  /** Translation key of the refusal. */
  key: string;
}

/**
 * Error shown under a field: its own issue once it should be shown, else a
 * backend refusal of that field.
 * @param issueKey Translation key of the field's issue, if any.
 * @param showIssue Whether the issue is shown yet (field left, or empty name).
 * @param refusal Last backend refusal, if any.
 * @param field The field.
 * @returns The translation key to show, or null.
 */
export const fieldErrorOf = (
  issueKey: string | null,
  showIssue: boolean,
  refusal: EditorRefusal | null,
  field: RefusalField,
): string | null => {
  if (issueKey && showIssue) return issueKey;
  return refusal?.field === field ? refusal.key : null;
};

/**
 * Note of the editor's action bar: "Fix the highlighted fields" only while a
 * field is actually shown in error (a new prompt's untouched empty text is
 * not highlighted yet, so the neutral note stays); otherwise what saving does.
 * @param fieldErrors Errors shown under the name and text fields.
 * @returns The note's key under the editor's `actions` keys.
 */
export const actionNoteKeyOf = (fieldErrors: {
  nameError: string | null;
  textError: string | null;
}): "note" | "fixFields" =>
  fieldErrors.nameError !== null || fieldErrors.textError !== null
    ? "fixFields"
    : "note";
