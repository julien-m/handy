// Name and text fields of the prompt editor (005 AC-001, Behavioral AC
// `has_validation`): each field shows its issue or the backend's refusal,
// and the name field exposes its visual state (empty, invalid, valid).
import React, { useId } from "react";
import { Trans, useTranslation } from "react-i18next";
import { Input } from "../../ui/Input";
import { Textarea } from "../../ui/Textarea";
import { PROMPT_KEYS, nameFieldState } from "./prompt-library-model";
import type { PromptEditorModel } from "./use-prompt-editor";
import { useFocusOnMount } from "./use-focus-on-mount";

const KEY = `${PROMPT_KEYS}.editor`;

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
  /** The field takes the focus when the editor opens. */
  focusOnOpen?: boolean;
}

interface FieldLabelProps {
  htmlFor: string;
  testId: string;
  text: string;
}

const FieldLabel: React.FC<FieldLabelProps> = ({ htmlFor, testId, text }) => (
  <label
    htmlFor={htmlFor}
    className="text-sm font-semibold"
    data-testid={testId}
  >
    {text}
  </label>
);

/** Accessibility and highlight attributes of a field with an error. */
const errorAttributesOf = (error: string | null, errorId: string) => ({
  "aria-invalid": error !== null,
  "aria-describedby": error ? errorId : undefined,
  className: `w-full ${error ? ERROR_BORDER : ""}`,
});

/** Name of the prompt: 1–60 characters, unique ignoring case. */
export const PromptNameField: React.FC<EditorFieldsProps> = (props) => {
  const { testId, model, focusOnOpen = false } = props;
  const { t } = useTranslation();
  const inputId = useId();
  const errorId = `${inputId}-error`;
  const focusRef = useFocusOnMount<HTMLDivElement>(focusOnOpen);
  return (
    <div ref={focusRef} className="flex flex-col gap-2">
      <FieldLabel
        htmlFor={inputId}
        testId={`${testId}-form-name-label`}
        text={t(`${KEY}.name`)}
      />
      <Input
        id={inputId}
        type="text"
        value={model.draft.name}
        onChange={(event) => model.setName(event.target.value)}
        onBlur={model.touchName}
        placeholder={t(`${KEY}.namePlaceholder`)}
        data-testid={`${testId}-form-name`}
        data-state={nameFieldState(model.validation.nameIssue)}
        {...errorAttributesOf(model.nameError, errorId)}
      />
      <FieldError
        id={errorId}
        testId={`${testId}-form-name-error`}
        messageKey={model.nameError}
      />
    </div>
  );
};

/** The `${output}` placeholder hint under the text. */
const TextHint: React.FC<{ testId: string }> = ({ testId }) => (
  <p
    className="text-xs text-mid-gray/70"
    data-testid={`${testId}-form-text-hint`}
  >
    <Trans i18nKey={`${KEY}.textHint`} components={{ code: <code /> }} />
  </p>
);

/** Text of the prompt: 1–8000 characters, `${output}` stands for the transcript. */
export const PromptTextField: React.FC<EditorFieldsProps> = ({
  testId,
  model,
}) => {
  const { t } = useTranslation();
  const inputId = useId();
  const errorId = `${inputId}-error`;
  const label = t(`${KEY}.text`);
  return (
    <div className="flex flex-col gap-2">
      <FieldLabel
        htmlFor={inputId}
        testId={`${testId}-form-text-label`}
        text={label}
      />
      <Textarea
        id={inputId}
        value={model.draft.text}
        onChange={(event) => model.setText(event.target.value)}
        onBlur={model.touchText}
        placeholder={t(`${KEY}.textPlaceholder`)}
        data-testid={`${testId}-form-text`}
        {...errorAttributesOf(model.textError, errorId)}
      />
      <TextHint testId={testId} />
      <FieldError
        id={errorId}
        testId={`${testId}-form-text-error`}
        messageKey={model.textError}
      />
    </div>
  );
};
