/**
 * Pure rules of the replacement editor (feature 002): the list draft, the exchange text of the
 * bulk view and the translation keys of typed issues. Validation, normalization and duplicate
 * detection stay in Rust (`check_word_replacements`, `preview_word_replacements`): this module
 * never folds case or compares rule keys itself.
 */
import type {
  ErrorItem,
  ErrorReason,
  Field,
  IssueKind,
  ReplacementRule,
  RuleDraft,
  RuleIssue,
} from "@/bindings";

/** Rule set bound of AC-002, shown by the rule count. */
export const MAX_RULES = 1000;
/** Header the bulk view always shows; the same one export writes (AC-004). */
export const EXCHANGE_HEADER = "recognized;replacement";
/** Example line of the empty state; a rule, not prose, so it is never translated. */
export const EXAMPLE_LINE = "Cloud Code;Claude Code";

/** One rule of the editor's draft; `key` is the saved id, or a local key for a new rule. */
export interface DraftRule {
  readonly key: string;
  readonly id: string | null;
  readonly recognized: string;
  readonly replacement: string;
}

/** Text of one rule field in the draft. */
export type RuleField = "recognized" | "replacement";

/**
 * Draft rows of the saved rules, keyed by their stable ids.
 * @param rules saved rules in order
 * @returns the draft equal to the saved set
 */
export const draftOf = (rules: ReadonlyArray<ReplacementRule>): DraftRule[] =>
  rules.map((rule) => ({
    key: rule.id,
    id: rule.id,
    recognized: rule.recognized,
    replacement: rule.replacement,
  }));

/**
 * Command payload of the draft: saved rules keep their id, new rules have none.
 * @param draft editor draft
 * @returns rules for `check_word_replacements` and `save_word_replacements`
 */
export const ruleDrafts = (draft: ReadonlyArray<DraftRule>): RuleDraft[] =>
  draft.map(({ id, recognized, replacement }) => ({
    id,
    recognized,
    replacement,
  }));

/**
 * Whether the draft differs from the saved rules (content, identity or order).
 * @param saved saved rules
 * @param draft editor draft
 * @returns true when Save has something to commit
 */
export const isDraftDirty = (
  saved: ReadonlyArray<ReplacementRule>,
  draft: ReadonlyArray<DraftRule>,
): boolean =>
  saved.length !== draft.length ||
  draft.some((rule, index) => {
    const before = saved[index];
    return (
      rule.id !== before.id ||
      rule.recognized !== before.recognized ||
      rule.replacement !== before.replacement
    );
  });

/**
 * The draft with a new rule at the end (AC-001: added rules keep list order).
 * @param draft editor draft
 * @param rule recognized and replacement text as typed
 * @param key local key of the new row
 * @returns a new draft
 */
export const withAddedRule = (
  draft: ReadonlyArray<DraftRule>,
  rule: Readonly<Pick<DraftRule, RuleField>>,
  key: string,
): DraftRule[] => [...draft, { key, id: null, ...rule }];

/**
 * The draft with one field of one row changed; its id and position are kept.
 * @param draft editor draft
 * @param key row key
 * @param field edited field
 * @param value new text
 * @returns a new draft
 */
export const withEditedRule = (
  draft: ReadonlyArray<DraftRule>,
  key: string,
  field: RuleField,
  value: string,
): DraftRule[] =>
  draft.map((rule) => (rule.key === key ? { ...rule, [field]: value } : rule));

/**
 * The draft without one row.
 * @param draft editor draft
 * @param key row key
 * @returns a new draft
 */
export const withoutRule = (
  draft: ReadonlyArray<DraftRule>,
  key: string,
): DraftRule[] => draft.filter((rule) => rule.key !== key);

/**
 * Row that takes the focus after a deletion (keyboard AC): the next rule, else the previous one,
 * else none (the add form).
 * @param draft draft before the deletion
 * @param key deleted row key
 * @returns key of the row to focus, or null for the add form
 */
export const focusAfterDelete = (
  draft: ReadonlyArray<DraftRule>,
  key: string,
): string | null => {
  const index = draft.findIndex((rule) => rule.key === key);
  if (index < 0) return null;
  return draft[index + 1]?.key ?? draft[index - 1]?.key ?? null;
};

/**
 * Exchange text of saved rules, as the bulk view shows it: the header, then one
 * `recognized;replacement` line per rule in list order. AC-002 forbids `;` and tabs in
 * recognized text, so the first `;` always separates the fields.
 * @param rules saved rules
 * @returns the bulk editor text
 */
