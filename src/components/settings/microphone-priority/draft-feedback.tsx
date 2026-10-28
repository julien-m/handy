// Draft feedback of the microphone priority editor (feature 001, AC-007): the "settings changed
// elsewhere" banner with Reload, and the compact save bar with Discard and Save, kept at the bottom
// of the visible content while the draft differs from the saved order (native session findings N2
// and N3). On a conflict the bar offers Reload in place of Save. Save and Reload are made inactive
// with aria-disabled rather than disabled, so a control that has focus keeps it while a write runs
// and when Save turns into Retry or Reload (SC-003).
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

/** Props of the save bar. */
export interface DraftBarProps {
  visible: boolean;
  phase: SavePhase;
  /** The order changed elsewhere: the bar offers Reload instead of Save. */
  conflict: boolean;
  onReload: () => void;
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

/** What the primary control of the bar does, says and shows (contract state). */
interface PrimaryControl {
  label: string;
  state: "loading" | "disabled" | "enabled";
  testId: string;
  isEnabled: boolean;
  run: () => void;
}

/**
 * The bar's primary control: Reload during a conflict (the draft cannot be saved over a newer
 * order), else Save, Retry after a failed write. A write in flight keeps it Saving… until it
 * settles, even if a conflict is raised meanwhile, so the focused control does not change role
 * under the user.
 * @param props bar props
 * @param screen contract screen shown
 * @returns its label key, contract state, test id and action
 */
const primaryControl = (
  props: DraftBarProps,
  screen: string,
): PrimaryControl => {
  if (props.phase === "saving") {
    return {
      label: "draft.saving",
      state: "loading",
      testId: `${screen}-save`,
      isEnabled: false,
      run: props.onSave,
    };
  }
  if (props.conflict) {
    return {
      label: "conflict.reload",
      state: "enabled",
      testId: `${screen}-draft-reload`,
      isEnabled: true,
      run: props.onReload,
    };
  }
  return {
    label: props.phase === "failed" ? "retry" : "draft.save",
    state: props.saveEnabled ? "enabled" : "disabled",
    testId: `${screen}-save`,
    isEnabled: props.saveEnabled,
    run: props.onSave,
  };
};

/**
 * Discard and the primary control. The primary stays the same button when Save turns into Retry
 * or Reload, so a focused control keeps the focus; it cannot dispatch again while a write runs.
 */
const DraftActions: React.FC<DraftBarProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  const primary = primaryControl(props, screen);
  const saving = props.phase === "saving";
  return (
    <div className="flex items-center gap-2 shrink-0">
      <Button
        variant="secondary"
        size="sm"
        onClick={props.onDiscard}
        disabled={saving}
        data-testid={`${screen}-discard`}
      >
        {t(`${PRIORITY_KEY_ROOT}.draft.discard`)}
      </Button>
      <Button
        variant="primary"
        size="sm"
        className={INACTIVE_CLASSES}
        onClick={() => {
          if (primary.isEnabled) primary.run();
        }}
        aria-disabled={!primary.isEnabled}
        aria-busy={saving}
        data-state={primary.state}
        data-testid={primary.testId}
      >
        {t(`${PRIORITY_KEY_ROOT}.${primary.label}`)}
      </Button>
    </div>
  );
};

/**
 * Compact save bar, shown only while the draft differs from the saved order and kept at the
 * bottom of the visible settings content (sticky in the scrolling area). It holds Discard and Save
 * (Reload on a conflict); a refusal of the last write or an invalid draft is the only text it shows.
 */
export const DraftBar: React.FC<DraftBarProps> = (props) => {
  const { t } = useTranslation();
  const screen = useMicrophoneScreen();
  if (!props.visible) return null;
  return (
    <div
      className="sticky bottom-2 z-10 ms-auto flex w-fit max-w-full items-center gap-3 px-2 py-2 bg-background border border-mid-gray/20 rounded-lg shadow-lg"
      data-testid={`${screen}-draft`}
      data-state={props.phase === "failed" ? "error" : props.phase}
      data-conflict={props.conflict || undefined}
    >
      {props.error && (
        <p
          role="alert"
          className="ps-2 text-xs text-red-400"
          data-testid={`${screen}-draft-note`}
        >
          {t(props.error.key, props.error.params)}
        </p>
      )}
      <DraftActions {...props} />
    </div>
  );
};
