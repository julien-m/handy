/**
 * Pure rules of the AI prompt settings (005): validation mirrored from the
 * backend library, row flags, provider readiness of an unsaved draft, Save
 * state, refusal fields, external changes and test panel views.
 * Run with `bun run test:prompt-settings`.
 */
import { describe, expect, test } from "bun:test";
import type {
  ListedPrompt,
  PostProcessProvider,
  PromptTestResult,
  SettingsCommandError,
} from "@/bindings";
import {
  EMPTY_DRAFT,
  draftOf,
  editorTestIdOf,
  externalChangeOf,
  fieldErrorOf,
  isDraftDirty,
  refusalFieldOf,
  saveStateOf,
  savedAfterCommit,
  validateDraft,
} from "./prompt-draft-model";
import {
  MAX_NAME_CHARS,
  MAX_TEXT_CHARS,
  charCount,
  nameFieldState,
  nameIssueOf,
  rowStatusOf,
  textIssueOf,
} from "./prompt-library-model";
import {
  APPLE_PROVIDER_ID,
  DEFAULT_PROVIDER_VALUE,
  type ProviderSettings,
  canListModels,
  draftReadinessOf,
  modelOptionsOf,
  overrideOf,
  providerOptionsOf,
  readinessFixOf,
  sameProviderDraft,
} from "./prompt-provider-model";
import {
  IDLE_TEST_VIEW,
  latestTranscriptOf,
  refusedTestViewOf,
  secondsOf,
  testViewOf,
} from "./prompt-test-model";

const provider = (id: string, baseUrl: string): PostProcessProvider => ({
  id,
  label: id.toUpperCase(),
  base_url: baseUrl,
});
const settings: ProviderSettings = {
  providers: [
    provider("openai", "https://api.openai.com/v1"),
    provider("custom", "http://localhost:11434/v1"),
    provider(APPLE_PROVIDER_ID, ""),
  ],
  defaultProviderId: "custom",
  models: { custom: "llama3.2", openai: "gpt-4.1-mini" },
  apiKeys: {},
};
const listed = (patch: Partial<ListedPrompt> = {}): ListedPrompt => ({
  id: "p1",
  name: "Shorten",
  prompt: "Shorten ${output}",
  provider_override: null,
  starter: true,
  issues: [],
  shortcuts: [],
  readiness: {
    provider_id: "custom",
    model: "llama3.2",
    is_override: false,
    ready: true,
    missing: null,
  },
  ...patch,
});
const refusal = (
  reason: SettingsCommandError["reason"],
): SettingsCommandError => ({
  code: "validation_failed",
  message: "Invalid value.",
  field: "post_process_prompts",
  reason,
  outcome: null,
  current_revision: null,
});

// @spec AC-001: Prompt name and text validation — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("prompt validation mirrors the backend library", () => {
  test("a name is trimmed, 1–60 characters and unique ignoring case", () => {
    expect(nameIssueOf("   ", [])).toBe("name_empty");
    expect(nameIssueOf("a".repeat(MAX_NAME_CHARS), [])).toBeNull();
    expect(nameIssueOf("a".repeat(MAX_NAME_CHARS + 1), [])).toBe(
      "name_too_long",
    );
    expect(nameIssueOf(" notes ", ["Notes"])).toBe("name_duplicate");
    expect(nameIssueOf("Notes 2", ["Notes"])).toBeNull();
  });
  test("characters are counted like Rust, not as UTF-16 units", () => {
    expect(charCount("😀")).toBe(1);
    expect(nameIssueOf("😀".repeat(MAX_NAME_CHARS), [])).toBeNull();
  });
  test("a text is non-blank and at most 8000 characters, kept verbatim", () => {
    expect(textIssueOf(" \n\t")).toBe("text_empty");
    expect(textIssueOf("x".repeat(MAX_TEXT_CHARS))).toBeNull();
    expect(textIssueOf("x".repeat(MAX_TEXT_CHARS + 1))).toBe("text_too_long");
  });
  test("the name field state is empty, invalid or valid", () => {
    expect(nameFieldState("name_empty")).toBe("empty");
    expect(nameFieldState("name_duplicate")).toBe("invalid");
    expect(nameFieldState(null)).toBe("valid");
  });
  test("a draft is valid only when both fields are", () => {
    const draft = { ...EMPTY_DRAFT, name: "Formal email", text: "${output}" };
    expect(validateDraft(draft, []).valid).toBe(true);
    expect(validateDraft({ ...draft, text: "" }, []).textIssue).toBe(
      "text_empty",
    );
    expect(validateDraft(draft, ["formal EMAIL"]).nameIssue).toBe(
      "name_duplicate",
    );
  });
});

