// One row of a shortcut list (004, 005): the warnings of a stored shortcut
// (conflict, refused by the system), its recorder chip and its remove button.
import React from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, X } from "lucide-react";
import type { ShortcutEdit } from "@/bindings";
import { ShortcutInput } from "./ShortcutInput";
import type { TranslatedMessage } from "./shortcut-list-model";

interface ShortcutRowProps {
  actionId: string;
  rowId: string;
  shortcut: string;
  canRemove: boolean;
  busy: boolean;
  conflict?: TranslatedMessage;
  /** The system refused this shortcut: it is stored but does nothing. */
  refusal?: TranslatedMessage;
  onEdit: (edit: ShortcutEdit) => Promise<void>;
}

interface RowWarningProps {
  message: TranslatedMessage;
  testId: string;
}

/** Warning icon of a row, explained by its accessible label and tooltip. */
const RowWarning: React.FC<RowWarningProps> = ({ message, testId }) => {
  const { t } = useTranslation();
  const text = t(message.key, message.params);
  return (
    <span
      role="img"
      className="text-yellow-500"
      data-testid={testId}
      aria-label={text}
      title={text}
    >
      <AlertTriangle size={14} />
    </span>
  );
};

interface RemoveButtonProps {
  testId: string;
  enabled: boolean;
  label: string;
  onRemove: () => void;
}

const RemoveButton: React.FC<RemoveButtonProps> = ({
  testId,
  enabled,
  label,
  onRemove,
}) => (
  <button
    type="button"
    className={`p-1 rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary ${enabled ? "hover:bg-logo-primary/30 cursor-pointer" : "opacity-50 cursor-not-allowed"}`}
    data-testid={testId}
    aria-label={label}
    aria-disabled={!enabled}
    title={label}
    onClick={() => {
      if (enabled) onRemove();
    }}
  >
    <X size={14} />
  </button>
);

/** Label of the remove button: remove, or why the last one stays. */
const useRemoveLabel = (canRemove: boolean): string => {
  const { t } = useTranslation();
  return canRemove
    ? t("settings.general.shortcut.list.remove")
    : t("settings.general.shortcut.list.removeLast");
};

/** One stored shortcut: warnings, recorder chip and remove button. */
export const ShortcutRow: React.FC<ShortcutRowProps> = (props) => {
  const { actionId, rowId, shortcut, canRemove, busy, onEdit } = props;
  const { conflict, refusal } = props;
  const { t } = useTranslation();
  const removeLabel = useRemoveLabel(canRemove);
  return (
    <div
      className="flex items-center gap-1"
      data-testid={rowId}
      data-key={shortcut}
    >
      {refusal && <RowWarning message={refusal} testId={`${rowId}-refused`} />}
      {conflict && (
        <RowWarning message={conflict} testId={`${rowId}-conflict`} />
      )}
      <ShortcutInput
        shortcutId={actionId}
        value={shortcut}
        disabled={busy}
        label={t("settings.general.shortcut.list.change")}
        testId={`${rowId}-recorder`}
        onCapture={(next) =>
          onEdit({ kind: "replace", previous: shortcut, shortcut: next })
        }
      />
      <RemoveButton
        testId={`${rowId}-remove`}
        enabled={canRemove && !busy}
        label={removeLabel}
        onRemove={() => void onEdit({ kind: "remove", shortcut })}
      />
    </div>
  );
};
