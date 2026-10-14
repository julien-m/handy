// Provider override of the prompt editor (005 AC-004): the default provider
// or another provider with its model (Apple Intelligence only where it is
// available), and the readiness of that choice with the item to fix.
import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { Dropdown } from "../../ui/Dropdown";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import { PROMPT_KEYS, reasonKey } from "./prompt-library-model";
import {
  type DraftReadiness,
  type ProviderSettings,
  type ReadinessFix,
  APPLE_PROVIDER_ID,
  CUSTOM_PROVIDER_ID,
  DEFAULT_PROVIDER_VALUE,
  canListModels,
  draftReadinessOf,
  modelForProvider,
  modelOptionsOf,
  providerLabelOf,
  providerOptionsOf,
  readinessFixOf,
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

interface FieldsProps {
  testId: string;
  model: PromptEditorModel;
  onFix: (fix: ReadinessFix) => void;
}

interface ReadinessProps {
  testId: string;
  readiness: DraftReadiness;
  onFix: (fix: ReadinessFix) => void;
}

/** "Ready" line, or what is missing and where to fix it. */
const ReadinessNotice: React.FC<ReadinessProps> = ({
  testId,
  readiness,
  onFix,
}) => {
  const { t } = useTranslation();
  const label = readiness.provider?.label ?? readiness.providerId;
  if (!readiness.missing) {
    const isCustom = readiness.providerId === CUSTOM_PROVIDER_ID;
    return (
      <p
        className="text-sm text-mid-gray"
        data-testid={`${testId}-form-endpoint`}
      >
        {isCustom
          ? t(`${KEY}.readyEndpoint`, {
              endpoint: readiness.provider?.base_url,
            })
          : t(`${KEY}.readyProvider`, {
              provider: label,
              model: readiness.model,
            })}
      </p>
    );
  }
  const fix = readinessFixOf(readiness.missing);
  return (
    <Alert
      variant="warning"
      role="alert"
      testId={`${testId}-form-not-ready`}
      action={
        fix && (
          <Button
            variant="warning"
            size="md"
            onClick={() => onFix(fix)}
            data-testid={`${testId}-form-fix`}
          >
            {t(`${KEY}.fix.${fix}`)}
          </Button>
        )
      }
    >
      <span className="block" data-testid={`${testId}-form-not-ready-title`}>
        {t(`${KEY}.notReadyTitle`, {
          provider: label,
          reason: t(reasonKey(readiness.missing)),
        })}
      </span>
      <span className="block">{t(`${KEY}.notReadyBody`)}</span>
    </Alert>
  );
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

// @spec AC-004: Per-prompt provider and model override — .specs/features/005-custom-ai-prompts/spec.md#ac-004
/** Provider and model of the prompt, with readiness. */
export const PromptProviderFields: React.FC<FieldsProps> = ({
  testId,
  model,
  onFix,
}) => {
  const { t } = useTranslation();
  const settings = useProviderSettings();
  const appleAvailable = useAppleAvailable(settings);
  const choice = model.draft.provider;
  const isDefault = choice.providerId === DEFAULT_PROVIDER_VALUE;
  const readiness = draftReadinessOf(settings, choice, appleAvailable);
  const listed = useModelListing(settings, choice.providerId);
  const defaultLabel = t(`${KEY}.defaultProviderOption`, {
    provider: providerLabelOf(settings.providers, settings.defaultProviderId),
  });
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-4">
        <LabeledControl
          testId={`${testId}-form-provider-label`}
          label={t(`${KEY}.provider`)}
        >
          <Dropdown
            options={providerOptionsOf(
              settings.providers,
              appleAvailable,
              choice.providerId,
              defaultLabel,
            )}
            selectedValue={choice.providerId}
            onSelect={(providerId) =>
              model.setProvider({
                providerId,
                model: modelForProvider(settings, providerId),
              })
            }
            ariaLabel={t(`${KEY}.provider`)}
            testId={`${testId}-form-provider`}
          />
        </LabeledControl>
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
              onSelect={(value) =>
                model.setProvider({ ...choice, model: value })
              }
              onCreate={(value) =>
                model.setProvider({ ...choice, model: value })
              }
              onBlur={() => undefined}
              className="w-full"
            />
          </div>
        </LabeledControl>
      </div>
      <ReadinessNotice testId={testId} readiness={readiness} onFix={onFix} />
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