// @spec AC-002: Invalid prompts flagged in the list — .specs/features/005-custom-ai-prompts/spec.md#ac-002
describe("list rows", () => {
  test("an invalid prompt is flagged before its readiness", () => {
    const row = listed({ issues: ["name_too_long"] });
    expect(rowStatusOf(row)).toEqual({
      kind: "invalid",
      reasonKey: "settings.postProcessing.promptLibrary.issues.name_too_long",
    });
  });
  test("a prompt whose provider is not ready names the missing item", () => {
    const readiness = {
      ...listed().readiness,
      ready: false,
      missing: "missing_api_key" as const,
    };
    expect(rowStatusOf(listed({ readiness }))).toEqual({
      kind: "not_ready",
      reasonKey:
        "settings.postProcessing.promptLibrary.reasons.missing_api_key",
    });
    expect(rowStatusOf(listed())).toBeNull();
  });
});

// @spec AC-004: Readiness of the edited provider choice — .specs/features/005-custom-ai-prompts/spec.md#ac-004
describe("provider override", () => {
  test("the default provider uses its configured model", () => {
    const readiness = draftReadinessOf(settings, EMPTY_DRAFT.provider, false);
    expect(readiness.providerId).toBe("custom");
    expect(readiness.model).toBe("llama3.2");
    expect(readiness.missing).toBeNull();
  });
  test("missing items follow the backend order: endpoint, model, key", () => {
    const noUrl = { ...settings, providers: [provider("custom", " ")] };
    expect(
      draftReadinessOf(noUrl, { providerId: "custom", model: "m" }, false)
        .missing,
    ).toBe("missing_endpoint");
    expect(
      draftReadinessOf(settings, { providerId: "openai", model: " " }, false)
        .missing,
    ).toBe("missing_model");
    expect(
      draftReadinessOf(settings, { providerId: "openai", model: "gpt" }, false)
        .missing,
    ).toBe("missing_api_key");
    const withKey = { ...settings, apiKeys: { openai: "sk-test" } };
    expect(
      draftReadinessOf(withKey, { providerId: "openai", model: "gpt" }, false)
        .missing,
    ).toBeNull();
    expect(
      draftReadinessOf(settings, { providerId: "gone", model: "m" }, false)
        .missing,
    ).toBe("unknown_provider");
  });
  test("Apple Intelligence needs no key but must be available", () => {
    const apple = { providerId: APPLE_PROVIDER_ID, model: "apple" };
    expect(draftReadinessOf(settings, apple, false).missing).toBe(
      "provider_unavailable",
    );
    expect(draftReadinessOf(settings, apple, true).missing).toBeNull();
  });
  test("Apple Intelligence is offered only where it is available", () => {
    const values = (available: boolean, current: string) =>
      providerOptionsOf(settings.providers, available, current, "Default").map(
        (o) => o.value,
      );
    expect(values(false, DEFAULT_PROVIDER_VALUE)).toEqual([
      "",
      "openai",
      "custom",
    ]);
    expect(values(true, DEFAULT_PROVIDER_VALUE)).toContain(APPLE_PROVIDER_ID);
    // A prompt that already uses it keeps its choice visible.
    expect(values(false, APPLE_PROVIDER_ID)).toContain(APPLE_PROVIDER_ID);
  });
  test("the override sent is none for the default provider", () => {
    expect(overrideOf(EMPTY_DRAFT.provider)).toBeNull();
    expect(overrideOf({ providerId: "custom", model: " llama3.2 " })).toEqual({
      provider_id: "custom",
      model: "llama3.2",
    });
    expect(
      sameProviderDraft(
        { providerId: "", model: "x" },
        { providerId: "", model: "y" },
      ),
    ).toBe(true);
  });
  test("only a missing key or endpoint is fixed outside the editor", () => {
    expect(readinessFixOf("missing_api_key")).toBe("api_key");
    expect(readinessFixOf("missing_endpoint")).toBe("endpoint");
    expect(readinessFixOf("missing_model")).toBeNull();
  });
  test("models are listed once the provider can answer", () => {
    expect(canListModels(settings, "custom")).toBe(true);
    expect(canListModels(settings, "openai")).toBe(false);
    expect(canListModels(settings, APPLE_PROVIDER_ID)).toBe(false);
    expect(modelOptionsOf(["a"], "mine").map((o) => o.value)).toEqual([
      "mine",
      "a",
    ]);
    expect(modelOptionsOf(undefined, "")).toEqual([]);
  });
});

