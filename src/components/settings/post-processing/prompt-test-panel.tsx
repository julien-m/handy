// Test panel of the prompt editor (005 AC-007, Behavioral AC `async_action`):
// an editable sample, one test at a time, and the result with its model,
// provider and duration, or the error with a retry. Visual states: idle,
// loading, success, error.
import React from "react";
import { useTranslation } from "react-i18next";
import { Play } from "lucide-react";
import { useSettings } from "../../../hooks/useSettings";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { Textarea } from "../../ui/Textarea";
import { PROMPT_KEYS } from "./prompt-library-model";
import { providerLabelOf } from "./prompt-provider-model";
import type { TestSuccess, TestView } from "./prompt-test-model";
import type { PromptTestModel } from "./use-prompt-test";

const KEY = `${PROMPT_KEYS}.test`;

interface PanelPartProps {
  testId: string;
}

/** Processed text and its "model · provider · duration" line. */
const TestResult: React.FC<PanelPartProps & { success: TestSuccess }> = ({
  testId,
  success,
}) => {
  const { t } = useTranslation();
  const { getSetting } = useSettings();
  const provider = providerLabelOf(
    getSetting("post_process_providers") ?? [],
    success.providerId,
  );
  return (
    <div
      role="status"
      className="p-3 space-y-1 rounded-md border border-mid-gray/20 bg-mid-gray/5"
      data-testid={`${testId}-test-result`}
    >
      <p
        className="text-sm whitespace-pre-wrap"
        data-testid={`${testId}-test-output`}
      >
        {success.text}
      </p>
      <p className="text-xs text-mid-gray" data-testid={`${testId}-test-meta`}>
        {t(`${KEY}.meta`, {
          model: success.model,
          provider,
          seconds: success.seconds,
        })}
      </p>
    </div>
  );
};

interface TestErrorProps extends PanelPartProps {
  view: TestView;
  onRetry: () => void;
}

/** Failure with its reason and a retry, or a notice (busy, empty sample). */
const TestError: React.FC<TestErrorProps> = ({ testId, view, onRetry }) => {
  const { t } = useTranslation();
  if (!view.message) return null;
  const reason = view.reasonKey ? t(view.reasonKey) : "";
  return (
    <Alert
      variant="error"
      role="alert"
      testId={`${testId}-test-error`}
      action={
        view.canRetry && (
          <Button
            variant="secondary"
            size="sm"
            onClick={onRetry}
            data-testid={`${testId}-test-retry`}
          >
            {t(`${KEY}.retry`)}
          </Button>
        )
      }
    >
      <span className="block" data-testid={`${testId}-test-error-title`}>
        {t(view.message.key, { ...view.message.params, reason })}
      </span>
      {view.reasonKey && (
        <span className="block">{t(`${KEY}.failedHint`)}</span>
      )}
    </Alert>
  );
};

interface PromptTestPanelProps extends PanelPartProps {
  test: PromptTestModel;
}

// @spec AC-007: Test panel on editable sample — .specs/features/005-custom-ai-prompts/spec.md#ac-007
/** The test group of the editor. */
export const PromptTestPanel: React.FC<PromptTestPanelProps> = ({
  testId,
  test,
}) => {
  const { t } = useTranslation();
  const isLoading = test.state === "loading";
  const run = () => void test.run();
  return (
    <div
      data-testid={`${testId}-test`}
      data-state={test.state}
      aria-busy={isLoading}
    >
      <SettingsGroup title={t(`${KEY}.title`)}>
        <div className="p-4 space-y-3">
          <Textarea
            variant="compact"
            value={test.sample}
            onChange={(event) => test.setSample(event.target.value)}
            placeholder={t(`${KEY}.samplePlaceholder`)}
            aria-label={t(`${KEY}.sampleLabel`)}
            data-testid={`${testId}-test-sample`}
            className="w-full min-h-[60px]"
          />
          <div className="flex flex-wrap items-center gap-3">
            <Button
              variant="secondary"
              size="md"
              onClick={run}
              disabled={isLoading}
              data-testid={`${testId}-test-run`}
              className="inline-flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-logo-primary"
            >
              <Play size={16} aria-hidden="true" />
              {t(`${KEY}.${isLoading ? "running" : "run"}`)}
            </Button>
            <span
              className="text-sm text-mid-gray"
              data-testid={`${testId}-test-note`}
            >
              {t(`${KEY}.note`)}
            </span>
          </div>
          {!isLoading && test.view.success && (
            <TestResult testId={testId} success={test.view.success} />
          )}
          {!isLoading && (
            <TestError testId={testId} view={test.view} onRetry={run} />
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};
