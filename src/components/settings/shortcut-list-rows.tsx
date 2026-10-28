// Rows of a shortcut list: each stored shortcut with its warnings and remove
// control, the "Add shortcut" draft recorder, the Add (or, during a draft,
// Save and Discard) and Reset controls, and the hint and messages under them.
import React, { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, X } from "lucide-react";
import type { ShortcutEdit } from "@/bindings";
import { Button } from "../ui/Button";
import { ResetButton } from "../ui/ResetButton";
import { ShortcutInput } from "./ShortcutInput";
import {
  type ShortcutChipControl,
  ShortcutChipContext,
  shortcutChipClass,
} from "./shortcut-chip";
import {
  type AddControlState,
  type ListControls,
  recorderValidity,
} from "./shortcut-draft-model";
import {
  MAX_SHORTCUTS_PER_ACTION,
  type TranslatedMessage,
} from "./shortcut-list-model";
import {
  type ListHint,
  type ShortcutTestIds,
  shortcutInstanceId,
} from "./shortcut-list-feedback";
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
  /** Non-zero: start recording this row ("Change"), once. */
  recordRequest: number;
  onRequestHandled: (request: number) => void;
  onEdit: (edit: ShortcutEdit) => Promise<unknown>;
}

/** A stored row's chip: started once by "Change", never on its own. */
const rowControl = (props: ShortcutRowProps): ShortcutChipControl => ({
  recordRequest: props.recordRequest,
  onRequestHandled: props.onRequestHandled,
});

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
      <ShortcutChipContext.Provider value={rowControl(props)}>
        <ShortcutInput
          shortcutId={props.actionId}
          value={shortcut}
          disabled={busy}
          label={t(`${LIST_KEY}.change`)}
          testId={ids.shortcut}
          instanceId={shortcutInstanceId(shortcut)}
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

/** What the draft recorder is told and reports back. */
const draftControl = (draft: ShortcutDraft): ShortcutChipControl => ({
  recordRequest: draft.recordRequest,
  onRequestHandled: draft.recorder.requestHandled,
  validity: recorderValidity(draft.state),
  isDraft: true,
  onRecordingChange: (isRecording) =>
    isRecording ? draft.recorder.started() : draft.stop(),
  onStartFailed: draft.recorder.startFailed,
});

// @spec AC-001: Adding records with the existing recorder — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-001
/**
 * The recorder opened by Add: empty, then valid or invalid once checked. Its
 * dashed chip and the "Not saved" label set it apart from a stored shortcut
 * until Save adds it (native session finding N8).
 */
export const DraftRow: React.FC<DraftRowProps> = (props) => {
  const { t } = useTranslation();
  const { ids, draft } = props;
  const { state } = draft;
  if (state.phase === "closed") return null;
  return (
    <div className="flex items-center gap-2" data-phase={state.phase}>
      {state.shortcut !== null && (
        <span
          className="text-xs italic text-mid-gray"
          data-testid={ids.unsaved}
        >
          {t(`${LIST_KEY}.unsaved`)}
        </span>
      )}
      <ShortcutChipContext.Provider value={draftControl(draft)}>
        <ShortcutInput
          shortcutId={props.actionId}
          value={state.shortcut}
          disabled={props.busy || state.phase === "committing"}
          label={t(`${LIST_KEY}.change`)}
          testId={ids.recording}
          onCapture={draft.capture}
        />
      </ShortcutChipContext.Provider>
    </div>
  );
};

interface AddRowProps {
  ids: ShortcutTestIds;
  /** Null for an action that keeps a single shortcut (no Add). */
  controls: ListControls | null;
  isFull: boolean;
  onAdd: () => void;
  onSave: () => void;
  onDiscard: () => void;
  onReset: () => void;
  resetDisabled: boolean;
}

interface ListButtonProps {
  state: AddControlState;
  testId: string;
  title?: string;
  onActivate: () => void;
  children: React.ReactNode;
}

