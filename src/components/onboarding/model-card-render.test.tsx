// LiveSpec traceability: @spec(AC-009)
/**
 * Rendered model card (024): a card disabled because an API engine is active is
 * announced as disabled and not clickable, and the active card shows the
 * fallback label instead of "Active". Run with `bun run test:transcription-engine`.
 */
import { describe, expect, mock, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { ModelInfo } from "@/bindings";

const t = (key: string): string => key;
mock.module("react-i18next", () => ({
  useTranslation: () => ({ t, i18n: { language: "en" } }),
}));
mock.module("@/stores/settingsStore", () => ({
  useSettingsStore: (select: (state: { settings: null }) => unknown) =>
    select({ settings: null }),
}));

const { default: ModelCard } = await import("./ModelCard");

/** A downloaded model with only the fields the card reads given meaningful values. */
const buildModel = (overrides: Partial<ModelInfo> = {}): ModelInfo =>
  ({
    id: "canary",
    name: "Canary 1B v2",
    description: "25-language speech-to-text",
    filename: "canary-1b-v2-Q5_K_M.gguf",
    size_mb: 797,
    supported_languages: ["en", "fr"],
    is_custom: false,
    is_recommended: false,
    is_downloaded: true,
    accuracy_score: 0.9,
    speed_score: 0.8,
    source: { Catalog: "canary" },
    ...overrides,
  }) as unknown as ModelInfo;

const noop = () => undefined;

describe("ModelCard with an API engine active", () => {
  test("should announce a downloaded card as disabled and not clickable", () => {
    const markup = renderToStaticMarkup(
      <ModelCard
        model={buildModel()}
        status="available"
        disabled
        onSelect={noop}
      />,
    );
    expect(markup).toContain('aria-disabled="true"');
    expect(markup).toContain("cursor-not-allowed");
  });

  test("should label the active card as the fallback", () => {
    const markup = renderToStaticMarkup(
      <ModelCard
        model={buildModel()}
        status="active"
        disabled
        activeLabel="settings.models.engine.fallbackBadge"
        onSelect={noop}
      />,
    );
    expect(markup).toContain("settings.models.engine.fallbackBadge");
    expect(markup).not.toContain("modelSelector.active");
  });
});
