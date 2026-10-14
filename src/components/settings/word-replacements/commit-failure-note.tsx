/**
 * Why a Save, Apply or import did not commit (visual state "error"): the translated typed
 * refusal, a reminder that the saved rules are unchanged, and Reload for a revision conflict.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import type { CommitFailure } from "./preview-model";

interface CommitFailureNoteProps {
  failure: CommitFailure;
  onReload: () => void;
}

export const CommitFailureNote: React.FC<CommitFailureNoteProps> = ({
  failure,
  onReload,
}) => {
  const { t } = useTranslation();
  const message = failure.conflict
    ? t("settings.replacements.commit.conflict")
    : t(failure.key);
  // A refusal changed nothing; an uncertain outcome must not claim that.
  const unchanged = failure.uncertain
    ? ""
    : ` ${t("settings.replacements.commit.unchanged")}`;
  return (
    <div className="space-y-2" role="alert">
      <Alert variant="error">{`${message}${unchanged}`}</Alert>
      {failure.conflict && (
        <Button variant="secondary" size="sm" onClick={onReload}>
          {t("settings.replacements.commit.reload")}
        </Button>
      )}
    </div>
  );
};