/** "Add shortcut" in Handy's chip look; a disabled one stays focusable to explain why. */
const AddButton = React.forwardRef<HTMLButtonElement, ListButtonProps>(
  function AddButton(props, ref) {
    return (
      <button
        ref={ref}
        type="button"
        data-testid={props.testId}
        data-state={props.state}
        aria-disabled={props.state !== "enabled"}
        aria-busy={props.state === "loading"}
        title={props.title}
        className={shortcutChipClass({
          isRecording: false,
          disabled: props.state !== "enabled",
        })}
        onClick={() => {
          if (props.state === "enabled") props.onActivate();
        }}
      >
        {props.children}
      </button>
    );
  },
);

/**
 * Gives the focus back to "Add shortcut" when Save or Discard, which had it,
 * left with the draft; a focus the user moved elsewhere is left alone.
 * @param mode Controls shown under the list.
 * @param add The Add button.
 */
const useFocusAddAfterDraft = (
  mode: ListControls["mode"] | undefined,
  add: React.RefObject<HTMLButtonElement | null>,
): void => {
  const previous = useRef(mode);
  useEffect(() => {
    const closedDraft = previous.current === "draft" && mode === "add";
    previous.current = mode;
    const isFocusLost =
      document.activeElement === null ||
      document.activeElement === document.body;
    if (closedDraft && isFocusLost) add.current?.focus({ preventScroll: true });
  }, [mode, add]);
};

interface DraftButtonsProps {
  ids: ShortcutTestIds;
  controls: ListControls;
  onSave: () => void;
  onDiscard: () => void;
}

/** Save (only a valid draft) and Discard, shown instead of Add during a draft. */
const DraftButtons: React.FC<DraftButtonsProps> = (props) => {
  const { t } = useTranslation();
  const { controls, ids } = props;
  const canSave = controls.submit === "enabled";
  return (
    <>
      <Button
        variant="secondary"
        size="sm"
        data-testid={ids.discard}
        aria-disabled={!controls.canDiscard}
        title={t(`${LIST_KEY}.discard`)}
        className={controls.canDiscard ? "" : "opacity-50 cursor-not-allowed"}
        onClick={() => {
          if (controls.canDiscard) props.onDiscard();
        }}
      >
        {t(`${LIST_KEY}.discardDraft`)}
      </Button>
      <Button
        variant="primary"
        size="sm"
        data-testid={ids.add}
        data-state={controls.submit}
        aria-disabled={!canSave}
        aria-busy={controls.submit === "loading"}
        className={canSave ? "" : "opacity-50 cursor-not-allowed"}
        onClick={() => {
          if (canSave) props.onSave();
        }}
      >
        {t(`${LIST_KEY}.save`)}
      </Button>
    </>
  );
};

/**
 * "Add shortcut" and Reset; during a draft, Save and Discard replace Add and
 * Reset waits (native session finding N8).
 */
export const AddRow: React.FC<AddRowProps> = (props) => {
  const { t } = useTranslation();
  const { controls, ids } = props;
  const addButton = useRef<HTMLButtonElement | null>(null);
  useFocusAddAfterDraft(controls?.mode, addButton);
  const limitLabel = t(`${LIST_KEY}.addLimit`, {
    limit: MAX_SHORTCUTS_PER_ACTION,
  });
  return (
    <div className="flex items-center gap-1">
      {controls?.mode === "draft" && (
        <DraftButtons
          ids={ids}
          controls={controls}
          onSave={props.onSave}
          onDiscard={props.onDiscard}
        />
      )}
      {controls?.mode === "add" && (
        <AddButton
          ref={addButton}
          state={controls.submit}
          testId={ids.add}
          title={props.isFull ? limitLabel : undefined}
          onActivate={props.onAdd}
        >
          {t(`${LIST_KEY}.add`)}
        </AddButton>
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
