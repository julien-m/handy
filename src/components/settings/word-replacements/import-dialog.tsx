/**
 * Import of a rule file (screen `replacement-settings-import`): the refused-file alert, and the
 * dialog with the file summary, the merge/replace mode, the preview counts and Apply import,
 * which commits at the preview's revision (AC-005). Nothing is saved before Apply import.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Dialog } from "../../ui/Dialog";
import { Button } from "../../ui/Button";
import { Alert } from "../../ui/Alert";
import { lineCount } from "./preview-model";
import { SCREEN_IDS } from "./rules-model";
import {
  canApplyImport,
  reloadImportPreview,
  useReplacementFiles,
} from "./file-store";
import { ImportMode, ImportSummary } from "./import-summary";
import { CommitFailureNote } from "./commit-failure-note";

const DIALOG_ID = `${SCREEN_IDS.import}-dialog`;

export const ImportScreen: React.FC = () => {
  const refusal = useReplacementFiles((state) => state.refusal);
  const open = useReplacementFiles((state) => state.file !== null);
  if (refusal === null && !open) return null;
  return (
    <div data-testid={`${SCREEN_IDS.import}-screen`}>
      <ImportRefusalAlert />
      <ImportDialog />
    </div>
  );
};

const ImportRefusalAlert: React.FC = () => {
  const { t } = useTranslation();
  const refusal = useReplacementFiles((state) => state.refusal);
  if (refusal === null) return null;
  const name = refusal.fileName;
  return (
    <div
      className="space-y-2"
      role="alert"
      data-testid={`${SCREEN_IDS.import}-unreadable`}
    >
      <Alert variant="error">
        <span
          className="block font-medium"
          data-testid={`${SCREEN_IDS.import}-unreadable-title`}
        >
          {t(`settings.replacements.importRefusal.${refusal.kind}Title`, {
            name,
          })}
        </span>
        <span className="block">
          {t(`settings.replacements.importRefusal.${refusal.kind}Body`)}
        </span>
      </Alert>
      <DismissRefusalButton />
    </div>
  );
};

const DismissRefusalButton: React.FC = () => {
  const { t } = useTranslation();
  return (
    <Button
      variant="secondary"
      size="sm"
      onClick={() => useReplacementFiles.getState().dismissRefusal()}
    >
      {t("settings.replacements.importRefusal.dismiss")}
    </Button>
  );
};

const ImportDialog: React.FC = () => {
  const { t } = useTranslation();
  const file = useReplacementFiles((state) => state.file);
  const busy = useReplacementFiles((state) => state.phase === "applying");
  return (
    <Dialog
      open={file !== null}
      title={t("settings.replacements.importDialog.title")}
      closeLabel={t("settings.replacements.importDialog.close")}
      dismissible={!busy}
      onOpenChange={(open) => {
        if (!open) useReplacementFiles.getState().closeImport();
      }}
      footer={<ImportActions />}
      testId={DIALOG_ID}
    >
      <div className="space-y-4">
        {file && (
          <FileSummary name={file.file_name} lines={lineCount(file.text)} />
        )}
        <ImportMode />
        <ImportSummary />
        <ImportFailureNote />
      </div>
    </Dialog>
  );
};

/** The failed Apply import, with a reload of the preview; nothing when the last commit did not fail. */
const ImportFailureNote: React.FC = () => {
  const failure = useReplacementFiles((state) => state.failure);
  if (!failure) return null;
  return (
    <CommitFailureNote
      failure={failure}
      onReload={() => void reloadImportPreview()}
    />
  );
};

const FileSummary: React.FC<{ name: string; lines: number }> = ({
  name,
  lines,
}) => {
  const { t } = useTranslation();
  return (
    <p className="text-sm text-mid-gray" data-testid={`${DIALOG_ID}-file`}>
      {lines === 1
        ? t("settings.replacements.importDialog.fileOne", { name })
        : t("settings.replacements.importDialog.fileMany", {
            name,
            count: lines,
          })}
    </p>
  );
};

const ImportActions: React.FC = () => {
  const { t } = useTranslation();
  const enabled = useReplacementFiles(canApplyImport);
  const applying = useReplacementFiles((state) => state.phase === "applying");
  const apply = async () => {
    if (await useReplacementFiles.getState().applyImport()) {
      toast.success(t("settings.replacements.importDialog.imported"));
    }
  };
  return (
    <>
      <CancelImportButton disabled={applying} />
      <Button
        variant="primary"
        disabled={!enabled}
        aria-busy={applying}
        data-testid={`${DIALOG_ID}-apply`}
        onClick={() => void apply()}
      >
        {applying
          ? t("settings.replacements.importDialog.applying")
          : t("settings.replacements.importDialog.apply")}
      </Button>
    </>
  );
};

const CancelImportButton: React.FC<{ disabled: boolean }> = ({ disabled }) => {
  const { t } = useTranslation();
  return (
    <Button
      variant="secondary"
      disabled={disabled}
      data-testid={`${DIALOG_ID}-cancel`}
      onClick={() => useReplacementFiles.getState().closeImport()}
    >
      {t("settings.replacements.importDialog.cancel")}
    </Button>
  );
};
