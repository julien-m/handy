// Readiness of the edited prompt's provider (005 AC-004, Story 3 "how to fix
// it"): "Ready", or the missing item with an inline field that sets the API
// key or the endpoint of the provider the prompt resolves to. The draft stays
// in the editor and the default provider never changes (keys and endpoints
// are stored per provider). The contract navigates to the page's provider
// settings instead: see implementation.md.
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../../hooks/useSettings";
import { Alert } from "../../ui/Alert";
import { Button } from "../../ui/Button";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { BaseUrlField } from "../PostProcessingSettingsApi/BaseUrlField";
import { PROMPT_KEYS, reasonKey } from "./prompt-library-model";
import {
  type DraftReadiness,
  type ProviderFixChange,
  type ProviderSettings,
  CUSTOM_PROVIDER_ID,
  fixValueOf,
  providerFixChangeOf,
  readinessFixOf,
} from "./prompt-provider-model";
import { useFocusOnMount } from "./use-focus-on-mount";

const KEY = `${PROMPT_KEYS}.editor`;
const API_KEY = "settings.postProcessing.api";

interface ReadinessProps {
  testId: string;
  readiness: DraftReadiness;
  settings: ProviderSettings;
}

/** "Ready" line with the endpoint or the provider and model. */
const ReadyLine: React.FC<ReadinessProps> = ({ testId, readiness }) => {
  const { t } = useTranslation();
  const isCustom = readiness.providerId === CUSTOM_PROVIDER_ID;
  const text = isCustom
    ? t(`${KEY}.readyEndpoint`, { endpoint: readiness.provider?.base_url })
    : t(`${KEY}.readyProvider`, {
        provider: readiness.provider?.label ?? readiness.providerId,
        model: readiness.model,
      });
  return (
    <p
      className="text-sm text-mid-gray"
      data-testid={`${testId}-form-endpoint`}
    >
      {text}
    </p>
  );
};

/** Saves a fix to the resolved provider's own setting. */
const useSaveFix = (): ((change: ProviderFixChange | null) => void) => {
  const { updatePostProcessApiKey, updatePostProcessBaseUrl } = useSettings();
  return (change) => {
    if (!change) return;
    const save =
      change.fix === "api_key"
        ? updatePostProcessApiKey
        : updatePostProcessBaseUrl;
    // The store reports a refused write itself (translated toast).
    void save(change.providerId, change.value);
  };
};

interface FixFieldProps extends ReadinessProps {
  fix: "api_key" | "endpoint";
}

/** Props of the existing API key / base URL field for the resolved provider. */
const useFixInputProps = ({ readiness, settings, fix }: FixFieldProps) => {
  const { t } = useTranslation();
  const { isUpdating } = useSettings();
  const saveFix = useSaveFix();
  const setting = fix === "api_key" ? "api_key" : "base_url";
  const placeholder = fix === "api_key" ? "apiKey" : "baseUrl";
  return {
    value: fixValueOf(settings, fix, readiness.providerId),
    onBlur: (typed: string) =>
      saveFix(providerFixChangeOf(settings, fix, readiness, typed)),
    disabled: isUpdating(`post_process_${setting}:${readiness.providerId}`),
    placeholder: t(`${API_KEY}.${placeholder}.placeholder`),
    className: "w-full",
  };
};

/**
 * Inline field of the API key or endpoint, saved when it is left; the
 * label element wraps the field so it names it.
 */
const ProviderFixField: React.FC<FixFieldProps> = (props) => {
  const { testId, readiness, fix } = props;
  const { t } = useTranslation();
  const inputProps = useFixInputProps(props);
  const focusRef = useFocusOnMount<HTMLDivElement>();
  const provider = readiness.provider?.label ?? readiness.providerId;
  const Field = fix === "api_key" ? ApiKeyField : BaseUrlField;
  return (
    <div
      ref={focusRef}
      className="space-y-2"
      data-testid={`${testId}-form-fix-field`}
    >
      <label className="flex flex-col gap-2 text-sm font-semibold">
        {t(`${KEY}.fixField.${fix}`, { provider })}
        <Field {...inputProps} />
      </label>
      <p className="text-xs text-mid-gray/70">{t(`${KEY}.fixField.hint`)}</p>
    </div>
  );
};

/** Opens the inline field of the missing item. */
const FixButton: React.FC<{
  testId: string;
  fix: string;
  onOpen: () => void;
}> = ({ testId, fix, onOpen }) => {
  const { t } = useTranslation();
  return (
    <Button
      variant="warning"
      size="md"
      onClick={onOpen}
      data-testid={`${testId}-form-fix`}
    >
      {t(`${KEY}.fix.${fix}`)}
    </Button>
  );
};

/** What is missing, and the button that opens its inline field. */
const NotReadyAlert: React.FC<ReadinessProps> = (props) => {
  const { testId, readiness } = props;
  const { t } = useTranslation();
  const [isFixing, setIsFixing] = useState(false);
  const fix = readinessFixOf(readiness.missing);
  const provider = readiness.provider?.label ?? readiness.providerId;
  const reason = t(reasonKey(readiness.missing ?? "unknown_provider"));
  const button = fix && !isFixing && (
    <FixButton testId={testId} fix={fix} onOpen={() => setIsFixing(true)} />
  );
  return (
    <div className="space-y-3">
      <Alert
        variant="warning"
        role="alert"
        testId={`${testId}-form-not-ready`}
        action={button}
      >
        <span className="block" data-testid={`${testId}-form-not-ready-title`}>
          {t(`${KEY}.notReadyTitle`, { provider, reason })}
        </span>
        <span className="block">{t(`${KEY}.notReadyBody`)}</span>
      </Alert>
      {fix && isFixing && <ProviderFixField {...props} fix={fix} />}
    </div>
  );
};

// @spec AC-004: Provider readiness and its fix in the editor — .specs/features/005-custom-ai-prompts/spec.md#ac-004
/** "Ready" line, or what is missing and its inline fix. */
export const ReadinessNotice: React.FC<ReadinessProps> = (props) =>
  props.readiness.missing ? (
    <NotReadyAlert {...props} />
  ) : (
    <ReadyLine {...props} />
  );
