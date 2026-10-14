// State of the prompt editor (005 AC-001, Behavioral AC `has_validation` and
// `is_submittable`): the draft against its saved values, live validation,
// the Save state, backend refusals per field, and a prompt changed or
// deleted elsewhere while it is open.
import {
  type Dispatch,
  type SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { ListedPrompt, PromptLibraryView } from "@/bindings";
import { settingsErrorKey } from "../../../stores/settings-errors";
import {
  createPrompt,
  updatePrompt,
  type CommitOutcome,
} from "./prompt-commands";
import {
  type DraftValidation,
  type EditorRefusal,
  type EditorTarget,
  type ExternalChange,
  type PromptDraft,
  type SaveState,
  EMPTY_DRAFT,
  draftOf,
  externalChangeOf,
  fieldErrorOf,
  isDraftDirty,
  refusalFieldOf,
  saveStateOf,
  savedAfterCommit,
  validateDraft,
} from "./prompt-draft-model";
import { fieldIssueKey, otherPromptNames } from "./prompt-library-model";
import type { ProviderDraft } from "./prompt-provider-model";

/** Everything the editor renders and the actions it takes. */
export interface PromptEditorModel {
  draft: PromptDraft;
  validation: DraftValidation;
  external: ExternalChange;
  saveState: SaveState;
  /** Translation keys of the errors under the name and text fields. */
  nameError: string | null;
  textError: string | null;
  /** Refusal of the provider override, or of the whole form. */
  providerError: string | null;
  formError: string | null;
  /** The last save committed everything. */
  justSaved: boolean;
  setName: (name: string) => void;
  setText: (text: string) => void;
  setProvider: (provider: ProviderDraft) => void;
  touchName: () => void;
  touchText: () => void;
  reloadSaved: () => void;
  save: () => Promise<void>;
}

const GENERIC_REFUSAL: EditorRefusal = {
  field: "form",
  key: "errors.settings.generic",
};

/** Parts of a draft committed by a save outcome. */
const committedPartsOf = (outcome: CommitOutcome) =>
  outcome.status === "saved"
    ? { content: true, provider: true }
    : { content: outcome.contentSaved, provider: false };

interface DraftPair {
  draft: PromptDraft;
  saved: PromptDraft;
  setDraft: Dispatch<SetStateAction<PromptDraft>>;
  setSaved: Dispatch<SetStateAction<PromptDraft>>;
}

const useDraftPair = (initial: PromptDraft): DraftPair => {
  const [draft, setDraft] = useState(initial);
  const [saved, setSaved] = useState(initial);
  return { draft, saved, setDraft, setSaved };
};

/**
 * Whether the edited prompt has been in the library: a prompt just created
 * here is not deleted while the library reload that lists it is pending.
 */
const useSeenInLibrary = (current: ListedPrompt | undefined): boolean => {
  const [seen, setSeen] = useState(Boolean(current));
  useEffect(() => {
    if (current) setSeen(true);
  }, [current]);
  return seen || Boolean(current);
};

/**
 * Change made elsewhere to the edited prompt. Without local changes the
 * editor silently takes the new values; with local changes it reports the
 * change and lets the user reload or keep their version.
 */
const useExternalChange = (
  target: EditorTarget,
  current: ListedPrompt | undefined,
  pair: DraftPair,
): ExternalChange => {
  const seen = useSeenInLibrary(current);
  const external =
    target.mode === "edit" && seen
      ? externalChangeOf(pair.saved, current)
      : "none";
  const isDirty = isDraftDirty(pair.draft, pair.saved);
  const { setDraft, setSaved } = pair;
  useEffect(() => {
    if (external !== "changed" || isDirty || !current) return;
    setSaved(draftOf(current));
    setDraft(draftOf(current));
  }, [external, isDirty, current, setDraft, setSaved]);
  return external;
};

interface SaveFlow {
  saving: boolean;
  refusal: EditorRefusal | null;
  justSaved: boolean;
  clearFeedback: () => void;
  save: () => Promise<void>;
}

/** Commits the draft and records what was saved or why it was refused. */
const useSaveFlow = (
  target: EditorTarget,
  pair: DraftPair,
  onCreated: (id: string) => void,
): SaveFlow => {
  const [saving, setSaving] = useState(false);
  const [refusal, setRefusal] = useState<EditorRefusal | null>(null);
  const [justSaved, setJustSaved] = useState(false);
  const save = async () => {
    setSaving(true);
    setRefusal(null);
    setJustSaved(false);
    try {
      const outcome =
        target.mode === "new"
          ? await createPrompt(pair.draft, EMPTY_DRAFT)
          : await updatePrompt(target.id, pair.draft, pair.saved);
      applyOutcome(outcome);
    } catch (failure) {
      console.error("Failed to save the prompt:", failure);
      setRefusal(GENERIC_REFUSAL);
    } finally {
      setSaving(false);
    }
  };
  const applyOutcome = (outcome: CommitOutcome) => {
    const base = target.mode === "new" ? EMPTY_DRAFT : pair.saved;
    const next = savedAfterCommit(pair.draft, base, committedPartsOf(outcome));
    pair.setSaved(next);
    if (outcome.status === "saved") {
      pair.setDraft(next);
      setJustSaved(true);
    } else {
      const field = refusalFieldOf(outcome.error);
      setRefusal({ field, key: settingsErrorKey(outcome.error) });
    }
    // A new prompt exists once created, even if its override was refused:
    // the editor then edits it.
    if (target.mode === "new" && outcome.id) onCreated(outcome.id);
  };
  const clearFeedback = useCallback(() => {
    setRefusal(null);
    setJustSaved(false);
  }, []);
  return { saving, refusal, justSaved, clearFeedback, save };
};

/** Name and text fields: show an issue once the field was left. */
const useTouched = () => {
  const [name, setName] = useState(false);
  const [text, setText] = useState(false);
  return {
    name,
    text,
    touchName: useCallback(() => setName(true), []),
    touchText: useCallback(() => setText(true), []),
  };
};

/**
 * Editor state for `target`.
 * @param target Prompt being edited, or a new one.
 * @param library Latest library (reloaded after every revision).
 * @param onCreated Called with the id of a new prompt once it exists.
 */
export const usePromptEditor = (
  target: EditorTarget,
  library: PromptLibraryView | null,
  onCreated: (id: string) => void,
): PromptEditorModel => {
  const prompts = useMemo(() => library?.prompts ?? [], [library]);
  const editingId = target.mode === "edit" ? target.id : null;
  const current = prompts.find((prompt) => prompt.id === editingId);
  const pair = useDraftPair(current ? draftOf(current) : EMPTY_DRAFT);
  const external = useExternalChange(target, current, pair);
  const flow = useSaveFlow(target, pair, onCreated);
  const touched = useTouched();
  const validation = validateDraft(
    pair.draft,
    otherPromptNames(prompts, editingId),
  );
  const isDirty = target.mode === "new" || isDraftDirty(pair.draft, pair.saved);
  const edit = (patch: Partial<PromptDraft>) => {
    flow.clearFeedback();
    pair.setDraft((draft) => ({ ...draft, ...patch }));
  };
  return {
    draft: pair.draft,
    validation,
    external,
    saveState: saveStateOf({
      valid: validation.valid,
      dirty: isDirty,
      saving: flow.saving,
      deleted: external === "deleted",
    }),
    ...fieldErrorsOf(validation, touched, flow.refusal),
    justSaved: flow.justSaved,
    setName: (name) => edit({ name }),
    setText: (text) => edit({ text }),
    setProvider: (provider) => edit({ provider }),
    touchName: touched.touchName,
    touchText: touched.touchText,
    reloadSaved: () => {
      if (!current) return;
      pair.setSaved(draftOf(current));
      pair.setDraft(draftOf(current));
    },
    save: flow.save,
  };
};

/**
 * Errors under each field. An empty name is explained at once (Save is
 * unavailable because of it); the other issues once the field was left.
 */
const fieldErrorsOf = (
  validation: DraftValidation,
  touched: { name: boolean; text: boolean },
  refusal: EditorRefusal | null,
) => {
  const { nameIssue, textIssue } = validation;
  return {
    nameError: fieldErrorOf(
      nameIssue && fieldIssueKey(nameIssue),
      nameIssue === "name_empty" || touched.name,
      refusal,
      "name",
    ),
    textError: fieldErrorOf(
      textIssue && fieldIssueKey(textIssue),
      touched.text,
      refusal,
      "text",
    ),
    providerError: refusal?.field === "provider" ? refusal.key : null,
    formError: refusal?.field === "form" ? refusal.key : null,
  };
};
