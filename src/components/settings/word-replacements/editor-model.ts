/**
 * Pure decisions of the replacement editor (feature 002) around the settings revision and the
 * screen: whether a committed settings change moves the draft's base revision, what an
 * uncertain Save outcome means once the rules are reread, when edits are locked, and where the
 * focus goes after a deletion. The stores and components apply these decisions.
 */
import type { ReplacementRule, RuleDraft } from "@/bindings";
import type { CommitFailure } from "./preview-model";

/**
 * Whether two rule lists are equal in identity, content and order.
 * @param left rules
 * @param right rules
 * @returns true when every position holds the same rule
 */
export const sameRules = (
  left: ReadonlyArray<ReplacementRule>,
  right: ReadonlyArray<ReplacementRule>,
): boolean =>
  left.length === right.length &&
  left.every((rule, index) => {
    const other = right[index];
    return (
      rule.id === other.id &&
      rule.recognized === other.recognized &&
      rule.replacement === other.replacement
    );
  });

/**
 * Whether saved rules are exactly a sent draft: same content and order, and the same id where
 * the draft had one (a new rule receives its id when committed).
 * @param sent draft sent by Save
 * @param rules rules read back
 * @returns true when `rules` is what the Save would have stored
 */
export const rulesMatchDraft = (
  sent: ReadonlyArray<RuleDraft>,
  rules: ReadonlyArray<ReplacementRule>,
): boolean =>
  sent.length === rules.length &&
  sent.every((draft, index) => {
    const rule = rules[index];
    return (
      (draft.id === null || draft.id === rule.id) &&
      draft.recognized === rule.recognized &&
      draft.replacement === rule.replacement
    );
  });

/** Editor facts that decide how a committed settings change is followed. */
export interface FollowInput {
  readonly ready: boolean;
  readonly saving: boolean;
  readonly dirty: boolean;
  readonly revision: number | null;
  readonly saved: ReadonlyArray<ReplacementRule>;
}

/**
 * How the list follows a committed settings change, once the rules were reread:
 * - `ignore`: nothing to do (a Save in flight applies its own result, or the view is older);
 * - `rebase`: the rules did not change (another setting did), so the draft, clean or not, moves
 *   to the new revision; the backend compares the global settings revision, so keeping the old
 *   one would turn the next Save into a false conflict;
 * - `replace`: the rules changed and the draft is clean, so the list shows the new rules;
 * - a changed draft over changed rules keeps its base revision: its Save reports the conflict.
 * @param state editor facts
 * @param view rules and revision read after the change
 * @returns the decision
 */
export const followDecision = (
  state: FollowInput,
  view: Readonly<{ revision: number; rules: ReadonlyArray<ReplacementRule> }>,
): "ignore" | "rebase" | "replace" => {
  if (!state.ready || state.saving) return "ignore";
  if (state.revision !== null && view.revision <= state.revision) {
    return "ignore";
  }
  if (sameRules(view.rules, state.saved)) return "rebase";
  return state.dirty ? "ignore" : "replace";
};

/**
 * Meaning of a Save whose outcome is uncertain, once the rules are reread:
 * - `committed`: the rules are the sent draft, so the Save happened;
 * - `notCommitted`: the rules are still the ones the draft started from;
 * - `conflict`: the rules are something else, so another writer changed them.
 * @param sent draft sent by Save
 * @param before saved rules the draft started from
 * @param reread rules read after the uncertain outcome
 * @returns the outcome
 */
export const uncertainOutcome = (
  sent: ReadonlyArray<RuleDraft>,
  before: ReadonlyArray<ReplacementRule>,
  reread: ReadonlyArray<ReplacementRule>,
): "committed" | "notCommitted" | "conflict" => {
  if (rulesMatchDraft(sent, reread)) return "committed";
  return sameRules(before, reread) ? "notCommitted" : "conflict";
};

/**
 * Failure shown once an uncertain Save is settled as not committed or as a conflict. Not
 * committed: the saved rules are known to be unchanged. Conflict: Save stays disabled until
 * Reload, and the outcome stays uncertain because the Save may be part of what changed.
 * @param failure failure of the uncertain Save
 * @param outcome settled outcome
 * @returns the failure to show
 */
export const settledFailure = (
  failure: CommitFailure,
  outcome: "notCommitted" | "conflict",
): CommitFailure =>
  outcome === "conflict"
    ? { ...failure, conflict: true }
    : { ...failure, uncertain: false };

/**
 * Failure kept when the user edits after a refusal: a revision conflict stays until Reload (a
 * Save would conflict again); any other refusal is cleared by the edit.
 * @param failure failure on screen
 * @returns the failure to keep
 */
export const failureAfterEdit = (
  failure: CommitFailure | null,
): CommitFailure | null => (failure?.conflict === true ? failure : null);

/**
 * Whether the list and its add form are shown rather than the empty state: some rule, an add
 * form opened in this session, or a changed draft (deleting the last rule keeps the list, so the
 * focus can move to the add form and Save can commit the deletion).
 * @param draftLength rules in the draft
 * @param adding whether the add form was opened
 * @param dirty whether the draft differs from the saved rules
 * @returns true for the list
 */
export const showsRuleList = (
  draftLength: number,
  adding: boolean,
  dirty: boolean,
): boolean => draftLength > 0 || adding || dirty;

/** Where the focus goes after a confirmed deletion (keyboard AC "Keyboard delete"). */
export type DeleteFocus =
  | { readonly kind: "rule"; readonly key: string }
  | { readonly kind: "addForm" };

/**
 * Focus after deleting a row: the next rule, else the add form (never the previous rule).
 * @param keys row keys before the deletion, in order
 * @param deleted deleted row key
 * @returns the focus target
 */
export const focusAfterDelete = (
  keys: ReadonlyArray<string>,
  deleted: string,
): DeleteFocus => {
  const index = keys.indexOf(deleted);
  const next = index < 0 ? undefined : keys[index + 1];
  return next === undefined ? { kind: "addForm" } : { kind: "rule", key: next };
};
