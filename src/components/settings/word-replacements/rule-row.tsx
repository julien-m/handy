/**
 * One rule of the replacement list: recognized text → replacement, with Edit (inline, in the
 * draft) and Delete. Enter or Done leaves the inline edit; Escape restores the row as it was
 * when the edit started. While a Save is in flight the row is locked: its fields are read-only
 * and its buttons disabled, so nothing typed is overwritten by the saved rules.
 */
import React, { useRef } from "react";
import { useTranslation } from "react-i18next";
import { ArrowRight, Check, Pencil, Trash2 } from "lucide-react";
import { Input } from "../../ui/Input";
import { useReplacementEditor } from "./editor-store";
import { type DraftRule, type RuleField, SCREEN_IDS } from "./rules-model";

const ROW_ID = `${SCREEN_IDS.list}-rule`;
const ICON_BUTTON =
  "p-1 rounded-md hover:bg-logo-primary/30 cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-logo-primary disabled:opacity-50 disabled:cursor-not-allowed";

interface RuleRowProps {
  rule: DraftRule;
  position: number;
  errors: string[];
  editing: boolean;
  /** A Save is in flight. */
  locked: boolean;
  onEditing: (editing: boolean) => void;
  onDelete: () => void;
  deleteRef: (button: HTMLButtonElement | null) => void;
}

export const RuleRow: React.FC<RuleRowProps> = (props) => {
  const { rule, errors } = props;
  return (
    <li
      className="px-4 py-2 space-y-1"
      data-testid={ROW_ID}
      data-instance-id={`rule-${rule.key}`}
    >
      <div className="flex items-center gap-2 min-h-8">
        {props.editing ? <RowEditor {...props} /> : <RowText rule={rule} />}
        <RowActions {...props} />
      </div>
      {errors.length > 0 && (
        <ul className="text-xs text-red-400 space-y-0.5" role="alert">
          {errors.map((message) => (
            <li key={message}>{message}</li>
          ))}
        </ul>
      )}
    </li>
  );
};

const RowText: React.FC<{ rule: DraftRule }> = ({ rule }) => (
  <>
    <span
      className="flex-1 min-w-0 truncate text-sm"
      data-testid={`${ROW_ID}-recognized`}
    >
      {rule.recognized}
    </span>
    <ArrowRight className="w-4 h-4 shrink-0 text-mid-gray" aria-hidden="true" />
    <span
      className="flex-1 min-w-0 truncate text-sm"
      data-testid={`${ROW_ID}-replacement`}
    >
      {rule.replacement}
    </span>
  </>
);

/** Inline edit of both fields, written to the draft as the user types. */
const RowEditor: React.FC<RuleRowProps> = (props) => {
  const handleKeyDown = useEditorKeys(props);
  const field = (name: RuleField) => (
    <RowField {...props} name={name} onKeyDown={handleKeyDown} />
  );
  return (
    <>
      {field("recognized")}
      <ArrowRight
        className="w-4 h-4 shrink-0 text-mid-gray"
        aria-hidden="true"
      />
      {field("replacement")}
    </>
  );
};

/**
 * Keys of the inline edit: Enter leaves it; Escape restores the row as it was when the edit
 * started, then leaves it.
 * @returns the key handler of both fields
 */
const useEditorKeys = ({
  rule,
  onEditing,
}: RuleRowProps): ((event: React.KeyboardEvent) => void) => {
  const before = useRef({
    recognized: rule.recognized,
    replacement: rule.replacement,
  });
  const edit = useReplacementEditor((state) => state.editRule);
  return (event) => {
    if (event.key === "Enter") onEditing(false);
    if (event.key !== "Escape") return;
    // Escape must not also close an enclosing dialog or leave the section.
    event.stopPropagation();
    edit(rule.key, "recognized", before.current.recognized);
    edit(rule.key, "replacement", before.current.replacement);
    onEditing(false);
  };
};

interface RowFieldProps extends RuleRowProps {
  name: RuleField;
  onKeyDown: (event: React.KeyboardEvent) => void;
}

const RowField: React.FC<RowFieldProps> = (props) => {
  const { t } = useTranslation();
  const { rule, name } = props;
  const edit = useReplacementEditor((state) => state.editRule);
  return (
    <Input
      type="text"
      variant="compact"
      className="flex-1 min-w-0"
      autoFocus={name === "recognized"}
      readOnly={props.locked}
      aria-label={t(`settings.replacements.rule.${name}`, {
        position: props.position,
      })}
      value={rule[name]}
      onChange={(event) => edit(rule.key, name, event.target.value)}
      onKeyDown={props.onKeyDown}
    />
  );
};

const RowActions: React.FC<RuleRowProps> = (props) => {
  const { t } = useTranslation();
  const recognized = props.rule.recognized;
  const editLabel = props.editing
    ? t("settings.replacements.rule.done", { recognized })
    : t("settings.replacements.rule.edit", { recognized });
  return (
    <div className="flex items-center gap-1 shrink-0">
      <button
        type="button"
        className={ICON_BUTTON}
        aria-label={editLabel}
        title={editLabel}
        aria-pressed={props.editing}
        disabled={props.locked}
        data-testid={`${ROW_ID}-edit`}
        onClick={() => props.onEditing(!props.editing)}
      >
        {props.editing ? <Check size={18} /> : <Pencil size={18} />}
      </button>
      <DeleteButton {...props} />
    </div>
  );
};

const DeleteButton: React.FC<RuleRowProps> = (props) => {
  const { t } = useTranslation();
  const label = t("settings.replacements.rule.delete", {
    recognized: props.rule.recognized,
  });
  return (
    <button
      type="button"
      ref={props.deleteRef}
      className={ICON_BUTTON}
      aria-label={label}
      title={label}
      disabled={props.locked}
      data-testid={`${ROW_ID}-delete`}
      onClick={props.onDelete}
    >
      <Trash2 size={18} />
    </button>
  );
};
