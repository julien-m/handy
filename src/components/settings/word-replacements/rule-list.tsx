/**
 * Saved-order rule list of the replacement editor (contract nodes `replacement-settings.list`,
 * `.rule`, `.rule.edit`, `.rule.delete`, `.count`). Edits change the draft only; a deletion asks
 * for confirmation, then moves the focus to the next rule or to the add form (keyboard AC).
 */
import React, { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useReplacementEditor } from "./editor-store";
import {
  type DraftRule,
  MAX_RULES,
  SCREEN_IDS,
  focusAfterDelete,
  issueKey,
  itemKey,
  rowIssues,
} from "./rules-model";
import { RuleRow } from "./rule-row";
import { DeleteRuleDialog } from "./delete-rule-dialog";

interface RuleListProps {
  recognizedRef: React.RefObject<HTMLInputElement>;
}

export const RuleList: React.FC<RuleListProps> = ({ recognizedRef }) => {
  const { t } = useTranslation();
  const draft = useReplacementEditor((state) => state.draft);
  const errors = useRowErrors();
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<DraftRule | null>(null);
  const deleteButtons = useRef(new Map<string, HTMLButtonElement>());
  const confirmDelete = () => {
    if (deleting === null) return;
    const next = focusAfterDelete(draft, deleting.key);
    useReplacementEditor.getState().removeRule(deleting.key);
    setDeleting(null);
    // After the dialog has closed and returned the focus, move it past the removed row.
    requestAnimationFrame(() => {
      const target = next === null ? null : deleteButtons.current.get(next);
      (target ?? recognizedRef.current)?.focus();
    });
  };
  return (
    <>
      <ul
        hidden={draft.length === 0}
        className="border border-mid-gray/20 rounded-lg divide-y divide-mid-gray/20"
        aria-label={t("settings.replacements.title")}
        data-testid={`${SCREEN_IDS.list}-list`}
      >
        {draft.map((rule, index) => (
          <RuleRow
            key={rule.key}
            rule={rule}
            position={index + 1}
            errors={errors(index)}
            editing={editingKey === rule.key}
            onEditing={(editing) => setEditingKey(editing ? rule.key : null)}
            onDelete={() => setDeleting(rule)}
            deleteRef={(button) => {
              if (button) deleteButtons.current.set(rule.key, button);
              else deleteButtons.current.delete(rule.key);
            }}
          />
        ))}
      </ul>
      <RuleCount count={draft.length} />
      <DeleteRuleDialog
        rule={deleting}
        onCancel={() => setDeleting(null)}
        onConfirm={confirmDelete}
      />
    </>
  );
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
