/**
 * Why a Save, Apply or import did not commit (visual state "error"): the translated typed
 * refusal, a reminder that the saved rules are unchanged, and Reload for a revision conflict.
 * When Reload replaces unsaved edits (the list), the note says so before the user chooses it.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import type { CommitFailure } from "./preview-model";

interface CommitFailureNoteProps {
  failure: CommitFailure;
  onReload: () => void;
  /** Reload drops the unsaved edits (list draft) instead of keeping them (text, import). */
  discardsOnReload?: boolean;
  /** Test id of the note (`<screen>-commit-failure`); its message and Reload derive from it. */
  testId: string;
}

export const CommitFailureNote: React.FC<CommitFailureNoteProps> = ({
  failure,
  onReload,
  discardsOnReload = false,
  testId,
}) => {
  const { t } = useTranslation();
  const discards =
    failure.conflict && discardsOnReload
      ? ` ${t("settings.replacements.commit.reloadDiscards")}`
      : "";
  const message = failure.conflict
    ? `${t("settings.replacements.commit.conflict")}${discards}`
    : t(failure.key);
  // A refusal changed nothing; an uncertain outcome must not claim that.
  const unchanged = failure.uncertain
    ? ""
    : ` ${t("settings.replacements.commit.unchanged")}`;
  return (
    <div className="space-y-2" role="alert" data-testid={testId}>
      <Alert variant="error">
        <span
          data-testid={`${testId}-message`}
        >{`${message}${unchanged}`}</span>
      </Alert>
      {failure.conflict && (
        <Button
          variant="secondary"
          size="sm"
          onClick={onReload}
          data-testid={`${testId}-reload`}
        >
          {t("settings.replacements.commit.reload")}
        </Button>
      )}
    </div>
  );
};
