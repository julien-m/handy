// Pure rules of a prompt's provider override in the editor (005 AC-004): the
// options offered, the override sent to the backend and the readiness of an
// unsaved choice, mirrored from `src-tauri/src/prompts/provider.rs` (the
// saved prompts' readiness comes from the backend listing).
import type {
  PostProcessProvider,
  PromptProviderOverride,
  ReadinessIssue,
  SecretMap,
} from "@/bindings";

/** Mirrors `APPLE_INTELLIGENCE_PROVIDER_ID` in src-tauri/src/settings.rs. */
export const APPLE_PROVIDER_ID = "apple_intelligence";
/** The local OpenAI-compatible endpoint, whose API key is optional. */
export const CUSTOM_PROVIDER_ID = "custom";
/** Dropdown value of "use the default provider" (no override). */
export const DEFAULT_PROVIDER_VALUE = "";

/** Provider configuration of the settings, never shown to the user as is. */
export interface ProviderSettings {
  providers: ReadonlyArray<PostProcessProvider>;
  defaultProviderId: string;
  models: Partial<Record<string, string>>;
  apiKeys: SecretMap;
}

/** Provider and model a draft overrides, as edited in the form. */
export interface ProviderDraft {
  /** `DEFAULT_PROVIDER_VALUE` for the default provider. */
  providerId: string;
  model: string;
}

/** Provider, model and first missing item of a provider choice. */
export interface DraftReadiness {
  provider: PostProcessProvider | null;
  providerId: string;
  model: string;
  missing: ReadinessIssue | null;
}

/** A provider option of the editor's provider dropdown. */
export interface ProviderOption {
  value: string;
  label: string;
}

/**
 * Display label of a provider id; an unknown id is shown as is.
 * @param providers Providers of the settings.
 * @param providerId Provider to name.
 */
export const providerLabelOf = (
  providers: ReadonlyArray<PostProcessProvider>,
  providerId: string,
): string =>
  providers.find((provider) => provider.id === providerId)?.label ?? providerId;

/**
 * Override sent to the backend: none for the default provider. A missing
 * model is kept so the backend refuses it with `prompt_override_incomplete`.
 * @param draft Provider choice of the form.
 */
export const overrideOf = (
  draft: ProviderDraft,
): PromptProviderOverride | null =>
  draft.providerId === DEFAULT_PROVIDER_VALUE
    ? null
    : { provider_id: draft.providerId, model: draft.model.trim() };

/**
 * Provider choice of the form for a saved override.
 * @param override Stored override, or null for the default provider.
 */
export const providerDraftOf = (
  override: PromptProviderOverride | null,
): ProviderDraft =>
  override
    ? { providerId: override.provider_id, model: override.model }
    : { providerId: DEFAULT_PROVIDER_VALUE, model: "" };

/**
 * Whether two provider choices are the same override.
 * @param left First choice.
 * @param right Second choice.
 */
export const sameProviderDraft = (
  left: ProviderDraft,
  right: ProviderDraft,
): boolean =>
  left.providerId === right.providerId &&
  (left.providerId === DEFAULT_PROVIDER_VALUE ||
    left.model.trim() === right.model.trim());

/**
 * Provider options of the editor: the default provider first, then every
 * provider. Apple Intelligence is offered only when available, or when the
 * prompt already uses it (so its current choice stays visible).
 * @param providers Providers of the settings.
 * @param appleAvailable Whether Apple Intelligence can run on this device.
 * @param currentId Provider the draft uses now.
 * @param defaultLabel Translated label of the default-provider option.
 */
export const providerOptionsOf = (
  providers: ReadonlyArray<PostProcessProvider>,
  appleAvailable: boolean,
  currentId: string,
  defaultLabel: string,
): ProviderOption[] => [
  { value: DEFAULT_PROVIDER_VALUE, label: defaultLabel },
  ...providers
    .filter(
      (provider) =>
        provider.id !== APPLE_PROVIDER_ID ||
        appleAvailable ||
        currentId === APPLE_PROVIDER_ID,
    )
    .map((provider) => ({ value: provider.id, label: provider.label })),
];

