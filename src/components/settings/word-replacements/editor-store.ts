/**
 * List editor state of the replacement rules (feature 002, AC-001/AC-002): the saved rules with
 * the revision they were read at, the draft edited by the list, its shared-validator issues and
 * the single Save commit. Save is always bound to the revision the draft started from, so a
 * change made elsewhere (CLI, another window) is a revision conflict, never a silent overwrite.
 */
import { create } from "zustand";
import {
  commands,
  type ReplacementRule,
  type ReplacementRulesView,
  type RuleIssue,
} from "@/bindings";
import {
  type DraftRule,
  type RuleField,
  draftOf,
  isDraftDirty,
  ruleDrafts,
  setIssues,
  rowIssues,
  withAddedRule,
  withEditedRule,
  withoutRule,
} from "./rules-model";
import {
  type CommitFailure,
  UNEXPECTED_FAILURE,
  commitFailure,
} from "./preview-model";

/** Loading of the saved rules. */
export type LoadState = "loading" | "ready" | "failed";
/** Save lifecycle (visual states idle, loading, success, error). */
export type CommitState = "idle" | "saving" | "saved" | "failed";
/** Rule view of the editor (AC-001: both edit the same rules). */
export type EditorView = "list" | "text";

export interface EditorStore {
  load: LoadState;
  saved: ReplacementRule[];
  /** Revision of `saved`; the base of the next Save. */
  revision: number | null;
  draft: DraftRule[];
  /** Shared-validator issues of `draft`. */
  issues: RuleIssue[];
  checking: boolean;
  commit: CommitState;
  failure: CommitFailure | null;
  view: EditorView;
  loadRules: () => Promise<void>;
  refreshAfterChange: (revision: number) => Promise<void>;
  checkCandidate: (
    rule: Readonly<Pick<DraftRule, RuleField>>,
  ) => Promise<RuleIssue[]>;
  addRule: (rule: Readonly<Pick<DraftRule, RuleField>>) => Promise<boolean>;
  editRule: (key: string, field: RuleField, value: string) => void;
  removeRule: (key: string) => void;
  discard: () => void;
  save: () => Promise<boolean>;
  acceptCommitted: (view: ReplacementRulesView) => void;
  setView: (view: EditorView) => void;
}

let checkSequence = 0;
let newRuleSequence = 0;

export const useReplacementEditor = create<EditorStore>()(() => ({
  load: "loading",
  saved: [],
  revision: null,
  draft: [],
  issues: [],
  checking: false,
  commit: "idle",
  failure: null,
  view: "list",
  loadRules,
  refreshAfterChange,
  checkCandidate,
  addRule,
  editRule,
  removeRule,
  discard,
  save,
  acceptCommitted,
  setView: (view) => patch({ view }),
}));

const editor = (): EditorStore => useReplacementEditor.getState();
const patch = (state: Partial<EditorStore>): void =>
  useReplacementEditor.setState(state);

/**
 * Whether the list draft differs from the saved rules.
 * @param state editor state
 * @returns true when Save has something to commit
 */
export const isListDirty = (state: EditorStore): boolean =>
  isDraftDirty(state.saved, state.draft);

/**
 * Whether Save may commit: a changed draft without issues, checked, no commit running and no
 * unresolved revision conflict.
 * @param state editor state
 * @returns true when Save is enabled
 */
export const canSave = (state: EditorStore): boolean =>
  isListDirty(state) &&
  state.issues.length === 0 &&
  !state.checking &&
  state.commit !== "saving" &&
  // After a revision conflict only Reload can continue: a retry would conflict again.
  state.failure?.conflict !== true;

/**
 * Read the saved rules and reset the draft to them.
 * Side effects: one `get_word_replacements` call.
 */
async function loadRules(): Promise<void> {
  patch({ load: editor().revision === null ? "loading" : editor().load });
  try {
    acceptView(await commands.getWordReplacements(), "idle");
    patch({ load: "ready" });
  } catch (error) {
    console.error("Failed to load replacement rules:", error);
    patch({ load: "failed" });
  }
}

/**
 * React to a committed settings change of `word_replacements`. A clean editor shows the new
 * rules; a dirty one keeps its draft and base revision, so its Save reports the conflict.
 * @param revision revision carried by the `settings-changed` event
 */
