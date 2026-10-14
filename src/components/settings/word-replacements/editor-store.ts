/**
 * List editor state of the replacement rules (feature 002, AC-001/AC-002): the saved rules with
 * the revision they were read at, the draft edited by the list, its shared-validator issues and
 * the single Save commit. Save is bound to the revision the draft started from, so a change of
 * the rules made elsewhere (CLI, another window) is a revision conflict, never a silent
 * overwrite. The backend compares the global settings revision: a committed change of another
 * setting moves the base revision (`followSettingsChange`), and a Save refused by such a change
 * is retried once, so only a real change of the rules is reported as a conflict.
 */
import { create } from "zustand";
import {
  commands,
  type ReplacementRule,
  type ReplacementRulesView,
  type Result,
  type RuleDraft,
  type RuleIssue,
  type SettingsCommandError,
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
import {
  failureAfterEdit,
  followDecision,
  sameRules,
  settledFailure,
  uncertainOutcome,
} from "./editor-model";

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
  followSettingsChange: (revision: number) => Promise<void>;
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
  followSettingsChange,
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
 * Whether the draft is locked: a Save in flight replaces it with what it committed, so edits
 * typed meanwhile would be lost (fields are read-only and the store ignores edits).
 * @param state editor state
 * @returns true while a Save is in flight
 */
export const isDraftLocked = (state: Pick<EditorStore, "commit">): boolean =>
  state.commit === "saving";

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
 * Follow a committed settings change (any key): the rules are reread and, per
 * `followDecision`, the base revision moves (rules unchanged), a clean list shows the new rules,
 * or a changed draft keeps its base revision so its Save reports the conflict.
 * Side effects: one `get_word_replacements` call unless the event is older than the editor.
 * @param revision revision carried by the `settings-changed` event
 */
async function followSettingsChange(revision: number): Promise<void> {
  const state = editor();
  if (state.revision !== null && revision <= state.revision) return;
  try {
    applyFollow(await commands.getWordReplacements());
  } catch (error) {
    console.error("Failed to refresh replacement rules:", error);
  }
}

/** Apply the follow decision to the editor as it is now (it may have changed meanwhile). */
function applyFollow(view: ReplacementRulesView): void {
  const latest = editor();
  const decision = followDecision(
    {
      ready: latest.load === "ready",
      saving: latest.commit === "saving",
      dirty: isListDirty(latest),
      revision: latest.revision,
      saved: latest.saved,
    },
    view,
  );
  if (decision === "rebase") patch({ revision: view.revision });
  if (decision === "replace") {
    acceptView(view, latest.commit === "saved" ? "saved" : "idle");
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
  // A Save started during the check owns the draft; the form keeps the typed rule.
  if (isDraftLocked(editor())) return false;
  newRuleSequence += 1;
  setDraft(withAddedRule(editor().draft, rule, `new-${newRuleSequence}`));
  return true;
}

function editRule(key: string, field: RuleField, value: string): void {
  if (isDraftLocked(editor())) return;
  setDraft(withEditedRule(editor().draft, key, field, value));
}

function removeRule(key: string): void {
  if (isDraftLocked(editor())) return;
  setDraft(withoutRule(editor().draft, key));
}

/**
 * Drop the unsaved draft: the list shows the saved rules again. After a conflict the rules known
 * here are stale, so they are read again (like Reload).
 */
function discard(): void {
  if (editor().failure?.conflict === true) {
    void loadRules();
    return;
  }
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
  patch({
    draft,
    checking: true,
    commit: "idle",
    failure: failureAfterEdit(editor().failure),
  });
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
 * Side effects: `save_word_replacements` (retried once after a conflict caused by another
 * setting); on an uncertain outcome the rules are reread to settle it.
 * @returns true when the rules were saved
 */
async function save(): Promise<boolean> {
  const state = editor();
  if (!canSave(state) || state.revision === null) return false;
  patch({ commit: "saving", failure: null });
  const sent = ruleDrafts(state.draft);
  try {
    const result = await saveAtCurrentRevision(
      sent,
      state.revision,
      state.saved,
    );
    if (result.status === "ok") {
      acceptCommitted(result.data);
      return true;
    }
    return failSave(commitFailure(result.error), sent, state.saved);
  } catch (error) {
    console.error("Failed to save replacement rules:", error);
    return failSave(UNEXPECTED_FAILURE, sent, state.saved);
  }
}

/**
 * Save `sent` at `revision`. A conflict while the saved rules are still `saved` comes from
 * another setting (the backend compares the global revision): the Save is retried once at the
 * current revision. A real change of the rules stays a conflict.
 * @param sent draft to save
 * @param revision base revision of the draft
 * @param saved rules the draft started from
 * @returns the reply of the last Save
 */
async function saveAtCurrentRevision(
  sent: RuleDraft[],
  revision: number,
  saved: ReadonlyArray<ReplacementRule>,
): Promise<Result<ReplacementRulesView, SettingsCommandError>> {
  const first = await commands.saveWordReplacements(sent, revision);
  if (first.status === "ok" || first.error.code !== "revision_conflict") {
    return first;
  }
  const current = await commands.getWordReplacements();
  if (!sameRules(current.rules, saved)) return first;
  return commands.saveWordReplacements(sent, current.revision);
}

/** Keep the draft and show why Save failed; an uncertain outcome is settled by a reread. */
function failSave(
  failure: CommitFailure,
  sent: RuleDraft[],
  before: ReadonlyArray<ReplacementRule>,
): false {
  patch({ commit: "failed", failure });
  if (failure.uncertain) void settleUncertainSave(failure, sent, before);
  return false;
}

/**
 * Reread the rules after an uncertain Save and settle it (`uncertainOutcome`): the sent draft
 * is accepted as committed; otherwise the draft keeps its base revision (never the reread one,
 * which could silently overwrite another writer) and a change made elsewhere is a conflict.
 * Nothing happens if the user moved on (edit, Discard, Reload) before the answer.
 */
async function settleUncertainSave(
  failure: CommitFailure,
  sent: RuleDraft[],
  before: ReadonlyArray<ReplacementRule>,
): Promise<void> {
  try {
    const view = await commands.getWordReplacements();
    if (editor().failure !== failure) return;
    const outcome = uncertainOutcome(sent, before, view.rules);
    if (outcome === "committed") acceptCommitted(view);
    else patch({ failure: settledFailure(failure, outcome) });
  } catch (error) {
    console.error("Failed to reread replacement rules:", error);
  }
}
