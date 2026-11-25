import React from "react";
import { useTranslation } from "react-i18next";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { LanguageSelector } from "../LanguageSelector";
import { TranslateToEnglish } from "../TranslateToEnglish";
import { useModelStore } from "../../../stores/modelStore";
import type { ModelInfo } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { selectedEngine } from "../transcription-engine/transcription-engine-options";

export const ModelSettingsCard: React.FC = () => {
  const { t } = useTranslation();
  const { currentModel, models } = useModelStore();
  const { getSetting } = useSettings();

  // @spec FR-010: Recognition controls follow the engine — .specs/features/024-transcription-engine/spec.md#fr-010
  if (selectedEngine(getSetting("transcription_engine")) === "chatgpt") {
    return (
      <SettingsGroup
        title={t("settings.modelSettings.title", {
          model: t("settings.models.engine.providers.chatgpt.title"),
        })}
      >
        <LanguageSelector descriptionMode="tooltip" grouped />
      </SettingsGroup>
    );
  }

  const currentModelInfo = models.find((m: ModelInfo) => m.id === currentModel);

  const showLanguageSelector =
    currentModelInfo?.supports_language_selection ?? false;
  const supportsTranslation = currentModelInfo?.supports_translation ?? false;
  const hasAnySettings = showLanguageSelector || supportsTranslation;

  // Don't render anything if no model is selected or no settings available
  if (!currentModel || !currentModelInfo || !hasAnySettings) {
    return null;
  }

  return (
    <SettingsGroup
      title={t("settings.modelSettings.title", {
        model: currentModelInfo.name,
      })}
    >
      {showLanguageSelector && (
        <LanguageSelector
          descriptionMode="tooltip"
          grouped={true}
          supportedLanguages={currentModelInfo.supported_languages}
          supportsLanguageDetection={
            currentModelInfo.supports_language_detection
          }
        />
      )}
      {supportsTranslation && (
        <TranslateToEnglish descriptionMode="tooltip" grouped={true} />
      )}
    </SettingsGroup>
  );
};
