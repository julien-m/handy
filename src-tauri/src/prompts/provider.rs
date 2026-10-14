//! Provider resolution and readiness of a prompt (005 AC-004, FR-003): the prompt's override or
//! the default provider with its model, and the first missing item that keeps it from
//! processing. Pure, except the Apple Intelligence availability, which goes through an injected
//! port and is only asked inside the running instance, never at startup.
use super::library;
use crate::settings::{
    AppSettings, LLMPrompt, PostProcessProvider, APPLE_INTELLIGENCE_PROVIDER_ID,
};
use serde::Serialize;
use specta::Type;

/// Why a prompt cannot process right now. Never carries a credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessIssue {
    /// The provider has no base URL.
    MissingEndpoint,
    /// No model is configured for the provider (or in the override).
    MissingModel,
    /// The provider needs an API key and has none.
    MissingApiKey,
    /// Apple Intelligence is not available on this device right now.
    ProviderUnavailable,
    /// The provider does not exist in this build.
    UnknownProvider,
    /// The prompt has validation issues and must be edited first.
    InvalidPrompt,
    /// The prompt is no longer in the library.
    PromptMissing,
}

/// An API key, never printed: `Debug` is redacted and no serializer exists.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ApiKey(String);
impl ApiKey {
    /// The key for the request header only.
    pub fn expose(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(if self.0.is_empty() {
            "ApiKey(none)"
        } else {
            "ApiKey(***)"
        })
    }
}

/// Provider, model and credential a prompt would use, frozen from one settings view.
#[derive(Clone, Debug)]
pub struct ProviderChoice {
    pub provider: PostProcessProvider,
    pub model: String,
    pub api_key: ApiKey,
    /// True when the prompt overrides the default provider.
    pub is_override: bool,
}

/// Public readiness of a prompt: which provider and model it uses and what is missing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Type)]
pub struct Readiness {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub is_override: bool,
    pub ready: bool,
    pub missing: Option<ReadinessIssue>,
}

/// Whether Apple Intelligence can run now (the native check is synchronous and may be slow).
pub trait AvailabilityProbe: Send + Sync {
    fn apple_intelligence_available(&self) -> bool;
}

/// The device's own Apple Intelligence check.
pub struct NativeAvailability;
impl AvailabilityProbe for NativeAvailability {
    fn apple_intelligence_available(&self) -> bool {
        native_apple_availability()
    }
}

/// The native check on a worker bounded by a timeout, one at a time (a hung check holds its
/// slot and later checks answer unavailable instead of piling up workers).
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_apple_availability() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    static IN_FLIGHT: AtomicBool = AtomicBool::new(false);
    // Upper bound for a check that normally answers in milliseconds.
    const CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);
    if IN_FLIGHT.swap(true, Ordering::AcqRel) {
        return false;
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let available = crate::apple_intelligence::check_apple_intelligence_availability();
        IN_FLIGHT.store(false, Ordering::Release);
        // The receiver may have timed out; the answer is then simply dropped.
        let _ = sender.send(available);
    });
    receiver.recv_timeout(CHECK_TIMEOUT).unwrap_or(false)
}
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn native_apple_availability() -> bool {
    false
}

/// Providers whose requests need no API key: the local custom endpoint (its key is optional)
/// and Apple Intelligence (on device). Every other built-in provider needs one.
fn needs_api_key(provider_id: &str) -> bool {
    !matches!(provider_id, "custom" | APPLE_INTELLIGENCE_PROVIDER_ID)
}

/// Provider id and model of `prompt`: its override, else the default provider and its model.
fn provider_and_model(settings: &AppSettings, prompt: &LLMPrompt) -> (String, String, bool) {
    match &prompt.provider_override {
        Some(choice) => (choice.provider_id.clone(), choice.model.clone(), true),
        None => {
            let id = settings.post_process_provider_id.clone();
            let model = settings
                .post_process_models
                .get(&id)
                .cloned()
                .unwrap_or_default();
            (id, model, false)
        }
    }
}

// @spec FR-003: Resolve provider and report readiness — .specs/features/005-custom-ai-prompts/spec.md#fr-003
/// Provider choice of `prompt` from `settings`, without the Apple availability check.
///
/// # Errors
/// The first missing item: an invalid prompt, an unknown provider, a missing endpoint, model or
/// required API key.
pub fn resolve(
    settings: &AppSettings,
    prompt: &LLMPrompt,
) -> Result<ProviderChoice, ReadinessIssue> {
    if !library::is_usable(settings, prompt) {
        return Err(ReadinessIssue::InvalidPrompt);
    }
    let (provider_id, model, is_override) = provider_and_model(settings, prompt);
    let provider = settings
        .post_process_provider(&provider_id)
        .cloned()
        .ok_or(ReadinessIssue::UnknownProvider)?;
    let key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();
    check_configuration(&provider, &model, &key)?;
    Ok(ProviderChoice {
        provider,
        model: model.trim().to_owned(),
        api_key: ApiKey(key),
        is_override,
    })
}

/// The first missing item of a provider configuration: endpoint (HTTP providers), model, then
/// the API key of a provider that needs one.
fn check_configuration(
    provider: &PostProcessProvider,
    model: &str,
    key: &str,
) -> Result<(), ReadinessIssue> {
    if provider.id != APPLE_INTELLIGENCE_PROVIDER_ID && provider.base_url.trim().is_empty() {
        return Err(ReadinessIssue::MissingEndpoint);
    }
    if model.trim().is_empty() {
        return Err(ReadinessIssue::MissingModel);
    }
    if needs_api_key(&provider.id) && key.trim().is_empty() {
        return Err(ReadinessIssue::MissingApiKey);
    }
    Ok(())
}

/// Provider choice of `prompt`, checking Apple Intelligence availability through `probe`.
///
/// # Errors
/// Same as `resolve`, plus `ProviderUnavailable`.
pub fn resolve_ready(
    settings: &AppSettings,
    prompt: &LLMPrompt,
    probe: &dyn AvailabilityProbe,
) -> Result<ProviderChoice, ReadinessIssue> {
    let choice = resolve(settings, prompt)?;
    let is_apple = choice.provider.id == APPLE_INTELLIGENCE_PROVIDER_ID;
    if is_apple && !probe.apple_intelligence_available() {
        return Err(ReadinessIssue::ProviderUnavailable);
    }
    Ok(choice)
}

/// Public readiness of `prompt` (ready, or the missing item), never its credential.
pub fn readiness(
    settings: &AppSettings,
    prompt: &LLMPrompt,
    probe: &dyn AvailabilityProbe,
) -> Readiness {
    let (provider_id, model, is_override) = provider_and_model(settings, prompt);
    let missing = resolve_ready(settings, prompt, probe).err();
    Readiness {
        provider_id: Some(provider_id).filter(|id| !id.is_empty()),
        model: Some(model).filter(|model| !model.trim().is_empty()),
        is_override,
        ready: missing.is_none(),
        missing,
    }
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;
