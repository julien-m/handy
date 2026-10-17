// History limit setting: a number field validated as it is typed. A refused
// value stays in the field with its error under it and is never sent; an
// accepted one is saved when the field is left (or on Enter), so typing
// "12" never saves the intermediate 1, whose cleanup would delete history.
import React, { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";
import {
  HISTORY_LIMIT_RANGE,
  type HistoryLimitEvent,
  SHOWN_STORED,
  historyLimitStep,
  isBlurWithinWindow,
  isCommitKey,
  parseHistoryLimit,
} from "./history-limit-model";
import { useReportedOutputScreen } from "./output-screen";

// Shown until the settings are loaded, as before 008.
const DEFAULT_LIMIT = 5;

interface HistoryLimitProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

/** The field's text and its event handler; saves through the settings. */
const useHistoryLimitField = () => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [field, setField] = useState(SHOWN_STORED);
  const stored = getSetting("history_limit") ?? DEFAULT_LIMIT;
  const apply = (event: HistoryLimitEvent) => {
    const step = historyLimitStep(field, event, stored);
    setField(step.field);
    if (step.save !== null) void updateSetting("history_limit", step.save);
  };
  const text = field.draft ?? String(stored);
  const isBusy = isUpdating("history_limit");
  return { text, isValid: parseHistoryLimit(text).isValid, isBusy, apply };
};

/** The inline error of a refused value. */
const HistoryLimitError: React.FC<{ id: string; testId: string }> = ({
  id,
  testId,
}) => {
  const { t } = useTranslation();
  return (
    <p
      id={id}
      role="alert"
      className="text-sm text-red-500 text-right"
      data-testid={`${testId}-error`}
    >
      {t("settings.debug.historyLimit.invalid", HISTORY_LIMIT_RANGE)}
    </p>
  );
};

type HistoryLimitFieldState = ReturnType<typeof useHistoryLimitField>;

interface NumberFieldProps {
  field: HistoryLimitFieldState;
  errorId: string;
  /** Test id of the row, under the output screen shown. */
  testId: string;
}

/** The number field: typing keeps a draft, Enter or leaving it inside the window saves. */
const HistoryLimitNumberField: React.FC<NumberFieldProps> = ({
  field,
  errorId,
  testId,
}) => (
  <Input
    type="number"
    min={HISTORY_LIMIT_RANGE.min}
    max={HISTORY_LIMIT_RANGE.max}
    value={field.text}
    onChange={(event) =>
      field.apply({ type: "typed", text: event.target.value })
    }
    onBlur={(event) => {
      const inside = isBlurWithinWindow(
        document.hasFocus(),
        event.relatedTarget,
      );
      field.apply({ type: "left", withinWindow: inside });
    }}
    onKeyDown={(event) =>
      isCommitKey(event.key) && field.apply({ type: "committed" })
    }
    disabled={field.isBusy}
    invalid={!field.isValid}
    aria-describedby={field.isValid ? undefined : errorId}
    data-testid={`${testId}-input`}
    className="w-20"
  />
);

/** The number field with its unit, and the error under them. */
const HistoryLimitInput: React.FC<NumberFieldProps> = (props) => {
  const { t } = useTranslation();
  const { field, errorId, testId } = props;
  return (
    <div className="flex flex-col items-end gap-1">
      <div className="flex items-center space-x-2">
        <HistoryLimitNumberField {...props} />
        <span className="text-sm text-text">
          {t("settings.debug.historyLimit.entries")}
        </span>
      </div>
      {!field.isValid && <HistoryLimitError id={errorId} testId={testId} />}
    </div>
  );
};

// @spec AC-003: Same validation as the UI, shown inline — .specs/features/008-settings-cli/spec.md#ac-003
export const HistoryLimit: React.FC<HistoryLimitProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const errorId = useId();
  const field = useHistoryLimitField();
  // A refused value is the settings-write-errors screen (008).
  const screen = useReportedOutputScreen(
    "history-limit",
    field.isValid ? null : "settings-write-errors",
  );
  const testId = `${screen}-history-limit`;
  return (
    <SettingContainer
      title={t("settings.debug.historyLimit.title")}
      description={t("settings.debug.historyLimit.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      testId={testId}
    >
      <HistoryLimitInput field={field} errorId={errorId} testId={testId} />
    </SettingContainer>
  );
};