async function refreshAfterChange(revision: number): Promise<void> {
  const state = editor();
  // A commit in flight applies its own result; an older event changes nothing.
  if (state.commit === "saving") return;
  if (state.revision !== null && revision <= state.revision) return;
  if (isListDirty(state)) return;
  try {
    const view = await commands.getWordReplacements();
    const latest = editor();
    if (isListDirty(latest) || latest.commit === "saving") return;
    if (latest.revision !== null && view.revision < latest.revision) return;
    acceptView(view, latest.commit === "saved" ? "saved" : "idle");
  } catch (error) {
    console.error("Failed to refresh replacement rules:", error);
  }
}

/** Replace the saved rules and the draft with a committed view. */
function acceptView(view: ReplacementRulesView, commit: CommitState): void {
  patch({
    saved: view.rules,
    revision: view.revision,
    draft: draftOf(view.rules),
    issues: [],
    checking: false,
    commit,
    failure: null,
  });
}

/**
 * The rules saved by Save, Apply or an import: shown with the saved confirmation.
 * @param view committed rules and revision
 */
function acceptCommitted(view: ReplacementRulesView): void {
  acceptView(view, "saved");
}

/**
 * Issues of a rule that would be added at the end of the draft (shared validator, so duplicate
 * keys follow the same Unicode folding as Save).
 * @param rule candidate fields
 * @returns the candidate's issues and whole-set issues; empty when it can be added
 */
async function checkCandidate(
  rule: Readonly<Pick<DraftRule, RuleField>>,
): Promise<RuleIssue[]> {
  const { draft } = editor();
  const candidate = withAddedRule(draft, rule, "candidate");
  const issues = await commands.checkWordReplacements(ruleDrafts(candidate));
  return [...rowIssues(issues, draft.length), ...setIssues(issues)];
}

/**
 * Add a rule to the draft when the shared validator accepts it.
 * @param rule fields as typed
 * @returns true when the rule was added
 */
async function addRule(
  rule: Readonly<Pick<DraftRule, RuleField>>,
): Promise<boolean> {
  try {
    if ((await checkCandidate(rule)).length > 0) return false;
  } catch (error) {
    console.error("Failed to check a replacement rule:", error);
    return false;
  }
  newRuleSequence += 1;
  setDraft(withAddedRule(editor().draft, rule, `new-${newRuleSequence}`));
  return true;
}

function editRule(key: string, field: RuleField, value: string): void {
  setDraft(withEditedRule(editor().draft, key, field, value));
}

function removeRule(key: string): void {
  setDraft(withoutRule(editor().draft, key));
}

/** Drop the unsaved draft: the list shows the saved rules again. */
function discard(): void {
  checkSequence += 1;
  const { saved } = editor();
  patch({
    draft: draftOf(saved),
    issues: [],
    checking: false,
    failure: null,
    commit: "idle",
  });
}

/** Store a new draft and check it with the shared validator; a newer check wins. */
function setDraft(draft: DraftRule[]): void {
  checkSequence += 1;
  const sequence = checkSequence;
  patch({ draft, checking: true, commit: "idle", failure: null });
  commands
    .checkWordReplacements(ruleDrafts(draft))
    .then((issues) => {
      if (sequence === checkSequence) patch({ issues, checking: false });
    })
    .catch((error: unknown) => {
      console.error("Failed to check replacement rules:", error);
      if (sequence === checkSequence) patch({ checking: false });
    });
}

/**
 * Save the complete draft in one commit at the revision it started from (AC-001).
 * Side effects: one `save_word_replacements` call; on an uncertain outcome the rules are reread.
 * @returns true when the rules were saved
 */
async function save(): Promise<boolean> {
  const state = editor();
  if (!canSave(state) || state.revision === null) return false;
  patch({ commit: "saving", failure: null });
  try {
    const result = await commands.saveWordReplacements(
      ruleDrafts(state.draft),
      state.revision,
    );
    if (result.status === "ok") {
      acceptCommitted(result.data);
      return true;
    }
    return failSave(commitFailure(result.error));
  } catch (error) {
    console.error("Failed to save replacement rules:", error);
    return failSave(UNEXPECTED_FAILURE);
  }
}

/** Keep the draft and show why Save failed; an uncertain outcome rereads the saved rules. */
function failSave(failure: CommitFailure): false {
  patch({ commit: "failed", failure });
  if (failure.uncertain) void rereadSaved();
  return false;
}

/** Reread the saved rules without touching the draft (an uncertain commit outcome). */
async function rereadSaved(): Promise<void> {
  try {
    const view = await commands.getWordReplacements();
    patch({ saved: view.rules, revision: view.revision });
  } catch (error) {
    console.error("Failed to reread replacement rules:", error);
  }
}
