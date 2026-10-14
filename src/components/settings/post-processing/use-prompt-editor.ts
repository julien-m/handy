// State of the prompt editor (005 AC-001, Behavioral AC `has_validation` and
// `is_submittable`): the draft against its saved values, live validation,
// the Save state, backend refusals per field, and a prompt changed or
// deleted elsewhere while it is open. The rules live in the pure
// `prompt-editor-sync.ts` and `prompt-draft-model.ts`.
import {
  type Dispatch,
  type MutableRefObject,
  type SetStateAction,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { settingsErrorKey } from "../../../stores/settings-errors";
import { useSettingsStore } from "../../../stores/settingsStore";
import {
  type CommitOutcome,
  createPrompt,
  updatePrompt,
} from "./prompt-commands";
import {
  type DraftValidation,
  type EditorRefusal,
  type EditorTarget,
  type ExternalChange,
  type PromptDraft,
  type SaveState,
  EMPTY_DRAFT,
  fieldErrorOf,
  isDraftDirty,
  refusalFieldOf,
  saveStateOf,
  validateDraft,
} from "./prompt-draft-model";
import {
  type EditorSync,
  type LibrarySnapshot,
  createdIdToOpen,
  initialSyncOf,
  reconcileWithLibrary,
  showsIssuesAtOpen,
  syncAfterSave,
} from "./prompt-editor-sync";
import { fieldIssueKey, otherPromptNames } from "./prompt-library-model";
import type { ProviderDraft } from "./prompt-provider-model";
import type { PromptLibraryState } from "./use-prompt-library";

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

type SetSync = Dispatch<SetStateAction<EditorSync>>;

/**
 * Editor state reconciled with each loaded library: a library older than the
 * editor's own save is ignored (see `reconcileWithLibrary`). The render uses
 * the reconciled state at once; the effect stores it by reconciling the
 * latest state again, so a keystroke made in between is never overwritten.
 */
const useEditorSync = (target: EditorTarget, snapshot: LibrarySnapshot) => {
  const [stored, setSync] = useState(() => initialSyncOf(snapshot.current));
  const { sync, external } = reconcileWithLibrary(stored, target, snapshot);
  useEffect(() => {
    if (sync === stored) return;
    setSync((now) => reconcileWithLibrary(now, target, snapshot).sync);
  }, [sync, stored, target, snapshot]);
  return { sync, external, setSync };
};

/** Whether the component is still mounted, for answers that come late. */
const useMounted = (): MutableRefObject<boolean> => {
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  return mounted;
};

/** Parts of a draft committed by a save outcome. */
const committedPartsOf = (outcome: CommitOutcome) =>
  outcome.status === "saved"
    ? { content: true, provider: true }
    : { content: outcome.contentSaved, provider: false };

/**
 * Sends `sent` as a new prompt or as the changes of the edited one.
 * @throws Error when a command cannot be invoked at all.
 */
const commitDraft = (
  target: EditorTarget,
  sent: PromptDraft,
  base: PromptDraft,
): Promise<CommitOutcome> =>
  target.mode === "new"
    ? createPrompt(sent, base)
    : updatePrompt(target.id, sent, base);

interface SaveFeedback {
  saving: boolean;
  refusal: EditorRefusal | null;
  justSaved: boolean;
}

const IDLE_FEEDBACK: SaveFeedback = {
  saving: false,
  refusal: null,
  justSaved: false,
};

/** Feedback of a save outcome: saved, or the refusal under its field. */
const feedbackOf = (outcome: CommitOutcome): SaveFeedback =>
  outcome.status === "saved"
    ? { ...IDLE_FEEDBACK, justSaved: true }
    : {
        ...IDLE_FEEDBACK,
        refusal: {
          field: refusalFieldOf(outcome.error),
          key: settingsErrorKey(outcome.error),
        },
      };

interface SaveFlow extends SaveFeedback {
  clearFeedback: () => void;
  save: () => Promise<void>;
}

/**
 * Commits the draft and records what was saved or why it was refused. An
 * answer that arrives after the editor closed changes nothing (a creation
 * then never reopens the editor).
 */
const useSaveFlow = (
  target: EditorTarget,
  sync: EditorSync,
  setSync: SetSync,
  onCreated: (id: string) => void,
): SaveFlow => {
  const [feedback, setFeedback] = useState(IDLE_FEEDBACK);
  const mounted = useMounted();
  const save = async () => {
    const sent = sync.draft;
    const base = target.mode === "new" ? EMPTY_DRAFT : sync.saved;
    setFeedback({ ...IDLE_FEEDBACK, saving: true });
    try {
      const outcome = await commitDraft(target, sent, base);
      const createdId = createdIdToOpen(target, outcome, mounted.current);
      if (!mounted.current) return;
      const revision = useSettingsStore.getState().revision;
      const committed = committedPartsOf(outcome);
      setSync((now) => syncAfterSave(now, { sent, base, committed, revision }));
      setFeedback(feedbackOf(outcome));
      if (createdId) onCreated(createdId);
    } catch (failure) {
      console.error("Failed to save the prompt:", failure);
      if (mounted.current)
        setFeedback({ ...IDLE_FEEDBACK, refusal: GENERIC_REFUSAL });
    }
  };
  const clearFeedback = useCallback(() => setFeedback(IDLE_FEEDBACK), []);
  return { ...feedback, clearFeedback, save };
};

/**
 * Name and text fields: show an issue once the field was left, or from the
 * start when `showAtOnce` (an invalid saved prompt opened to be fixed).
 */
const useTouched = (showAtOnce: boolean) => {
  const [name, setName] = useState(showAtOnce);
  const [text, setText] = useState(showAtOnce);
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
 * @param libraryState Latest library (reloaded after every revision).
 * @param onCreated Called with the id of a new prompt once it exists.
 */
export const usePromptEditor = (
  target: EditorTarget,
  libraryState: PromptLibraryState,
  onCreated: (id: string) => void,
): PromptEditorModel => {
  const prompts = libraryState.library?.prompts ?? [];
  const editingId = target.mode === "edit" ? target.id : null;
  const current = prompts.find((prompt) => prompt.id === editingId);
  const snapshot = { current, revision: libraryState.loadedRevision };
  const { sync, external, setSync } = useEditorSync(target, snapshot);
  const flow = useSaveFlow(target, sync, setSync, onCreated);
  const touched = useTouched(showsIssuesAtOpen(target, current));
  const names = otherPromptNames(prompts, editingId);
  const validation = validateDraft(sync.draft, names);
  return {
    draft: sync.draft,
    validation,
    external,
    saveState: saveStateOf({
      valid: validation.valid,
      dirty: target.mode === "new" || isDraftDirty(sync.draft, sync.saved),
      saving: flow.saving,
      deleted: external === "deleted",
    }),
    ...fieldErrorsOf(validation, touched, flow.refusal),
    justSaved: flow.justSaved,
    ...editActionsOf(setSync, flow.clearFeedback, touched),
    reloadSaved: () => current && setSync(initialSyncOf(current)),
    save: flow.save,
  };
};

/**
 * Field setters (an edit clears the last saved/refused feedback) and the
 * "field left" handlers.
 */
const editActionsOf = (
  setSync: SetSync,
  clearFeedback: () => void,
  touched: { touchName: () => void; touchText: () => void },
) => {
  const edit = (patch: Partial<PromptDraft>) => {
    clearFeedback();
    setSync((now) => ({ ...now, draft: { ...now.draft, ...patch } }));
  };
  return {
    setName: (name: string) => edit({ name }),
    setText: (text: string) => edit({ text }),
    setProvider: (provider: ProviderDraft) => edit({ provider }),
    touchName: touched.touchName,
    touchText: touched.touchText,
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
