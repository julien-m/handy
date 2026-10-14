/**
 * Saved-order rule list of the replacement editor (contract nodes `replacement-settings.list`,
 * `.rule`, `.rule.edit`, `.rule.delete`, `.count`). Edits change the draft only; a deletion asks
 * for confirmation, then moves the focus to the next rule or, after the last row, to the add form
 * (keyboard AC "Keyboard delete").
 */
import React, { useRef, useState } from "react";
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

interface RuleListProps {
  /** Focuses the add form's recognized field (or the empty state's Add rule). */
  onFocusAddForm: () => void;
}

export const RuleList: React.FC<RuleListProps> = ({ onFocusAddForm }) => {
  const { t } = useTranslation();
  const draft = useReplacementEditor((state) => state.draft);
  const deletion = useRuleDeletion(draft, onFocusAddForm);
  return (
    <>
      <ul
        hidden={draft.length === 0}
        className="border border-mid-gray/20 rounded-lg divide-y divide-mid-gray/20"
        aria-label={t("settings.replacements.title")}
        data-testid={`${SCREEN_IDS.list}-list`}
      >
        <RuleRows
          draft={draft}
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

interface RuleDeletion {
  deleting: DraftRule | null;
  setDeleting: (rule: DraftRule | null) => void;
  deleteButtons: Map<string, HTMLButtonElement>;
  handleConfirm: () => void;
}

/**
 * Confirmed deletion of a row, then the focus on the next rule or, after the last row, on the
 * add form (`focusAfterDelete`).
 * @param draft rows shown
 * @param onFocusAddForm focuses the add form
 * @returns the row being confirmed and the dialog handlers
 */
const useRuleDeletion = (
  draft: ReadonlyArray<DraftRule>,
  onFocusAddForm: () => void,
): RuleDeletion => {
  const [deleting, setDeleting] = useState<DraftRule | null>(null);
  const deleteButtons = useRef(new Map<string, HTMLButtonElement>()).current;
  const handleConfirm = () => {
    if (deleting === null) return;
    const keys = draft.map((rule) => rule.key);
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
  draft: ReadonlyArray<DraftRule>;
  /** Delete buttons by row key, kept for the focus after a deletion. */
  deleteButtons: Map<string, HTMLButtonElement>;
  onDelete: (rule: DraftRule) => void;
}

/** The rows of the draft; edits and deletions are locked while a Save is in flight. */
const RuleRows: React.FC<RuleRowsProps> = ({
  draft,
  deleteButtons,
  onDelete,
}) => {
  const errors = useRowErrors();
  const locked = useReplacementEditor(isDraftLocked);
  const [editingKey, setEditingKey] = useState<string | null>(null);
  return draft.map((rule, index) => (
    <RuleRow
      key={rule.key}
      rule={rule}
      position={index + 1}
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
