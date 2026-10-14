// Rows of a shortcut list: each stored shortcut with its warnings and remove
// control, the "Add shortcut" draft recorder, the Add and Reset controls,
// and the hint and messages under them.
import React from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, X } from "lucide-react";
import type { ShortcutEdit } from "@/bindings";
import { ResetButton } from "../ui/ResetButton";
import { ShortcutInput } from "./ShortcutInput";
import { ShortcutChipContext, shortcutChipClass } from "./shortcut-chip";
import { type AddControlState, recorderValidity } from "./shortcut-draft-model";
import {
  MAX_SHORTCUTS_PER_ACTION,
  type TranslatedMessage,
} from "./shortcut-list-model";
import type { ListHint, ShortcutTestIds } from "./shortcut-list-feedback";
import type { ShortcutDraft } from "./use-shortcut-list";

const LIST_KEY = "settings.general.shortcut.list";
const MESSAGE_CLASS = "text-xs max-w-xs text-right";

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

interface RowButtonProps {
  label: string;
  isEnabled: boolean;
  testId: string;
  onClick: () => void;
}

/** Small X button of a row; a disabled one stays focusable to explain why. */
const RowRemoveButton: React.FC<RowButtonProps> = ({
  label,
  isEnabled,
  testId,
  onClick,
}) => (
  <button
    type="button"
    className={`p-1 rounded-md ${isEnabled ? "hover:bg-logo-primary/30 cursor-pointer" : "opacity-50 cursor-not-allowed"}`}
    data-testid={testId}
    aria-label={label}
    aria-disabled={!isEnabled}
    title={label}
    onClick={() => {
      if (isEnabled) onClick();
    }}
  >
    <X size={14} />
  </button>
);

export interface ShortcutRowProps {
  actionId: string;
  shortcut: string;
  ids: ShortcutTestIds;
  /** False for an action that keeps a single shortcut (cancel). */
  hasRemove: boolean;
  canRemove: boolean;
  busy: boolean;
  conflict?: TranslatedMessage;
  /** The system refused this shortcut: it is stored but does nothing. */
  refusal?: TranslatedMessage;
  /** Non-zero: start recording this row ("Change"). */
  recordRequest: number;
  onEdit: (edit: ShortcutEdit) => Promise<unknown>;
}

/** One stored shortcut: record it again to replace it, or remove it. */
export const ShortcutRow: React.FC<ShortcutRowProps> = (props) => {
  const { t } = useTranslation();
  const { shortcut, ids, canRemove, busy, onEdit } = props;
  return (
    <div className="flex items-center gap-1">
      {props.refusal && (
        <RowWarning message={props.refusal} testId={ids.refused} />
      )}
      {props.conflict && (
        <RowWarning message={props.conflict} testId={ids.conflictIcon} />
      )}
      <ShortcutChipContext.Provider
        value={{ recordRequest: props.recordRequest }}
      >
        <ShortcutInput
          shortcutId={props.actionId}
          value={shortcut}
          disabled={busy}
          label={t(`${LIST_KEY}.change`)}
          testId={ids.shortcut}
          onCapture={async (next) => {
            await onEdit({
              kind: "replace",
              previous: shortcut,
              shortcut: next,
            });
          }}
        />
      </ShortcutChipContext.Provider>
      {props.hasRemove && (
        <RowRemoveButton
          label={t(`${LIST_KEY}.${canRemove ? "remove" : "removeLast"}`)}
          isEnabled={canRemove && !busy}
          testId={ids.remove}
          onClick={() => void onEdit({ kind: "remove", shortcut })}
        />
      )}
    </div>
  );
};

interface DraftRowProps {
  actionId: string;
  ids: ShortcutTestIds;
  draft: ShortcutDraft;
  busy: boolean;
}

