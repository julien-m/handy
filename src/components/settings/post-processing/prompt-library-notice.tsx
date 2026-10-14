// Notice of a failed library reload (005 AC-001): the last library stays on
// screen, flagged as possibly out of date, with Retry. Shown in the list and
// in the editor, which never falls back to older values because of it.
import React from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { PROMPT_KEYS } from "./prompt-library-model";
import type { PromptLibraryState } from "./use-prompt-library";

interface LibraryReloadNoticeProps {
  testId: string;
  state: PromptLibraryState;
}

/** Shown once a reload failed while a library is displayed. */
export const LibraryReloadNotice: React.FC<LibraryReloadNoticeProps> = ({
  testId,
  state,
}) => {
  const { t } = useTranslation();
  if (state.loadState !== "error" || !state.library) return null;
  const retry = (
    <Button
      variant="warning"
      size="sm"
      onClick={state.reload}
      data-testid={`${testId}-reload-retry`}
    >
      {t(`${PROMPT_KEYS}.list.retry`)}
    </Button>
  );
  return (
    <Alert
      variant="warning"
      role="alert"
      testId={`${testId}-reload-error`}
      action={retry}
    >
      {t(`${PROMPT_KEYS}.list.reloadFailed`)}
    </Alert>
  );
};
