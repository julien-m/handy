// The Post Process settings page: AI post-processing switch and default
// prompt, prompt library and editor (005), AI dictation shortcut (004) and
// the OpenAI-compatible provider settings.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";

import { Alert } from "../../ui/Alert";
import { SettingContainer, SettingsGroup } from "@/components/ui";
import { ResetButton } from "../../ui/ResetButton";

import { ProviderSelect } from "../PostProcessingSettingsApi/ProviderSelect";
import { BaseUrlField } from "../PostProcessingSettingsApi/BaseUrlField";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import { usePostProcessProviderState } from "../PostProcessingSettingsApi/usePostProcessProviderState";
import { ShortcutList } from "../shortcut-list";
import type { EditorTarget } from "./prompt-draft-model";
import { PromptEditor } from "./prompt-editor";
import { focusReturnOf } from "./prompt-library-model";
import { PromptListGroup } from "./prompt-list";
import { PromptProcessingGroup } from "./prompt-processing-group";
import {
  type PromptLibraryState,
  usePromptLibrary,
} from "./use-prompt-library";

const PostProcessingSettingsApiComponent: React.FC = () => {
  const { t } = useTranslation();
  const state = usePostProcessProviderState();

  return (
    <>
      <SettingContainer
        title={t("settings.postProcessing.api.provider.title")}
        description={t("settings.postProcessing.api.provider.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <div className="flex items-center gap-2">
          <ProviderSelect
            options={state.providerOptions}
            value={state.selectedProviderId}
            onChange={state.handleProviderSelect}
          />
        </div>
      </SettingContainer>

      {state.isAppleProvider ? (
        state.appleIntelligenceUnavailable ? (
          <Alert variant="error" contained>
            {t("settings.postProcessing.api.appleIntelligence.unavailable")}
          </Alert>
        ) : null
      ) : (
        <>
          {state.selectedProvider?.id === "custom" && (
            <SettingContainer
              title={t("settings.postProcessing.api.baseUrl.title")}
              description={t("settings.postProcessing.api.baseUrl.description")}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <div className="flex items-center gap-2">
                <BaseUrlField
                  value={state.baseUrl}
                  onBlur={state.handleBaseUrlChange}
                  placeholder={t(
                    "settings.postProcessing.api.baseUrl.placeholder",
                  )}
                  disabled={state.isBaseUrlUpdating}
                  className="min-w-[380px]"
                />
              </div>
            </SettingContainer>
          )}

          <SettingContainer
            title={t("settings.postProcessing.api.apiKey.title")}
            description={t("settings.postProcessing.api.apiKey.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div className="flex items-center gap-2">
              <ApiKeyField
                value={state.apiKey}
                onBlur={state.handleApiKeyChange}
                placeholder={t(
                  "settings.postProcessing.api.apiKey.placeholder",
                )}
                disabled={state.isApiKeyUpdating}
                className="min-w-[320px]"
              />
            </div>
          </SettingContainer>
        </>
      )}

      {!state.isAppleProvider && (
        <SettingContainer
          title={t("settings.postProcessing.api.model.title")}
          description={
            state.isCustomProvider
              ? t("settings.postProcessing.api.model.descriptionCustom")
              : t("settings.postProcessing.api.model.descriptionDefault")
          }
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <div className="flex items-center gap-2">
            <ModelSelect
              value={state.model}
              options={state.modelOptions}
              disabled={state.isModelUpdating}
              isLoading={state.isFetchingModels}
              placeholder={
                state.modelOptions.length > 0
                  ? t(
                      "settings.postProcessing.api.model.placeholderWithOptions",
                    )
                  : t("settings.postProcessing.api.model.placeholderNoOptions")
              }
              onSelect={state.handleModelSelect}
              onCreate={state.handleModelCreate}
              onBlur={() => {}}
              className="flex-1 min-w-[380px]"
            />
            <ResetButton
              onClick={state.handleRefreshModels}
              disabled={state.isFetchingModels}
              ariaLabel={t("settings.postProcessing.api.model.refreshModels")}
              className="flex h-10 w-10 items-center justify-center"
            >
              <RefreshCcw
                className={`h-4 w-4 ${state.isFetchingModels ? "animate-spin" : ""}`}
              />
            </ResetButton>
          </div>
        </SettingContainer>
      )}
    </>
  );
};

export const PostProcessingSettingsApi = React.memo(
  PostProcessingSettingsApiComponent,
);
PostProcessingSettingsApi.displayName = "PostProcessingSettingsApi";

/**
 * Which editor is open, and which one closed last (its button takes the
 * focus back when the list returns).
 */
const useEditorTarget = () => {
  const [target, setTarget] = useState<EditorTarget | null>(null);
  const [closed, setClosed] = useState<EditorTarget | null>(null);
  const open = (next: EditorTarget) => {
    setClosed(null);
    setTarget(next);
  };
  const close = () => {
    setClosed(target);
    setTarget(null);
  };
  return { target, closed, open, close };
};

/** The prompt groups, the AI dictation shortcut and the provider settings. */
const PromptSettingsPage: React.FC<{
  libraryState: PromptLibraryState;
  focusId: string | null;
  open: (target: EditorTarget) => void;
}> = ({ libraryState, focusId, open }) => {
  const { t } = useTranslation();
  return (
    <div
      className="max-w-3xl w-full mx-auto space-y-6"
      data-testid="prompt-settings-screen"
    >
      <PromptProcessingGroup library={libraryState.library} />
      <PromptListGroup
        state={libraryState}
        focusId={focusId}
        onEdit={(id) => open({ mode: "edit", id })}
        onNew={() => open({ mode: "new" })}
      />
      <SettingsGroup title={t("settings.postProcessing.hotkey.title")}>
        <ShortcutList
          actionId="transcribe_with_post_process"
          descriptionMode="tooltip"
          grouped={true}
        />
      </SettingsGroup>
      <SettingsGroup title={t("settings.postProcessing.api.title")}>
        <PostProcessingSettingsApi />
      </SettingsGroup>
    </div>
  );
};

// @spec AC-001: Prompt list and editor in the settings — .specs/features/005-custom-ai-prompts/spec.md#ac-001
/**
 * The Post Process page: the AI post-processing switch and default prompt,
 * the prompt library (or the editor of one prompt), the AI dictation
 * shortcut and the provider settings.
 */
export const PostProcessingSettings: React.FC = () => {
  const libraryState = usePromptLibrary();
  const { target, closed, open, close } = useEditorTarget();
  if (target) {
    return (
      <PromptEditor
        target={target}
        libraryState={libraryState}
        onCreated={(id) => open({ mode: "edit", id })}
        onClose={close}
      />
    );
  }
  const focusId = closed
    ? focusReturnOf(
        closed.mode === "edit" ? closed.id : null,
        libraryState.library?.prompts ?? [],
      )
    : null;
  return (
    <PromptSettingsPage
      libraryState={libraryState}
      focusId={focusId}
      open={open}
    />
  );
};
