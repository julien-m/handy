// Pure synchronisation of the prompt editor with the library (005 AC-001):
// the draft and its saved values, the values a save just committed while
// the library has not reloaded yet, and how each reloaded library is read.
// The editor never takes a library older than its own save for a change
// made elsewhere, and keeps what the user typed while a save was running.
import type { ListedPrompt } from "@/bindings";
import type { CommitOutcome } from "./prompt-commands";
import {
  type CommittedParts,
  type EditorTarget,
  type ExternalChange,
  type PromptDraft,
  EMPTY_DRAFT,
  draftOf,
  externalChangeOf,
  isDraftDirty,
  savedAfterCommit,
} from "./prompt-draft-model";

/** Values a save committed, until a library lists them. */
export interface PendingCommit {
  values: PromptDraft;
  /** Settings revision this window knew when the save answered. */
  revision: number | null;
}

/** The editor's form values, saved values and pending commit. */
export interface EditorSync {
  draft: PromptDraft;
  saved: PromptDraft;
  pending: PendingCommit | null;
}

/** The library as loaded, with the settings revision it was loaded for. */
export interface LibrarySnapshot {
  current: ListedPrompt | undefined;
  revision: number | null;
}

/**
 * How a library is read against a pending commit:
 * - `current`: authoritative (no pending commit, or loaded after the save);
 * - `confirmed`: it lists the values just saved;
 * - `stale`: possibly loaded before the save, so it is ignored.
 */
export type LibraryReading = "current" | "confirmed" | "stale";

/**
 * Starting state of the editor.
 * @param current The edited prompt in the library, or none for a new one.
 */
export const initialSyncOf = (
  current: ListedPrompt | undefined,
): EditorSync => {
  const values = current ? draftOf(current) : EMPTY_DRAFT;
  return { draft: values, saved: values, pending: null };
};

/**
 * Whether two drafts are the same values, as typed (the name untrimmed).
 * @param left First draft.
 * @param right Second draft.
 */
export const sameDraft = (left: PromptDraft, right: PromptDraft): boolean =>
  left.name === right.name &&
  left.text === right.text &&
  left.provider.providerId === right.provider.providerId &&
  left.provider.model === right.provider.model;

/**
 * Reading of a library against the pending commit: a library that lists the
 * saved values confirms the save; one loaded for a revision newer than the
 * one known when the save answered is later than the save; any other one
 * may predate the save and is ignored.
 * @param pending Values a save committed, if not yet listed.
 * @param snapshot The edited prompt in the library and its revision.
 */
export const libraryReadingOf = (
  pending: PendingCommit | null,
  snapshot: LibrarySnapshot,
): LibraryReading => {
  if (!pending) return "current";
  const { current, revision } = snapshot;
  if (current && !isDraftDirty(draftOf(current), pending.values)) {
    return "confirmed";
  }
  const isLater =
    revision !== null &&
    pending.revision !== null &&
    revision > pending.revision;
  return isLater ? "current" : "stale";
};

/** A reconciled state and the change made elsewhere it shows. */
export interface Reconciled {
  sync: EditorSync;
  external: ExternalChange;
}

/**
 * State of the editor once a library is read. Without local changes a
 * change made elsewhere is taken silently; with local changes it is
 * reported. Returns the same `sync` object when nothing changes.
 * @param sync Current editor state.
 * @param target Prompt being edited, or a new one.
 * @param snapshot The edited prompt in the library and its revision.
 */
export const reconcileWithLibrary = (
  sync: EditorSync,
  target: EditorTarget,
  snapshot: LibrarySnapshot,
): Reconciled => {
  if (target.mode === "new") return { sync, external: "none" };
  const reading = libraryReadingOf(sync.pending, snapshot);
  if (reading === "stale") return { sync, external: "none" };
  // Confirmed or later than the save: the pending commit is settled.
  const settled = sync.pending ? { ...sync, pending: null } : sync;
  const external = externalChangeOf(settled.saved, snapshot.current);
  if (external !== "changed" || isDraftDirty(settled.draft, settled.saved)) {
    return { sync: settled, external };
  }
  // Nothing typed here: the editor takes the values saved elsewhere.
  // `changed` implies the prompt is in the library (else it is `deleted`).
  const values = draftOf(snapshot.current as ListedPrompt);
  return { sync: { draft: values, saved: values, pending: null }, external };
};

/** What a save sent and what it committed. */
export interface SaveResult {
  /** The form values when Save was clicked. */
  sent: PromptDraft;
  /** Saved values the save was compared with. */
  base: PromptDraft;
  committed: CommittedParts;
  /** Settings revision this window knew when the save answered. */
  revision: number | null;
}

/**
 * State after a save answered: the committed parts become the saved values
 * and wait for a library that lists them; the form takes the saved values
 * only when nothing was typed meanwhile, so keystrokes made during the save
 * stay (and make the form dirty again).
 * @param sync State when the save answered.
 * @param result What was sent and committed.
 */
export const syncAfterSave = (
  sync: EditorSync,
  result: SaveResult,
): EditorSync => {
  const { sent, base, committed, revision } = result;
  if (!committed.content && !committed.provider) return sync;
  const saved = savedAfterCommit(sent, base, committed);
  const isFull = committed.content && committed.provider;
  const untouched = sameDraft(sync.draft, sent);
  return {
    draft: isFull && untouched ? saved : sync.draft,
    saved,
    pending: { values: saved, revision },
  };
};

/**
 * Whether the editor shows the field issues from the start: a saved prompt
 * that is already invalid (AC-002) is opened to be fixed, so its issues are
 * highlighted at once ("Fix the highlighted fields").
 * @param target Prompt being edited, or a new one.
 * @param current The edited prompt in the library.
 */
export const showsIssuesAtOpen = (
  target: EditorTarget,
  current: ListedPrompt | undefined,
): boolean => target.mode === "edit" && (current?.issues.length ?? 0) > 0;

/**
 * Prompt the editor switches to after a creation answered: the created
 * prompt (even if its override was refused), and none once the editor was
 * closed meanwhile, so Cancel during a creation never reopens it.
 * @param target Prompt being edited, or a new one, when Save was clicked.
 * @param outcome Outcome of the save.
 * @param isOpen Whether the editor is still shown.
 */
export const createdIdToOpen = (
  target: EditorTarget,
  outcome: CommitOutcome,
  isOpen: boolean,
): string | null => (isOpen && target.mode === "new" ? outcome.id : null);
