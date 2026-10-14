/**
 * Guard of the editor's own transitions (AC-001): switching between the list and the text, or
 * importing a file, while the current view holds unsaved edits first asks to save (or apply) or
 * to discard them.
 */
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "../../ui/Dialog";
import { Button } from "../../ui/Button";
import { canSave, useReplacementEditor } from "./editor-store";
import { canApplyText, useReplacementText } from "./text-store";
import { discardEditor, isEditorDirty } from "./use-editor-lifecycle";

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
  const [pending, setPending] = useState<{ action: () => void } | null>(null);
  const gate = (action: () => void) => {
    if (isEditorDirty()) setPending({ action });
    else action();
  };
  const finish = () => {
    pending?.action();
    setPending(null);
  };
  const dialog = (
    <SaveOrDiscardDialog
      open={pending !== null}
      onCancel={() => setPending(null)}
      onDiscard={() => {
        discardEditor();
        finish();
      }}
      onSave={async () => {
        if (await commitCurrentView()) finish();
        else setPending(null);
      }}
    />
  );
  return { gate, dialog };
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
  return (
    <Dialog
      open={props.open}
      title={t("settings.replacements.switchDialog.title")}
      closeLabel={t("settings.replacements.switchDialog.cancel")}
      onOpenChange={(open) => {
        if (!open) props.onCancel();
      }}
      footer={
        <SaveOrDiscardFooter
          onCancel={props.onCancel}
          onDiscard={props.onDiscard}
          onSave={props.onSave}
          savable={savable}
        />
      }
    >
      <p className="text-sm">{t("settings.replacements.switchDialog.body")}</p>
    </Dialog>
  );
};

const SaveOrDiscardFooter: React.FC<
  Omit<SaveOrDiscardDialogProps, "open"> & { savable: boolean }
> = ({ onCancel, onDiscard, onSave, savable }) => {
  const { t } = useTranslation();
  return (
    <>
      <Button variant="secondary" onClick={onCancel}>
        {t("settings.replacements.switchDialog.cancel")}
      </Button>
      <Button variant="secondary" onClick={onDiscard}>
        {t("settings.replacements.switchDialog.discard")}
      </Button>
      <Button
        variant="primary"
        disabled={!savable}
        onClick={() => void onSave()}
      >
        {t("settings.replacements.switchDialog.save")}
      </Button>
    </>
  );
};