/** Providers whose requests need no API key, like `needs_api_key`. */
const needsApiKey = (providerId: string): boolean =>
  providerId !== CUSTOM_PROVIDER_ID && providerId !== APPLE_PROVIDER_ID;

/**
 * First missing item of a configuration, in the backend's order: endpoint
 * (HTTP providers), model, then a required API key.
 */
const missingConfiguration = (
  provider: PostProcessProvider,
  model: string,
  apiKey: string,
): ReadinessIssue | null => {
  if (provider.id !== APPLE_PROVIDER_ID && provider.base_url.trim() === "") {
    return "missing_endpoint";
  }
  if (model.trim() === "") return "missing_model";
  return needsApiKey(provider.id) && apiKey.trim() === ""
    ? "missing_api_key"
    : null;
};

// @spec AC-004: Provider readiness of the edited prompt — .specs/features/005-custom-ai-prompts/spec.md#ac-004
/**
 * Readiness of an unsaved provider choice: the override, or the default
 * provider with its model, and the first missing item. Apple Intelligence
 * availability is checked last, as `resolve_ready` does.
 * @param settings Provider configuration of the settings.
 * @param draft Provider choice of the form.
 * @param appleAvailable Whether Apple Intelligence can run on this device.
 */
export const draftReadinessOf = (
  settings: ProviderSettings,
  draft: ProviderDraft,
  appleAvailable: boolean,
): DraftReadiness => {
  const isDefault = draft.providerId === DEFAULT_PROVIDER_VALUE;
  const providerId = isDefault ? settings.defaultProviderId : draft.providerId;
  const model = isDefault ? (settings.models[providerId] ?? "") : draft.model;
  const provider =
    settings.providers.find((candidate) => candidate.id === providerId) ?? null;
  const base = { provider, providerId, model: model.trim() };
  if (!provider) return { ...base, missing: "unknown_provider" };
  const missing = missingConfiguration(
    provider,
    model,
    settings.apiKeys[providerId] ?? "",
  );
  if (missing) return { ...base, missing };
  const unavailable = providerId === APPLE_PROVIDER_ID && !appleAvailable;
  return { ...base, missing: unavailable ? "provider_unavailable" : null };
};

/** Where the user fixes a missing item, when it is outside the editor. */
export type ReadinessFix = "api_key" | "endpoint" | null;

/**
 * Fix offered next to a not-ready provider: the API key and the custom
 * endpoint are set in the provider settings of the page; a missing model is
 * chosen in the editor itself, and the other items have no fix there.
 * @param missing First missing item.
 */
export const readinessFixOf = (
  missing: ReadinessIssue | null,
): ReadinessFix => {
  if (missing === "missing_api_key") return "api_key";
  return missing === "missing_endpoint" ? "endpoint" : null;
};

/**
 * Model prefilled when the draft switches to `providerId`: the model that
 * provider is configured with, else none.
 * @param settings Provider configuration of the settings.
 * @param providerId Provider chosen in the form.
 */
export const modelForProvider = (
  settings: ProviderSettings,
  providerId: string,
): string => settings.models[providerId] ?? "";

/**
 * Whether the models of `providerId` can be listed now: the custom endpoint
 * once it has a URL, a remote provider once it has a key, never Apple.
 * @param settings Provider configuration of the settings.
 * @param providerId Provider chosen in the form.
 */
export const canListModels = (
  settings: ProviderSettings,
  providerId: string,
): boolean => {
  if (providerId === APPLE_PROVIDER_ID) return false;
  if (providerId === CUSTOM_PROVIDER_ID) {
    const provider = settings.providers.find((p) => p.id === providerId);
    return (provider?.base_url ?? "").trim() !== "";
  }
  return (settings.apiKeys[providerId] ?? "").trim() !== "";
};

/**
 * Model options: the listed models, with the current model kept first when
 * the listing does not contain it (a custom model name).
 * @param listed Models listed for the provider, if any.
 * @param current Model of the draft.
 */
export const modelOptionsOf = (
  listed: ReadonlyArray<string> | undefined,
  current: string,
): ProviderOption[] => {
  const models = listed ? [...listed] : [];
  const trimmed = current.trim();
  if (trimmed !== "" && !models.includes(trimmed)) models.unshift(trimmed);
  return models.map((model) => ({ value: model, label: model }));
};
