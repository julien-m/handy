// Name and text fields of the prompt editor (005 AC-001, Behavioral AC
// `has_validation`): each field shows its issue or the backend's refusal,
// and the name field exposes its visual state (empty, invalid, valid).
import React, { useId } from "react";
import { Trans, useTranslation } from "react-i18next";
import { Input } from "../../ui/Input";
import { Textarea } from "../../ui/Textarea";
import { PROMPT_KEYS, nameFieldState } from "./prompt-library-model";
import type { PromptEditorModel } from "./use-prompt-editor";

/** Error classes of a field: the red border wins over hover and focus. */
const ERROR_BORDER = "border-red-400!";

interface FieldErrorProps {
  id: string;
  testId: string;
  messageKey: string | null;
}

const FieldError: React.FC<FieldErrorProps> = ({ id, testId, messageKey }) => {
  const { t } = useTranslation();
  if (!messageKey) return null;
  return (
    <p
      id={id}
      role="alert"
      className="text-sm text-red-400"
      data-testid={testId}
    >
      {t(messageKey)}
    </p>
  );
};

interface EditorFieldsProps {
  testId: string;
  model: PromptEditorModel;
}

/** Name of the prompt: 1–60 characters, unique ignoring case. */
export const PromptNameField: React.FC<EditorFieldsProps> = ({
  testId,
  model,
}) => {
  const { t } = useTranslation();
  const inputId = useId();
  const errorId = `${inputId}-error`;
  return (
    <div className="flex flex-col gap-2">
      <label
        htmlFor={inputId}
        className="text-sm font-semibold"
        data-testid={`${testId}-form-name-label`}
      >
        {t(`${PROMPT_KEYS}.editor.name`)}
      </label>
      <Input
        id={inputId}
        type="text"
        value={model.draft.name}
        onChange={(event) => model.setName(event.target.value)}
        onBlur={model.touchName}
        placeholder={t(`${PROMPT_KEYS}.editor.namePlaceholder`)}
        data-testid={`${testId}-form-name`}
        data-state={nameFieldState(model.validation.nameIssue)}
        aria-invalid={model.nameError !== null}
        aria-describedby={model.nameError ? errorId : undefined}
        className={`w-full ${model.nameError ? ERROR_BORDER : ""}`}
      />
      <FieldError
        id={errorId}
        testId={`${testId}-form-name-error`}
        messageKey={model.nameError}
      />
    </div>
  );
};

/** Text of the prompt: 1–8000 characters, `${output}` stands for the transcript. */
export const PromptTextField: React.FC<EditorFieldsProps> = ({
  testId,
  model,
}) => {
  const { t } = useTranslation();
  const inputId = useId();
  const errorId = `${inputId}-error`;
  return (
    <div className="flex flex-col gap-2">
      <label
        htmlFor={inputId}
        className="text-sm font-semibold"
        data-testid={`${testId}-form-text-label`}
      >
        {t(`${PROMPT_KEYS}.editor.text`)}
      </label>
      <Textarea
        id={inputId}
        value={model.draft.text}
        onChange={(event) => model.setText(event.target.value)}
        onBlur={model.touchText}
        placeholder={t(`${PROMPT_KEYS}.editor.textPlaceholder`)}
        data-testid={`${testId}-form-text`}
        aria-invalid={model.textError !== null}
        aria-describedby={model.textError ? errorId : undefined}
        className={`w-full ${model.textError ? ERROR_BORDER : ""}`}
      />
      <p
        className="text-xs text-mid-gray/70"
        data-testid={`${testId}-form-text-hint`}
      >
        <Trans
          i18nKey={`${PROMPT_KEYS}.editor.textHint`}
          components={{ code: <code /> }}
        />
      </p>
      <FieldError
        id={errorId}
        testId={`${testId}-form-text-error`}
        messageKey={model.textError}
      />
    </div>
  );
};
