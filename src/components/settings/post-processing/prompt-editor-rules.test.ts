/**
 * Editor rules added by the delivery review of the prompt settings (005):
 * the inline fix of a not-ready provider, the freshness of a test result,
 * and where the focus returns when the editor closes.
 * Run with `bun run test:prompt-settings`.
 */
import { describe, expect, test } from "bun:test";
import type { ListedPrompt, PostProcessProvider } from "@/bindings";
import { NEW_PROMPT_FOCUS, focusReturnOf } from "./prompt-library-model";
import {
  DEFAULT_PROVIDER_VALUE,
  type ProviderSettings,
  draftReadinessOf,
  fixFieldKeyOf,
  fixSettingOf,
  fixValueOf,
  providerFixChangeOf,
} from "./prompt-provider-model";
import { actionNoteKeyOf } from "./prompt-draft-model";
import { isResultOutdated } from "./prompt-test-model";

const provider = (id: string, baseUrl: string): PostProcessProvider => ({
  id,
  label: id.toUpperCase(),
  base_url: baseUrl,
});
const settings: ProviderSettings = {
  providers: [
    provider("openai", "https://api.openai.com/v1"),
    provider("custom", "http://localhost:11434/v1"),
  ],
  defaultProviderId: "custom",
  models: { custom: "llama3.2", openai: "gpt-4.1-mini" },
  apiKeys: {},
};
const listed = (id: string): ListedPrompt => ({
  id,
  name: "Shorten",
  prompt: "Shorten ${output}",
  provider_override: null,
  starter: false,
  issues: [],
  shortcuts: [],
  readiness: {
    provider_id: "custom",
    model: "llama3.2",
    is_override: false,
    ready: true,
    missing: null,
  },
});

// @spec AC-004: The inline fix sets the resolved provider, never the default — .specs/features/005-custom-ai-prompts/spec.md#ac-004
describe("inline provider fix", () => {
  const openaiDraft = { providerId: "openai", model: "gpt-4.1-mini" };
  const readiness = draftReadinessOf(settings, openaiDraft, false);

  test("the API key is saved for the prompt's provider, trimmed", () => {
    expect(readiness.missing).toBe("missing_api_key");
    expect(
      providerFixChangeOf(settings, "api_key", readiness, "  sk-test  "),
    ).toEqual({ fix: "api_key", providerId: "openai", value: "sk-test" });
  });

  test("the default provider choice fixes the default provider's own setting", () => {
    const viaDefault = draftReadinessOf(
      { ...settings, providers: [provider("custom", "")] },
      { providerId: DEFAULT_PROVIDER_VALUE, model: "" },
      false,
    );
    expect(viaDefault.missing).toBe("missing_endpoint");
    expect(
      providerFixChangeOf(settings, "endpoint", viaDefault, "http://x/v1"),
    ).toEqual({ fix: "endpoint", providerId: "custom", value: "http://x/v1" });
  });

  test("an empty or unchanged value saves nothing", () => {
    const keyed = { ...settings, apiKeys: { openai: "sk-test" } };
    expect(
      providerFixChangeOf(settings, "api_key", readiness, "  "),
    ).toBeNull();
    expect(
      providerFixChangeOf(keyed, "api_key", readiness, "sk-test"),
    ).toBeNull();
    expect(fixValueOf(keyed, "api_key", "openai")).toBe("sk-test");
    expect(fixValueOf(settings, "endpoint", "custom")).toBe(
      "http://localhost:11434/v1",
    );
  });
});

// @spec AC-007: A test result belongs to the draft it ran on — .specs/features/005-custom-ai-prompts/spec.md#ac-007
// @spec AC-004: The inline fix sets the resolved provider, never the default — .specs/features/005-custom-ai-prompts/spec.md#ac-004
describe("inline fix writes", () => {
  test("the endpoint fix writes the base URL alone, so the model is kept", () => {
    expect(fixSettingOf("endpoint")).toBe("base_url");
    expect(fixSettingOf("api_key")).toBe("api_key");
  });

  test("one fix field per missing item and provider", () => {
    expect(fixFieldKeyOf("api_key", "openai")).toBe("api_key:openai");
    expect(fixFieldKeyOf("api_key", "openai")).not.toBe(
      fixFieldKeyOf("api_key", "anthropic"),
    );
    expect(fixFieldKeyOf("api_key", "custom")).not.toBe(
      fixFieldKeyOf("endpoint", "custom"),
    );
  });
});

// @spec AC-001: Editor note and Save state — .specs/features/005-custom-ai-prompts/spec.md#ac-001
describe("action bar note", () => {
  test("'Fix the highlighted fields' only while a field shows an error", () => {
    const none = { nameError: null, textError: null };
    expect(actionNoteKeyOf(none)).toBe("note");
    expect(actionNoteKeyOf({ ...none, textError: "text_empty" })).toBe(
      "fixFields",
    );
    expect(actionNoteKeyOf({ ...none, nameError: "name_empty" })).toBe(
      "fixFields",
    );
  });
});

describe("test result freshness", () => {
  const tested = {
    text: "Shorten ${output}",
    provider_override: { provider_id: "custom", model: "llama3.2" },
    sample: "hello",
  };

  test("no result, or the same draft and sample, is not outdated", () => {
    expect(isResultOutdated(null, tested)).toBe(false);
    expect(isResultOutdated(tested, { ...tested })).toBe(false);
  });

  test("a new text, provider, model or sample makes it outdated", () => {
    expect(isResultOutdated(tested, { ...tested, text: "x" })).toBe(true);
    expect(isResultOutdated(tested, { ...tested, sample: "bye" })).toBe(true);
    expect(
      isResultOutdated(tested, { ...tested, provider_override: null }),
    ).toBe(true);
    expect(
      isResultOutdated(tested, {
        ...tested,
        provider_override: { provider_id: "custom", model: "qwen" },
      }),
    ).toBe(true);
  });
});

describe("focus after the editor closes", () => {
  const prompts = [listed("p1")];

  test("the edited prompt's button takes the focus back", () => {
    expect(focusReturnOf("p1", prompts)).toBe("p1");
  });

  test("a new prompt or a deleted one returns to New prompt", () => {
    expect(focusReturnOf(null, prompts)).toBe(NEW_PROMPT_FOCUS);
    expect(focusReturnOf("gone", prompts)).toBe(NEW_PROMPT_FOCUS);
  });
});
