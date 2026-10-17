// Draft feedback of the microphone priority editor (feature 001, AC-007): the "settings changed
// elsewhere" banner with Reload, and the Unsaved changes bar with Discard and Save. Save and Reload
// are made inactive with aria-disabled rather than disabled, so a control that has focus keeps it
// while a write runs and when Save turns into Retry (SC-003).
import React from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle } from "lucide-react";
import { Button } from "../../ui/Button";
import {
  PRIORITY_KEY_ROOT,
  type SavePhase,
  type TranslatedMessage,
} from "./draft-state";
import { useMicrophoneScreen } from "./screen-id";

/** Props of the conflict banner. */
export interface ConflictAlertProps {
  visible: boolean;
  /** A write is in flight: Reload is inactive until it settles. */
  busy: boolean;
  onReload: () => void;
}

/** Props of the Unsaved changes bar. */
export interface DraftBarProps {
  visible: boolean;
  phase: SavePhase;
  /** Refusal of the last write, or why the draft cannot be saved. */
  error: TranslatedMessage | null;
  saveEnabled: boolean;
  onDiscard: () => void;
  onSave: () => void;
}

/** Classes of a control made inactive with aria-disabled (the Button styles only `disabled`). */
const INACTIVE_CLASSES =
  "aria-disabled:opacity-50 aria-disabled:cursor-not-allowed";

/** Reload of the conflict banner; inactive (still focusable) while a write is in flight. */
const ReloadButton: React.FC<Omit<ConflictAlertProps, "visible">> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  return (
    <Button
      variant="warning"
      className={INACTIVE_CLASSES}
      onClick={() => {
        if (!props.busy) props.onReload();
      }}
      aria-disabled={props.busy}
      data-testid={`${screen}-reload`}
    >
      {t(`${PRIORITY_KEY_ROOT}.conflict.reload`)}
    </Button>
  );
};

/** The committed order changed elsewhere: the draft is kept, and Save is inactive, until Reload or Discard. */
export const ConflictAlert: React.FC<ConflictAlertProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  if (!props.visible) return null;
  return (
    <div
      role="alert"
      className="flex items-start gap-3 p-4 rounded-lg bg-yellow-500/10"
      data-testid={`${screen}-conflict`}
    >
      <AlertTriangle className="w-5 h-5 shrink-0 mt-0.5 text-yellow-500" />
      <div className="flex-1 text-sm text-yellow-400">
        <p className="font-medium" data-testid={`${screen}-conflict-title`}>
          {t(`${PRIORITY_KEY_ROOT}.conflict.title`)}
        </p>
        <p>{t(`${PRIORITY_KEY_ROOT}.conflict.body`)}</p>
      </div>
      <ReloadButton busy={props.busy} onReload={props.onReload} />
    </div>
  );
};

/**
 * Label and visual state of the Save control.
 * @param props bar props
 * @returns the label key and the contract state (loading, disabled or enabled)
 */
const saveControl = (props: DraftBarProps) => {
  if (props.phase === "saving")
    return { label: "draft.saving", state: "loading" };
  const label = props.phase === "failed" ? "retry" : "draft.save";
  return { label, state: props.saveEnabled ? "enabled" : "disabled" };
};

/** Discard and Save; Save is busy (and cannot dispatch again) while a write is in flight. */
const DraftActions: React.FC<DraftBarProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  const save = saveControl(props);
  const saving = props.phase === "saving";
  return (
    <div className="flex items-center gap-2 shrink-0">
      <Button
        variant="secondary"
        onClick={props.onDiscard}
        disabled={saving}
        data-testid={`${screen}-discard`}
      >
        {t(`${PRIORITY_KEY_ROOT}.draft.discard`)}
      </Button>
      <Button
        variant="primary"
        className={INACTIVE_CLASSES}
        onClick={() => {
          if (props.saveEnabled) props.onSave();
        }}
        aria-disabled={!props.saveEnabled}
        aria-busy={saving}
        data-state={save.state}
        data-testid={`${screen}-save`}
      >
        {t(`${PRIORITY_KEY_ROOT}.${save.label}`)}
      </Button>
    </div>
  );
};

/** Unsaved changes: the next recording keeps the saved order until Save succeeds. */
export const DraftBar: React.FC<DraftBarProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  if (!props.visible) return null;
  return (
    <div
      className="flex items-center justify-between gap-3 px-4 py-3 bg-background border border-mid-gray/20 rounded-lg"
      data-testid={`${screen}-draft`}
      data-state={props.phase === "failed" ? "error" : props.phase}
    >
      <p
        role={props.error ? "alert" : undefined}
        className={`text-sm ${props.error ? "text-red-400" : "text-mid-gray"}`}
        data-testid={`${screen}-draft-note`}
      >
        {props.error
          ? t(props.error.key, props.error.params)
          : t(`${PRIORITY_KEY_ROOT}.draft.note`)}
      </p>
      <DraftActions {...props} />
    </div>
  );
};
