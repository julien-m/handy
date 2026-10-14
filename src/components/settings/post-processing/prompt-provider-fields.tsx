// Provider override of the prompt editor (005 AC-004): the default provider
// or another provider with its model (Apple Intelligence only where it is
// available), and the readiness of that choice with the item to fix
// (`prompt-provider-fix.tsx`).
import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { Dropdown } from "../../ui/Dropdown";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import { PROMPT_KEYS } from "./prompt-library-model";
import { ReadinessNotice } from "./prompt-provider-fix";
import {
  type DraftReadiness,
  type ProviderSettings,
  APPLE_PROVIDER_ID,
  DEFAULT_PROVIDER_VALUE,
  canListModels,
  draftReadinessOf,
  modelForProvider,
  modelOptionsOf,
  providerLabelOf,
  providerOptionsOf,
} from "./prompt-provider-model";
import type { PromptEditorModel } from "./use-prompt-editor";

const KEY = `${PROMPT_KEYS}.editor`;

/** Provider configuration of the settings, for readiness and options. */
const useProviderSettings = (): ProviderSettings => {
  const { settings } = useSettings();
  return useMemo(
    () => ({
      providers: settings?.post_process_providers ?? [],
      defaultProviderId: settings?.post_process_provider_id ?? "",
      models: settings?.post_process_models ?? {},
      apiKeys: settings?.post_process_api_keys ?? {},
    }),
    [settings],
  );
};

/** Whether Apple Intelligence can run here; asked once, only if offered. */
const useAppleAvailable = (settings: ProviderSettings): boolean => {
  const [available, setAvailable] = useState(false);
  const offered = settings.providers.some((p) => p.id === APPLE_PROVIDER_ID);
  useEffect(() => {
    if (!offered) return;
    let isCurrent = true;
    const check = async () => {
      const result = await commands.checkAppleIntelligenceAvailable();
      if (isCurrent) setAvailable(result);
    };
    check().catch((failure) => {
      // Unknown availability: Apple Intelligence is not offered.
      console.error("Failed to check Apple Intelligence:", failure);
    });
    return () => {
      isCurrent = false;
    };
  }, [offered]);
  return available;
};

/** Lists the models of the chosen provider when it can answer. */
const useModelListing = (settings: ProviderSettings, providerId: string) => {
  const { fetchPostProcessModels, postProcessModelOptions } = useSettings();
  const canList = canListModels(settings, providerId);
  useEffect(() => {
    if (providerId === DEFAULT_PROVIDER_VALUE || !canList) return;
    fetchPostProcessModels(providerId).catch((failure) => {
      // The model stays typeable by hand when the listing fails.
      console.error("Failed to list provider models:", failure);
    });
  }, [providerId, canList, fetchPostProcessModels]);
  return postProcessModelOptions[providerId];
};

interface ControlProps {
  testId: string;
  model: PromptEditorModel;
  settings: ProviderSettings;
  readiness: DraftReadiness;
  appleAvailable: boolean;
}

/** Provider dropdown: the default provider, then every offered provider. */
const ProviderControl: React.FC<ControlProps> = (props) => {
  const { testId, model, settings, appleAvailable } = props;
  const { t } = useTranslation();
  const choice = model.draft.provider;
  const defaultLabel = t(`${KEY}.defaultProviderOption`, {
    provider: providerLabelOf(settings.providers, settings.defaultProviderId),
  });
  const options = providerOptionsOf(
    settings.providers,
    appleAvailable,
    choice.providerId,
    defaultLabel,
  );
  const select = (providerId: string) =>
    model.setProvider({
      providerId,
      model: modelForProvider(settings, providerId),
    });
  return (
    <LabeledControl
      testId={`${testId}-form-provider-label`}
      label={t(`${KEY}.provider`)}
    >
      <Dropdown
        options={options}
        selectedValue={choice.providerId}
        onSelect={select}
        ariaLabel={t(`${KEY}.provider`)}
        testId={`${testId}-form-provider`}
      />
    </LabeledControl>
  );
};

/** Model of the override; the default provider's model, read-only. */
const ModelControl: React.FC<ControlProps> = (props) => {
  const { testId, model, settings, readiness } = props;
  const { t } = useTranslation();
  const choice = model.draft.provider;
  const isDefault = choice.providerId === DEFAULT_PROVIDER_VALUE;
  const listed = useModelListing(settings, choice.providerId);
  const setModel = (value: string) =>
    model.setProvider({ ...choice, model: value });
  return (
    <LabeledControl
      testId={`${testId}-form-model-label`}
      label={t(`${KEY}.model`)}
    >
      <div data-testid={`${testId}-form-model`}>
        <ModelSelect
          value={isDefault ? readiness.model : choice.model}
          options={modelOptionsOf(listed, choice.model)}
          disabled={isDefault}
          placeholder={t(`${KEY}.modelPlaceholder`)}
          ariaLabel={t(`${KEY}.model`)}
          onSelect={setModel}
          onCreate={setModel}
          onBlur={() => undefined}
          className="w-full"
        />
      </div>
    </LabeledControl>
  );
};

// @spec AC-004: Per-prompt provider and model override — .specs/features/005-custom-ai-prompts/spec.md#ac-004
/** Provider and model of the prompt, with readiness. */
export const PromptProviderFields: React.FC<{
  testId: string;
  model: PromptEditorModel;
}> = ({ testId, model }) => {
  const { t } = useTranslation();
  const settings = useProviderSettings();
  const appleAvailable = useAppleAvailable(settings);
  const readiness = draftReadinessOf(
    settings,
    model.draft.provider,
    appleAvailable,
  );
  const controlProps = { testId, model, settings, readiness, appleAvailable };
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-4">
        <ProviderControl {...controlProps} />
        <ModelControl {...controlProps} />
      </div>
      <ReadinessNotice
        testId={testId}
        readiness={readiness}
        settings={settings}
      />
      {model.providerError && (
        <p
          role="alert"
          className="text-sm text-red-400"
          data-testid={`${testId}-form-provider-error`}
        >
          {t(model.providerError)}
        </p>
      )}
    </div>
  );
};

interface LabeledControlProps {
  testId: string;
  label: string;
  children: React.ReactNode;
}

/** A form label above its control, sharing the row with its sibling. */
const LabeledControl: React.FC<LabeledControlProps> = ({
  testId,
  label,
  children,
}) => (
  <div className="flex flex-col gap-2 flex-1 min-w-[200px]">
    <span className="text-sm font-semibold" data-testid={testId}>
      {label}
    </span>
    {children}
  </div>
);
