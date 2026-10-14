// History limit setting: a number field validated as it is typed. A refused
// value stays in the field with its error under it and is never sent; an
// accepted one is saved when the field is left (or on Enter), so typing
// "12" never saves the intermediate 1, whose cleanup would delete history.
import React, { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";
import { HISTORY_LIMIT_RANGE, parseHistoryLimit } from "./history-limit-model";

const TEST_ID = "settings-write-errors-history-limit";

interface HistoryLimitProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

// @spec AC-003: Same validation as the UI, shown inline — .specs/features/008-settings-cli/spec.md#ac-003
export const HistoryLimit: React.FC<HistoryLimitProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  // The text being typed; null shows the stored value.
  const [draft, setDraft] = useState<string | null>(null);
  const errorId = useId();
  const stored = getSetting("history_limit") ?? 5;
  const text = draft ?? String(stored);
  const parsed = parseHistoryLimit(text);

  const commit = () => {
    if (!parsed.isValid) return;
    if (parsed.value !== stored) {
      void updateSetting("history_limit", parsed.value);
    }
    setDraft(null);
  };

  return (
    <SettingContainer
      title={t("settings.debug.historyLimit.title")}
      description={t("settings.debug.historyLimit.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
      testId={TEST_ID}
    >
      <div className="flex flex-col items-end gap-1">
        <div className="flex items-center space-x-2">
          <Input
            type="number"
            min={HISTORY_LIMIT_RANGE.min}
            max={HISTORY_LIMIT_RANGE.max}
            value={text}
            onChange={(event) => setDraft(event.target.value)}
            onBlur={commit}
            onKeyDown={(event) => {
              if (event.key === "Enter") commit();
            }}
            disabled={isUpdating("history_limit")}
            invalid={!parsed.isValid}
            aria-describedby={parsed.isValid ? undefined : errorId}
            data-testid={`${TEST_ID}-input`}
            className="w-20"
          />
          <span className="text-sm text-text">
            {t("settings.debug.historyLimit.entries")}
          </span>
        </div>
        {!parsed.isValid && (
          <p
            id={errorId}
            role="alert"
            className="text-sm text-red-500 text-right"
            data-testid={`${TEST_ID}-error`}
          >
            {t("settings.debug.historyLimit.invalid", HISTORY_LIMIT_RANGE)}
          </p>
        )}
      </div>
    </SettingContainer>
  );
};