// @spec AC-001: Adding records with the existing recorder — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/** The recorder opened by Add: empty, then valid or invalid once checked. */
export const DraftRow: React.FC<DraftRowProps> = ({
  actionId,
  ids,
  draft,
  busy,
}) => {
  const { t } = useTranslation();
  const { state } = draft;
  if (state.phase === "closed") return null;
  const control = {
    recordRequest: draft.recordRequest,
    validity: recorderValidity(state),
    onRecordingChange: (isRecording: boolean) => {
      if (!isRecording) draft.stop();
    },
  };
  return (
    <div className="flex items-center gap-1" data-phase={state.phase}>
      <ShortcutChipContext.Provider value={control}>
        <ShortcutInput
          shortcutId={actionId}
          value={state.shortcut}
          disabled={busy || state.phase === "committing"}
          label={t(`${LIST_KEY}.change`)}
          testId={ids.recording}
          onCapture={draft.capture}
        />
      </ShortcutChipContext.Provider>
      {state.shortcut !== null && (
        <RowRemoveButton
          label={t(`${LIST_KEY}.discard`)}
          isEnabled={state.phase !== "committing"}
          testId={ids.discard}
          onClick={draft.discard}
        />
      )}
    </div>
  );
};

interface AddRowProps {
  ids: ShortcutTestIds;
  /** Null for an action that keeps a single shortcut (no Add). */
  addState: AddControlState | null;
  isFull: boolean;
  onAdd: () => void;
  onReset: () => void;
  resetDisabled: boolean;
}

/** "Add shortcut" (opens the draft, then adds it) and Reset. */
export const AddRow: React.FC<AddRowProps> = (props) => {
  const { t } = useTranslation();
  const { addState, ids } = props;
  const limitLabel = t(`${LIST_KEY}.addLimit`, {
    limit: MAX_SHORTCUTS_PER_ACTION,
  });
  return (
    <div className="flex items-center gap-1">
      {addState !== null && (
        <button
          type="button"
          data-testid={ids.add}
          data-state={addState}
          aria-disabled={addState !== "enabled"}
          aria-busy={addState === "loading"}
          title={props.isFull ? limitLabel : undefined}
          className={shortcutChipClass({
            isRecording: false,
            disabled: addState !== "enabled",
          })}
          onClick={() => {
            if (addState === "enabled") props.onAdd();
          }}
        >
          {t(`${LIST_KEY}.add`)}
        </button>
      )}
      <ResetButton
        onClick={props.onReset}
        disabled={props.resetDisabled}
        ariaLabel={t(`${LIST_KEY}.reset`)}
        testId={ids.reset}
      />
    </div>
  );
};

interface ListHintTextProps {
  hint: ListHint;
  count: number;
  ids: ShortcutTestIds;
}

/** Why Add or Remove is unavailable: the limit, or the last shortcut. */
export const ListHintText: React.FC<ListHintTextProps> = ({
  hint,
  count,
  ids,
}) => {
  const { t } = useTranslation();
  if (hint === null) return null;
  const isLimit = hint === "limit";
  return (
    <p
      className={`${MESSAGE_CLASS} text-mid-gray`}
      data-testid={isLimit ? ids.limit : ids.last}
    >
      {isLimit
        ? t(`${LIST_KEY}.limitHint`, {
            count,
            limit: MAX_SHORTCUTS_PER_ACTION,
          })
        : t(`${LIST_KEY}.lastHint`)}
    </p>
  );
};

interface ListMessagesProps {
  ids: ShortcutTestIds;
  conflictsFailed: boolean;
  error: TranslatedMessage | null;
}

/** Refusal of the last edit or draft, and a failed conflict check. */
export const ListMessages: React.FC<ListMessagesProps> = ({
  ids,
  conflictsFailed,
  error,
}) => {
  const { t } = useTranslation();
  return (
    <>
      {conflictsFailed && (
        <p
          role="status"
          className={`${MESSAGE_CLASS} text-yellow-600`}
          data-testid={ids.conflictsUnavailable}
        >
          {t(`${LIST_KEY}.errors.conflictsUnavailable`)}
        </p>
      )}
      {error && (
        <p
          role="alert"
          className={`${MESSAGE_CLASS} text-red-500`}
          data-testid={ids.error}
        >
          {t(error.key, error.params)}
        </p>
      )}
    </>
  );
};
