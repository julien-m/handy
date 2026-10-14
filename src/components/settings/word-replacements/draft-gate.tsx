/**
 * Guard of the editor's own transitions (AC-001): switching between the list and the text, or
 * importing a file, while the current view holds unsaved edits first asks to save (or apply) or
 * to discard them. While a Save or an Apply is in flight the guard takes no transition and its
 * dialog cannot be closed, cancelled or discarded (rules in `draft-gate-model.ts`).
 */
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../../ui/Dialog";
import { Button } from "../../ui/Button";
import { canSave, useReplacementEditor } from "./editor-store";
import { canApplyText, useReplacementText } from "./text-store";
import {
  discardEditor,
  isCommitInFlight,
  isEditorDirty,
  useCommitInFlight,
} from "./use-editor-lifecycle";
import { createDraftGate, type PendingTransition } from "./draft-gate-model";

/** Guarded transition and the dialog element to render with it. */
export interface DraftGate {
  readonly gate: (action: () => void) => void;
  readonly dialog: React.ReactNode;
}

/** Commit the current view's edits: Save for the list, Apply for the text. */
const commitCurrentView = (): Promise<boolean> =>
  useReplacementEditor.getState().view === "list"
    ? useReplacementEditor.getState().save()
    : useReplacementText.getState().apply();

export const useDraftGate = (): DraftGate => {
  const [pending, setPending] = useState<PendingTransition | null>(null);
  const controller = useMemo(
    () =>
      createDraftGate({
        isDirty: isEditorDirty,
        isBusy: isCommitInFlight,
        discard: discardEditor,
        commit: commitCurrentView,
        onChange: setPending,
      }),
    [],
  );
  const dialog = (
    <SaveOrDiscardDialog
      open={pending !== null}
      onCancel={controller.cancel}
      onDiscard={controller.discard}
      onSave={controller.save}
    />
  );
  return { gate: controller.request, dialog };
};

interface SaveOrDiscardDialogProps {
  open: boolean;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => Promise<void>;
}

/** Whether the current view (list or text) can be saved or applied right now. */
const useCurrentViewSavable = (): boolean => {
  const listSavable = useReplacementEditor(
    (state) => state.view === "list" && canSave(state),
  );
  const textSavable = useReplacementText(canApplyText);
  const view = useReplacementEditor((state) => state.view);
  return view === "list" ? listSavable : textSavable;
};

const SaveOrDiscardDialog: React.FC<SaveOrDiscardDialogProps> = (props) => {
  const { t } = useTranslation();
  const savable = useCurrentViewSavable();
  const busy = useCommitInFlight();
  return (
    <Dialog
      open={props.open}
      title={t("settings.replacements.switchDialog.title")}
      closeLabel={t("settings.replacements.switchDialog.cancel")}
      dismissible={!busy}
      onOpenChange={(open) => {
        if (!open) props.onCancel();
      }}
      footer={
        <SaveOrDiscardFooter
          onCancel={props.onCancel}
          onDiscard={props.onDiscard}
          onSave={props.onSave}
          savable={savable}
          busy={busy}
        />
      }
    >
      <p className="text-sm">{t("settings.replacements.switchDialog.body")}</p>
    </Dialog>
  );
};

/** Classes of a control made inactive with aria-disabled (the Button styles only `disabled`). */
const INACTIVE_CLASSES =
  "aria-disabled:opacity-50 aria-disabled:cursor-not-allowed";

/** Props of one footer button that is inactive (aria-disabled) while the commit runs. */
interface FooterButtonProps {
  variant: "primary" | "secondary";
  busy: boolean;
  disabled?: boolean;
  label: string;
  onPress: () => void;
}

/**
 * A footer button that stays focusable but inactive (aria-disabled) while the commit runs, so the
 * focus is not lost from the dialog.
 */
const FooterButton: React.FC<FooterButtonProps> = (props) => (
  <Button
    variant={props.variant}
    className={INACTIVE_CLASSES}
    disabled={props.disabled}
    aria-disabled={props.busy}
    aria-busy={props.variant === "primary" ? props.busy : undefined}
    onClick={() => {
      if (!props.busy) props.onPress();
    }}
  >
    {props.label}
  </Button>
);

/** Cancel, Discard and Save, all inactive while the commit runs. */
const SaveOrDiscardFooter: React.FC<
  Omit<SaveOrDiscardDialogProps, "open"> & { savable: boolean; busy: boolean }
> = ({ onCancel, onDiscard, onSave, savable, busy }) => {
  const { t } = useTranslation();
  const label = (key: string) => t(`settings.replacements.switchDialog.${key}`);
  return (
    <>
      <FooterButton
        variant="secondary"
        busy={busy}
        label={label("cancel")}
        onPress={onCancel}
      />
      <FooterButton
        variant="secondary"
        busy={busy}
        label={label("discard")}
        onPress={onDiscard}
      />
      <FooterButton
        variant="primary"
        busy={busy}
        disabled={!savable && !busy}
        label={label("save")}
        onPress={() => void onSave()}
      />
    </>
  );
};