export const exchangeText = (rules: ReadonlyArray<ReplacementRule>): string =>
  [
    EXCHANGE_HEADER,
    ...rules.map((rule) => `${rule.recognized};${rule.replacement}`),
  ].join("\n");

/** Translation keys under `settings.replacements.issues`. */
const ISSUE_KEYS: Readonly<Record<string, string>> = {
  "recognized:too_short": "recognizedEmpty",
  "recognized:too_long": "recognizedTooLong",
  "recognized:forbidden_character": "recognizedForbidden",
  "recognized:control_character": "recognizedControl",
  "recognized:leading_comment": "recognizedComment",
  "recognized:duplicate_key": "duplicate",
  "replacement:too_short": "replacementEmpty",
  "replacement:too_long": "replacementTooLong",
  "replacement:forbidden_character": "replacementForbidden",
  "replacement:control_character": "replacementControl",
  "line:missing_separator": "missingSeparator",
};

/** Whole-set issues, whatever their field. */
const SET_ISSUE_KEYS: Partial<Record<IssueKind, string>> = {
  too_many_rules: "tooManyRules",
  invalid_saved_rules: "invalidSavedRules",
  unicode_data_unavailable: "unicodeUnavailable",
};

/**
 * Translation key of a typed issue (fields of a draft rule, an exchange line or the whole set).
 * @param field issue field
 * @param kind issue kind
 * @returns a key under `settings.replacements.issues`; `invalidRule` for identity issues
 */
export const issueKey = (field: Field, kind: IssueKind): string => {
  const name =
    SET_ISSUE_KEYS[kind] ?? ISSUE_KEYS[`${field}:${kind}`] ?? "invalidRule";
  return `settings.replacements.issues.${name}`;
};

/** Typed reasons of `details.items` (a refused Save or Apply) and their issue keys. */
const ITEM_KEYS: Partial<Record<ErrorReason, string>> = {
  recognized_empty: "recognizedEmpty",
  recognized_too_long: "recognizedTooLong",
  recognized_forbidden_character: "recognizedForbidden",
  recognized_control_character: "recognizedControl",
  recognized_leading_comment: "recognizedComment",
  duplicate_recognized: "duplicate",
  replacement_empty: "replacementEmpty",
  replacement_too_long: "replacementTooLong",
  replacement_forbidden_character: "replacementForbidden",
  replacement_control_character: "replacementControl",
  missing_separator: "missingSeparator",
  too_many_rules: "tooManyRules",
  invalid_saved_rules: "invalidSavedRules",
  unicode_data_unavailable: "unicodeUnavailable",
};

/**
 * Translation key of one refused position of a Save or Apply.
 * @param item refused position from `details.items`
 * @returns a key under `settings.replacements.issues`
 */
export const itemKey = (item: Pick<ErrorItem, "code">): string =>
  `settings.replacements.issues.${ITEM_KEYS[item.code] ?? "invalidRule"}`;

/**
 * Issues of one draft row.
 * @param issues issues of the whole draft
 * @param index zero-based row index
 * @returns the row's issues, in validator order
 */
export const rowIssues = (
  issues: ReadonlyArray<RuleIssue>,
  index: number,
): RuleIssue[] => issues.filter((issue) => issue.rule_index === index);

/**
 * Whole-set issues of a draft (no row), for example more than 1000 rules.
 * @param issues issues of the whole draft
 * @returns issues without a row index
 */
export const setIssues = (issues: ReadonlyArray<RuleIssue>): RuleIssue[] =>
  issues.filter((issue) => issue.rule_index === null);

/** Test-id prefixes of the contract screens (`penflow/flow-ui-contract/contract.json`). */
export const SCREEN_IDS = {
  list: "replacement-settings",
  empty: "replacement-settings-empty",
  text: "replacement-settings-bulk",
  import: "replacement-settings-import",
} as const;

/** Prefix of the editor's contract screen. */
export type ScreenId = (typeof SCREEN_IDS)[Exclude<
  keyof typeof SCREEN_IDS,
  "import"
>];

/**
 * Contract screen the editor shows: the text view, the list, or the empty state (no rule and no
 * add form open).
 * @param view rule view
 * @param showsList whether the list and its add form are shown
 * @returns the test-id prefix of that screen
 */
export const screenId = (
  view: "list" | "text",
  showsList: boolean,
): ScreenId => {
  if (view === "text") return SCREEN_IDS.text;
  return showsList ? SCREEN_IDS.list : SCREEN_IDS.empty;
};
