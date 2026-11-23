// LiveSpec traceability: @spec(AC-009)
/**
 * Rendered footer model dropdown (024): while an API engine is active the
 * entries are announced as disabled, the selected model reads as the fallback
 * and a click does not switch the model. Run with `bun run test:transcription-engine`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ModelInfo } from "@/bindings";

const t = (key: string): string => key;
mock.module("react-i18next", () => ({
  useTranslation: () => ({ t, i18n: { language: "en" } }),
}));

const { default: ModelDropdown } = await import("./ModelDropdown");

const model = {
  id: "canary",
  name: "Canary 1B v2",
  description: "25-language speech-to-text",
  is_downloaded: true,
  is_custom: false,
  supports_streaming: false,
} as unknown as ModelInfo;

describe("ModelDropdown with an API engine active", () => {
  test("should announce entries as disabled and label the model as fallback", () => {
    const markup = renderToStaticMarkup(
      <ModelDropdown
        models={[model]}
        currentModelId="canary"
        onModelSelect={() => undefined}
        isLocked
      />,
    );
    expect(markup).toContain('aria-disabled="true"');
    expect(markup).toContain("settings.models.engine.fallbackBadge");
    expect(markup).toContain("model-dropdown-fallback-notice");
    expect(markup).not.toContain("modelSelector.active");
  });

  test("should keep the active label and no notice for the local engine", () => {
    const markup = renderToStaticMarkup(
      <ModelDropdown
        models={[model]}
        currentModelId="canary"
        onModelSelect={() => undefined}
      />,
    );
    expect(markup).toContain("modelSelector.active");
    expect(markup).not.toContain("aria-disabled");
  });
});
