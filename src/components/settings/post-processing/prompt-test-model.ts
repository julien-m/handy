// Pure rules of the prompt test panel (005 AC-007): the panel state of each
// test outcome, the error messages of failures and refusals, the sample
// prefilled from the last transcript, and whether a result still describes
// the draft shown.
import type {
  HistoryEntry,
  PromptProviderOverride,
  PromptTestRequest,
  PromptTestResult,
  SettingsCommandError,
} from "@/bindings";
import { settingsErrorKey } from "../../../stores/settings-errors";
import type { TranslatedMessage } from "../shortcut-list-model";
import { PROMPT_KEYS, reasonKey } from "./prompt-library-model";

/** Visual state of the test panel (Behavioral AC `async_action`). */
export type TestPanelState = "idle" | "loading" | "success" | "error";

/** Result shown after a successful test. */
export interface TestSuccess {
  text: string;
  providerId: string;
  model: string;
  /** Duration in seconds, one decimal, as shown. */
  seconds: string;
}

/** What the panel shows for a finished test. */
export interface TestView {
  state: TestPanelState;
  success: TestSuccess | null;
  /** Error or notice of a test without result. */
  message: TranslatedMessage | null;
  /** Translation key of the failure reason, interpolated as `reason`. */
  reasonKey: string | null;
  /** Whether the message is a failure the user can retry. */
  canRetry: boolean;
}

/** The panel before any test, and after a cancelled one. */
export const IDLE_TEST_VIEW: TestView = {
  state: "idle",
  success: null,
  message: null,
  reasonKey: null,
  canRetry: false,
};

const TEST_KEY = `${PROMPT_KEYS}.test`;

/**
 * Duration as shown in the result meta line.
 * @param durationMs Duration of the test in milliseconds.
 * @returns Seconds with one decimal ("1.4").
 */
export const secondsOf = (durationMs: number): string =>
  (durationMs / 1000).toFixed(1);

const errorView = (
  message: TranslatedMessage,
  canRetry: boolean,
  failureKey: string | null = null,
): TestView => ({
  state: "error",
  success: null,
  message,
  reasonKey: failureKey,
  canRetry,
});

const successViewOf = (result: PromptTestResult): TestView => ({
  state: "success",
  success: {
    text: result.text ?? "",
    providerId: result.provider_id ?? "",
    model: result.model ?? "",
    seconds: secondsOf(result.duration_ms),
  },
  message: null,
  reasonKey: null,
  canRetry: false,
});

// @spec AC-007: Test panel result, provider, model and duration or error — .specs/features/005-custom-ai-prompts/spec.md#ac-007
/**
 * Panel view of a test result: the processed text with provider, model and
 * duration, or the failure with a retry. A test refused because another one
 * runs, or skipped for an empty sample, is a notice without retry; a
 * cancelled test (the editor closed) leaves the panel idle.
 * @param result Result returned by `test_post_process_prompt`.
 */
export const testViewOf = (result: PromptTestResult): TestView => {
  switch (result.status) {
    case "success":
      return successViewOf(result);
    case "failed":
      return errorView(
        { key: `${TEST_KEY}.failedTitle`, params: {} },
        true,
        // A failed test always carries its reason; a generic provider error
        // is the safe wording if a future backend omits it.
        reasonKey(result.failure ?? "provider_error"),
      );
    case "busy":
      return errorView({ key: `${TEST_KEY}.busy`, params: {} }, false);
    case "skipped":
      return errorView({ key: `${TEST_KEY}.skipped`, params: {} }, false);
    default:
      return IDLE_TEST_VIEW;
  }
};

/**
 * Panel view of a test the backend refused before running it (a draft the
 * library would refuse), translated from its typed reason.
 * @param error Typed refusal of `test_post_process_prompt`.
 */
export const refusedTestViewOf = (error: SettingsCommandError): TestView =>
  errorView({ key: settingsErrorKey(error), params: {} }, true);

/**
 * Sample prefilled in the test box: the raw transcript of the latest
 * history entry (text-only rows included), else an empty box.
 * @param entries History entries, newest first.
 */
export const latestTranscriptOf = (
  entries: ReadonlyArray<HistoryEntry>,
): string => entries[0]?.transcription_text ?? "";

/** Whether two provider overrides are the same provider and model. */
const sameOverride = (
  left: PromptProviderOverride | null,
  right: PromptProviderOverride | null,
): boolean =>
  left === null || right === null
    ? left === right
    : left.provider_id === right.provider_id &&
      left.model.trim() === right.model.trim();

/**
 * Whether a test result no longer describes the draft: its text, provider,
 * model or sample changed since the test ran. The panel then marks it as
 * outdated instead of passing it off as the current draft's result.
 * @param tested Request the shown result answers, or null before any test.
 * @param current Request the current draft and sample would send.
 */
export const isResultOutdated = (
  tested: PromptTestRequest | null,
  current: PromptTestRequest,
): boolean =>
  tested !== null &&
  (tested.text !== current.text ||
    tested.sample !== current.sample ||
    !sameOverride(tested.provider_override, current.provider_override));
