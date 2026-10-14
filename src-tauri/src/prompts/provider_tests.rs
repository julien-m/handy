//! Readiness rules per provider family, default versus override, and Apple availability through
//! the injected probe; no credential appears in any readiness or debug output.
use super::*;
use crate::settings::{get_default_settings, PromptProviderOverride};

struct Apple(bool);
impl AvailabilityProbe for Apple {
    fn apple_intelligence_available(&self) -> bool {
        self.0
    }
}
const SECRET: &str = "sk-SENTINEL-005";

fn settings_with_prompt(choice: Option<PromptProviderOverride>) -> (AppSettings, LLMPrompt) {
    let mut settings = get_default_settings();
    let mut prompt = LLMPrompt::new("p".into(), "Notes".into(), "Notes: ${output}".into());
    prompt.provider_override = choice;
    settings.post_process_prompts = vec![prompt.clone()];
    (settings, prompt)
}
fn custom(model: &str) -> Option<PromptProviderOverride> {
    Some(PromptProviderOverride {
        provider_id: "custom".into(),
        model: model.into(),
    })
}
fn missing(settings: &AppSettings, prompt: &LLMPrompt) -> Option<ReadinessIssue> {
    readiness(settings, prompt, &Apple(true)).missing
}

#[test]
fn the_default_remote_provider_needs_a_model_and_a_key() {
    let (mut settings, prompt) = settings_with_prompt(None);
    settings.post_process_provider_id = "openai".into();
    assert_eq!(
        missing(&settings, &prompt),
        Some(ReadinessIssue::MissingModel)
    );
    settings
        .post_process_models
        .insert("openai".into(), "gpt-4o-mini".into());
    assert_eq!(
        missing(&settings, &prompt),
        Some(ReadinessIssue::MissingApiKey)
    );
    settings
        .post_process_api_keys
        .insert("openai".into(), SECRET.into());
    let ready = readiness(&settings, &prompt, &Apple(false));
    assert!(ready.ready && !ready.is_override);
    assert_eq!(ready.provider_id.as_deref(), Some("openai"));
    assert_eq!(ready.model.as_deref(), Some("gpt-4o-mini"));
}
#[test]
fn the_local_custom_override_needs_an_endpoint_and_a_model_but_no_key() {
    let (mut settings, prompt) = settings_with_prompt(custom("llama3.2"));
    let ready = readiness(&settings, &prompt, &Apple(false));
    assert!(ready.ready && ready.is_override);
    assert_eq!(ready.provider_id.as_deref(), Some("custom"));
    if let Some(provider) = settings.post_process_provider_mut("custom") {
        provider.base_url = " ".into();
    }
    assert_eq!(
        missing(&settings, &prompt),
        Some(ReadinessIssue::MissingEndpoint)
    );
}
#[test]
fn an_override_wins_over_the_default_provider_model() {
    let (mut settings, prompt) = settings_with_prompt(custom("override-model"));
    settings
        .post_process_models
        .insert("custom".into(), "default-model".into());
    let choice = resolve(&settings, &prompt).map(|choice| choice.model);
    assert_eq!(choice, Ok("override-model".to_string()));
}
#[test]
fn unknown_and_invalid_prompts_are_not_ready() {
    let (settings, prompt) = settings_with_prompt(Some(PromptProviderOverride {
        provider_id: "gone".into(),
        model: "m".into(),
    }));
    // The unknown override provider is a validation issue of the prompt itself.
    assert_eq!(
        missing(&settings, &prompt),
        Some(ReadinessIssue::InvalidPrompt)
    );
    let (mut settings, prompt) = settings_with_prompt(None);
    settings.post_process_provider_id = "gone".into();
    assert_eq!(
        missing(&settings, &prompt),
        Some(ReadinessIssue::UnknownProvider)
    );
}
#[test]
fn apple_intelligence_follows_the_probe() {
    let (mut settings, prompt) = settings_with_prompt(None);
    settings.post_process_providers.push(PostProcessProvider {
        id: APPLE_INTELLIGENCE_PROVIDER_ID.into(),
        label: "Apple Intelligence".into(),
        base_url: "apple-intelligence://local".into(),
        allow_base_url_edit: false,
        models_endpoint: None,
        supports_structured_output: true,
    });
    settings.post_process_provider_id = APPLE_INTELLIGENCE_PROVIDER_ID.into();
    settings.post_process_models.insert(
        APPLE_INTELLIGENCE_PROVIDER_ID.into(),
        "Apple Intelligence".into(),
    );
    assert!(readiness(&settings, &prompt, &Apple(true)).ready);
    let unavailable = readiness(&settings, &prompt, &Apple(false));
    assert_eq!(
        unavailable.missing,
        Some(ReadinessIssue::ProviderUnavailable)
    );
}
#[test]
fn no_readiness_or_debug_output_contains_the_key() {
    let (mut settings, prompt) = settings_with_prompt(None);
    settings
        .post_process_models
        .insert("openai".into(), "gpt".into());
    settings
        .post_process_api_keys
        .insert("openai".into(), SECRET.into());
    let choice = resolve(&settings, &prompt);
    assert!(!format!("{choice:?}").contains(SECRET));
    let ready = readiness(&settings, &prompt, &Apple(false));
    let json = serde_json::to_string(&ready).unwrap_or_default();
    assert!(!json.contains(SECRET) && json.contains("\"ready\":true"));
}
