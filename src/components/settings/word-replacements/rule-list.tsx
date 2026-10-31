/**
 * Alphabetical rule list of the replacement editor (decision `alphabetical_display`: sorted by
 * recognized text for display only, the saved order is unchanged; the order holds still while a
 * row is edited) (contract nodes `replacement-settings.list`,
 * `.rule`, `.rule.edit`, `.rule.delete`, `.count`). Edits change the draft only; a deletion asks
 * for confirmation, then moves the focus to the next rule or, after the last row, to the add form
 * (keyboard AC "Keyboard delete").
 */
import React, { useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { isDraftLocked, useReplacementEditor } from "./editor-store";
import { focusAfterDelete } from "./editor-model";
import {
  type DraftRule,
  MAX_RULES,
  SCREEN_IDS,
  issueKey,
  itemKey,
  rowIssues,
} from "./rules-model";
import { RuleRow } from "./rule-row";
import { DeleteRuleDialog } from "./delete-rule-dialog";
import { BOUNDED_LIST_ROWS, useRevealAdded } from "../../ui/bounded-list";
import { type Placed, displayOrder } from "../../../lib/utils/alphabetical";

/** A rule row: `py-2` around a `min-h-8` line, plus its divider. */
const LIST_MAX_HEIGHT = `calc(${BOUNDED_LIST_ROWS} * (3rem + 1px) + 2px)`;

interface RuleListProps {
  /** Focuses the add form's recognized field (or the empty state's Add rule). */
  onFocusAddForm: () => void;
}

export const RuleList: React.FC<RuleListProps> = ({ onFocusAddForm }) => {
  const { t } = useTranslation();
  const draft = useReplacementEditor((state) => state.draft);
  const shown = useShownRules(draft);
  const deletion = useRuleDeletion(shown.keys, onFocusAddForm);
  const list = useRef<HTMLUListElement>(null);
  useRevealAdded(list, shown.keys);
  return (
    <>
      <ul
        ref={list}
        hidden={draft.length === 0}
        className="border border-mid-gray/20 rounded-lg divide-y divide-mid-gray/20 overflow-y-auto"
        style={{ maxHeight: LIST_MAX_HEIGHT }}
        aria-label={t("settings.replacements.title")}
        data-testid={`${SCREEN_IDS.list}-list`}
      >
        <RuleRows
          shown={shown}
          deleteButtons={deletion.deleteButtons}
          onDelete={deletion.setDeleting}
        />
      </ul>
      <RuleCount count={draft.length} />
      <DeleteRuleDialog
        rule={deletion.deleting}
        onCancel={() => deletion.setDeleting(null)}
        onConfirm={deletion.handleConfirm}
      />
    </>
  );
};

interface ShownRules {
  /** Rules in display order, each with its draft index. */
  rows: Placed<DraftRule>[];
  /** Row keys in display order. */
  keys: string[];
  editingKey: string | null;
  setEditingKey: (key: string | null) => void;
}

/**
 * Rules in alphabetical order of their recognized text; while a row is edited the order shown
 * when editing started holds, so the row does not move under the typing.
 * @param draft rules in saved order
 * @returns the rows to show and the row being edited
 */
const useShownRules = (draft: ReadonlyArray<DraftRule>): ShownRules => {
  const { i18n } = useTranslation();
  const [editingKey, setEditing] = useState<string | null>(null);
  const frozen = useRef<string[] | null>(null);
  const rows = displayOrder(
    draft,
    (rule) => rule.key,
    (rule) => rule.recognized,
    i18n.language,
    editingKey === null ? null : frozen.current,
  );
  const joined = rows.map((row) => row.item.key).join("\n");
  const keys = useMemo(
    () => (joined === "" ? [] : joined.split("\n")),
    [joined],
  );
  const setEditingKey = (key: string | null) => {
    frozen.current = key === null ? null : keys;
    setEditing(key);
  };
  return { rows, keys, editingKey, setEditingKey };
};

interface RuleDeletion {
  deleting: DraftRule | null;
  setDeleting: (rule: DraftRule | null) => void;
  deleteButtons: Map<string, HTMLButtonElement>;
  handleConfirm: () => void;
}

/**
 * Confirmed deletion of a row, then the focus on the next rule or, after the last row, on the
 * add form (`focusAfterDelete`).
 * @param keys row keys in display order
 * @param onFocusAddForm focuses the add form
 * @returns the row being confirmed and the dialog handlers
 */
const useRuleDeletion = (
  keys: ReadonlyArray<string>,
  onFocusAddForm: () => void,
): RuleDeletion => {
  const [deleting, setDeleting] = useState<DraftRule | null>(null);
  const deleteButtons = useRef(new Map<string, HTMLButtonElement>()).current;
  const handleConfirm = () => {
    if (deleting === null) return;
    const target = focusAfterDelete(keys, deleting.key);
    useReplacementEditor.getState().removeRule(deleting.key);
    setDeleting(null);
    // After the dialog has closed and returned the focus, move it past the removed row.
    requestAnimationFrame(() => {
      const next =
        target.kind === "rule" ? deleteButtons.get(target.key) : null;
      if (next) next.focus();
      else onFocusAddForm();
    });
  };
  return { deleting, setDeleting, deleteButtons, handleConfirm };
};

interface RuleRowsProps {
  shown: ShownRules;
  /** Delete buttons by row key, kept for the focus after a deletion. */
  deleteButtons: Map<string, HTMLButtonElement>;
  onDelete: (rule: DraftRule) => void;
}

/**
 * The rows in display order; errors stay bound to each rule's draft index. Edits and deletions
 * are locked while a Save is in flight.
 */
const RuleRows: React.FC<RuleRowsProps> = ({
  shown,
  deleteButtons,
  onDelete,
}) => {
  const errors = useRowErrors();
  const locked = useReplacementEditor(isDraftLocked);
  const { rows, editingKey, setEditingKey } = shown;
  return rows.map(({ item: rule, index }, position) => (
    <RuleRow
      key={rule.key}
      rule={rule}
      position={position + 1}
      errors={errors(index)}
      editing={editingKey === rule.key}
      locked={locked}
      onEditing={(editing) => setEditingKey(editing ? rule.key : null)}
      onDelete={() => onDelete(rule)}
      deleteRef={(button) => {
        if (button) deleteButtons.set(rule.key, button);
        else deleteButtons.delete(rule.key);
      }}
    />
  ));
};

/**
 * Translated errors of each row: shared-validator issues of the draft, then the positions a
 * refused Save reported (`details.items`).
 * @returns a function from row index to its messages
 */
const useRowErrors = (): ((index: number) => string[]) => {
  const { t } = useTranslation();
  const issues = useReplacementEditor((state) => state.issues);
  const failure = useReplacementEditor((state) => state.failure);
  return (index) => [
    ...rowIssues(issues, index).map((issue) =>
      t(issueKey(issue.field, issue.kind)),
    ),
    ...(failure?.items ?? [])
      .filter((item) => item.index === index)
      .map((item) => t(itemKey(item))),
  ];
};

const RuleCount: React.FC<{ count: number }> = ({ count }) => {
  const { t, i18n } = useTranslation();
  return (
    <p
      className="text-sm text-mid-gray"
      data-testid={`${SCREEN_IDS.list}-count`}
    >
      {t("settings.replacements.count", {
        count,
        max: MAX_RULES.toLocaleString(i18n.language),
      })}
    </p>
  );
};
