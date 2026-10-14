/**
 * Notices of the vocabulary editor that are not part of a mockup node: a failed commit with its
 * truthful outcome and Retry, and the saved list while it loads or after its read failed.
 */
import React from "react";
import { useTranslation } from "react-i18next";
import type { SettingsCommandError } from "@/bindings";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import {
  commitErrorKey,
  failureNotice,
  FOCUS_RING,
  type VocabularyScreen,
} from "./vocabulary-model";

/** Secondary Retry with the visible keyboard focus its variant lacks. */
export const RetryButton: React.FC<{ onRetry: () => void }> = ({ onRetry }) => {
  const { t } = useTranslation();
  return (
    <Button
      variant="secondary"
      size="sm"
      className={FOCUS_RING}
      onClick={onRetry}
    >
      {t("settings.vocabulary.retry")}
    </Button>
  );
};

interface CommitFailureProps {
  screen: VocabularyScreen;
  error: SettingsCommandError | null;
  /** Id of the message, referenced by the term input. */
  errorId: string;
  onRetry: () => void;
}

/**
 * A failed commit: its message, "unchanged" only when no outcome is reported, and Retry when
 * retrying can succeed.
 * @param props screen prefix, typed error (null when untyped), message id and retry callback
 */
export const CommitFailure: React.FC<CommitFailureProps> = (props) => {
  const { screen, error, errorId } = props;
  const { t } = useTranslation();
  const notice = failureNotice(error);
  return (
    <div
      className="flex flex-col items-start gap-2"
      data-testid={`${screen}-error`}
      role="alert"
    >
      <Alert variant="error">
        <span id={errorId} className="block">
          {t(commitErrorKey(error))}
        </span>
        {notice.unchanged && (
          <span className="block">{t("settings.vocabulary.errors.saved")}</span>
        )}
      </Alert>
      {notice.retry && <RetryButton onRetry={props.onRetry} />}
    </div>
  );
};

interface ListReadProps {
  screen: VocabularyScreen;
  onRetry: () => void;
}

/**
 * A failed read of the saved list, with Retry; nothing was written.
 * @param props screen prefix and retry callback
 */
export const ListReadFailure: React.FC<ListReadProps> = ({
  screen,
  onRetry,
}) => {
  const { t } = useTranslation();
  return (
    <div
      className="flex flex-col items-start gap-2"
      data-testid={`${screen}-list-error`}
      role="alert"
    >
      <Alert variant="error">{t("settings.vocabulary.listFailed")}</Alert>
      <RetryButton onRetry={onRetry} />
    </div>
  );
};

/**
 * The saved list before its first answer.
 * @param props screen prefix
 */
export const ListLoading: React.FC<{ screen: VocabularyScreen }> = ({
  screen,
}) => {
  const { t } = useTranslation();
  return (
    <p
      className="text-sm text-mid-gray"
      data-testid={`${screen}-loading`}
      role="status"
    >
      {t("settings.vocabulary.loading")}
    </p>
  );
};
