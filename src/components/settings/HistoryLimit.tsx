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
  isCommitKey,
  parseHistoryLimit,
} from "./history-limit-model";

const TEST_ID = "settings-write-errors-history-limit";
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
const HistoryLimitError: React.FC<{ id: string }> = ({ id }) => {
  const { t } = useTranslation();
  return (
    <p
      id={id}
      role="alert"
      className="text-sm text-red-500 text-right"
      data-testid={`${TEST_ID}-error`}
    >
      {t("settings.debug.historyLimit.invalid", HISTORY_LIMIT_RANGE)}
    </p>
  );
};

/** The number field and its unit. */
const HistoryLimitInput: React.FC<{ errorId: string }> = ({ errorId }) => {
  const { t } = useTranslation();
  const { text, isValid, isBusy, apply } = useHistoryLimitField();
  return (
    <div className="flex flex-col items-end gap-1">
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min={HISTORY_LIMIT_RANGE.min}
          max={HISTORY_LIMIT_RANGE.max}
          value={text}
          onChange={(event) =>
            apply({ type: "typed", text: event.target.value })
          }
          onBlur={() => apply({ type: "committed" })}
          onKeyDown={(event) =>
            isCommitKey(event.key) && apply({ type: "committed" })
          }
          disabled={isBusy}
          invalid={!isValid}
          aria-describedby={isValid ? undefined : errorId}
          data-testid={`${TEST_ID}-input`}
          className="w-20"
        />
        <span className="text-sm text-text">
          {t("settings.debug.historyLimit.entries")}
        </span>
      </div>
      {!isValid && <HistoryLimitError id={errorId} />}
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
  return (
    <SettingContainer
      title={t("settings.debug.historyLimit.title")}
      description={t("settings.debug.historyLimit.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      testId={TEST_ID}
    >
      <HistoryLimitInput errorId={errorId} />
    </SettingContainer>
  );
};
