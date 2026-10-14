// Action bar of the prompt editor (005 AC-001, Behavioral AC `is_submittable`):
// what saving does or which fields block it, Delete for a saved prompt,
// Cancel back to the list, and Save with its enabled/disabled/loading state.
import React from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { actionNoteKeyOf } from "./prompt-draft-model";
import { PROMPT_KEYS } from "./prompt-library-model";
import type { PromptDeletion } from "./prompt-delete-dialog";
import type { PromptEditorModel } from "./use-prompt-editor";

const KEY = `${PROMPT_KEYS}.actions`;
const BUTTON_CLASSES =
  "shrink-0 whitespace-nowrap focus-visible:ring-2 focus-visible:ring-logo-primary focus-visible:outline-none";

interface PromptEditorActionsProps {
  testId: string;
  model: PromptEditorModel;
  /** Deletion of a saved prompt; absent for a new prompt. */
  deletion: PromptDeletion | null;
  onCancel: () => void;
}

/** Saved confirmation and refusals of the form or of the deletion. */
const ActionMessages: React.FC<PromptEditorActionsProps> = ({
  testId,
  model,
  deletion,
}) => {
  const { t } = useTranslation();
  const error = model.formError ?? deletion?.error ?? null;
  return (
    <>
      {model.justSaved && (
        <p
          role="status"
          className="px-4 pb-3 text-sm text-green-400"
          data-testid={`${testId}-saved`}
        >
          {t(`${KEY}.saved`)}
        </p>
      )}
      {error && (
        <p
          role="alert"
          className="px-4 pb-3 text-sm text-red-400"
          data-testid={`${testId}-actions-error`}
        >
          {t(error)}
        </p>
      )}
    </>
  );
};

/** Save with its enabled, disabled or loading state. */
const SaveButton: React.FC<PromptEditorActionsProps> = ({ testId, model }) => {
  const { t } = useTranslation();
  const isSaving = model.saveState === "loading";
  return (
    <Button
      variant="primary"
      className={BUTTON_CLASSES}
      onClick={() => void model.save()}
      disabled={model.saveState !== "enabled"}
      aria-busy={isSaving}
      data-state={model.saveState}
      data-testid={`${testId}-save`}
    >
      {t(`${KEY}.${isSaving ? "saving" : "save"}`)}
    </Button>
  );
};

/** Delete (saved prompt only), Cancel and Save. */
const ActionButtons: React.FC<PromptEditorActionsProps> = (props) => {
  const { testId, deletion, onCancel } = props;
  const { t } = useTranslation();
  const deleteButton = deletion && (
    <Button
      variant="danger-ghost"
      className={BUTTON_CLASSES}
      onClick={deletion.ask}
      disabled={deletion.isDeleting}
      data-testid={`${testId}-delete`}
    >
      {t(`${KEY}.delete`)}
    </Button>
  );
  return (
    <>
      {deleteButton}
      <Button
        variant="secondary"
        className={BUTTON_CLASSES}
        onClick={onCancel}
        data-testid={`${testId}-cancel`}
      >
        {t(`${KEY}.cancel`)}
      </Button>
      <SaveButton {...props} />
    </>
  );
};

/** The editor's action bar. */
export const PromptEditorActions: React.FC<PromptEditorActionsProps> = (
  props,
) => {
  const { t } = useTranslation();
  const { testId, model } = props;
  return (
    <div data-testid={`${testId}-actions`}>
      <SettingsGroup>
        <div>
          <div className="flex items-center gap-3 p-4">
            <p
              className="flex-1 min-w-0 text-sm text-mid-gray"
              data-testid={`${testId}-actions-note`}
            >
              {t(`${KEY}.${actionNoteKeyOf(model)}`)}
            </p>
            <ActionButtons {...props} />
          </div>
          <ActionMessages {...props} />
        </div>
      </SettingsGroup>
    </div>
  );
};