// @spec AC-001: Save state and refusals of the editor — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("editor draft", () => {
  test("Save is enabled for a valid change, disabled otherwise, loading while saving", () => {
    const base = { valid: true, dirty: true, saving: false, deleted: false };
    expect(saveStateOf(base)).toBe("enabled");
    expect(saveStateOf({ ...base, valid: false })).toBe("disabled");
    expect(saveStateOf({ ...base, dirty: false })).toBe("disabled");
    expect(saveStateOf({ ...base, deleted: true })).toBe("disabled");
    expect(saveStateOf({ ...base, saving: true })).toBe("loading");
  });
  test("a backend refusal goes under its field", () => {
    expect(refusalFieldOf(refusal("prompt_name_duplicate"))).toBe("name");
    expect(refusalFieldOf(refusal("prompt_text_too_long"))).toBe("text");
    expect(refusalFieldOf(refusal("prompt_override_incomplete"))).toBe(
      "provider",
    );
    expect(refusalFieldOf(refusal("shortcut_session_busy"))).toBe("form");
  });
  test("a field shows its own issue once shown, else its refusal", () => {
    const nameRefusal = {
      field: "name" as const,
      key: "errors.settings.promptNameDuplicate",
    };
    expect(fieldErrorOf("issue", true, nameRefusal, "name")).toBe("issue");
    expect(fieldErrorOf("issue", false, nameRefusal, "name")).toBe(
      nameRefusal.key,
    );
    expect(fieldErrorOf(null, true, nameRefusal, "text")).toBeNull();
  });
  test("a partial save keeps the unsaved part as before", () => {
    const saved = draftOf(listed());
    const draft = {
      name: " Brief ",
      text: "New",
      provider: { providerId: "openai", model: "gpt" },
    };
    const next = savedAfterCommit(draft, saved, {
      content: true,
      provider: false,
    });
    expect(next.name).toBe("Brief");
    expect(next.text).toBe("New");
    expect(next.provider).toEqual(saved.provider);
    expect(isDraftDirty(draft, next)).toBe(true);
  });
  test("a change made elsewhere is detected, a deletion too", () => {
    const saved = draftOf(listed());
    expect(externalChangeOf(saved, listed())).toBe("none");
    expect(externalChangeOf(saved, listed({ prompt: "Other" }))).toBe(
      "changed",
    );
    expect(externalChangeOf(saved, undefined)).toBe("deleted");
  });
  test("the test-id prefix follows the contract screen of the mode", () => {
    expect(editorTestIdOf({ mode: "edit", id: "p1" })).toBe(
      "prompt-settings-editor",
    );
    expect(editorTestIdOf({ mode: "new" })).toBe("prompt-settings-invalid");
  });
});

// @spec AC-007: Test panel outcomes — .specs/features/005-custom-ai-prompts/spec.md#ac-007
describe("test panel", () => {
  const result = (patch: Partial<PromptTestResult>): PromptTestResult => ({
    status: "success",
    text: "Meeting moves to Thursday.",
    failure: null,
    provider_id: "custom",
    model: "llama3.2",
    duration_ms: 1420,
    ...patch,
  });
  test("a success shows the text, provider, model and duration", () => {
    const view = testViewOf(result({}));
    expect(view.state).toBe("success");
    expect(view.success).toEqual({
      text: "Meeting moves to Thursday.",
      providerId: "custom",
      model: "llama3.2",
      seconds: "1.4",
    });
  });
  test("a failure shows its translated reason and a retry", () => {
    const view = testViewOf(
      result({ status: "failed", text: null, failure: "timeout" }),
    );
    expect(view.state).toBe("error");
    expect(view.canRetry).toBe(true);
    expect(view.reasonKey).toBe(
      "settings.postProcessing.promptLibrary.reasons.timeout",
    );
  });
  test("busy and empty sample are notices without retry; cancelled is idle", () => {
    expect(testViewOf(result({ status: "busy" })).canRetry).toBe(false);
    expect(testViewOf(result({ status: "skipped" })).message?.key).toBe(
      "settings.postProcessing.promptLibrary.test.skipped",
    );
    expect(testViewOf(result({ status: "cancelled" }))).toEqual(IDLE_TEST_VIEW);
  });
  test("a refused draft is translated from its typed reason", () => {
    expect(refusedTestViewOf(refusal("prompt_text_empty")).message?.key).toBe(
      "errors.settings.promptTextEmpty",
    );
  });
  test("the sample is the latest raw transcript, or empty", () => {
    expect(latestTranscriptOf([])).toBe("");
    expect(secondsOf(30000)).toBe("30.0");
  });
});
