//! Library rules on in-memory settings: Unicode bounds, case-insensitive uniqueness, legacy
//! issues, delete-to-None, default and override checks.
use super::*;
use crate::settings::get_default_settings;

fn settings_with(prompts: &[(&str, &str, &str)]) -> AppSettings {
    let mut settings = get_default_settings();
    settings.post_process_prompts = prompts
        .iter()
        .map(|(id, name, text)| LLMPrompt::new((*id).into(), (*name).into(), (*text).into()))
        .collect();
    settings
}
fn invalid(issue: PromptIssue) -> Result<(), PromptError> {
    Err(PromptError::Invalid(issue))
}
fn override_of(provider_id: &str, model: &str) -> Option<PromptProviderOverride> {
    Some(PromptProviderOverride {
        provider_id: provider_id.into(),
        model: model.into(),
    })
}

// @spec AC-001: Name and text bounds — .specs/features/005-custom-ai-prompts/spec.md#ac-001
#[test]
fn names_count_unicode_characters_after_trimming() {
    let sixty = "é".repeat(60);
    assert_eq!(name_issue(&format!("  {sixty}  ")), None);
    assert_eq!(name_issue(&"é".repeat(61)), Some(PromptIssue::NameTooLong));
    assert_eq!(name_issue("   "), Some(PromptIssue::NameEmpty));
    assert_eq!(name_issue("a"), None);
}
#[test]
fn texts_count_characters_and_refuse_whitespace_only() {
    assert_eq!(text_issue(&"ü".repeat(8000)), None);
    assert_eq!(
        text_issue(&"ü".repeat(8001)),
        Some(PromptIssue::TextTooLong)
    );
    assert_eq!(text_issue(" \n\t "), Some(PromptIssue::TextEmpty));
    assert_eq!(text_issue("x"), None);
}
#[test]
fn create_trims_the_name_keeps_the_text_and_refuses_duplicates_ignoring_case() {
    let mut settings = settings_with(&[("a", "Écrire", "x")]);
    assert_eq!(
        create(&mut settings, "b", "  écrire ", "y"),
        invalid(PromptIssue::NameDuplicate)
    );
    assert_eq!(
        create(&mut settings, "b", "  Formal email ", " ${output} "),
        Ok(())
    );
    let created = find(&settings, "b").cloned();
    assert_eq!(
        created.as_ref().map(|p| p.name.as_str()),
        Some("Formal email")
    );
    assert_eq!(created.map(|p| p.prompt), Some(" ${output} ".to_string()));
}
// @spec AC-001: At most 50 prompts — .specs/features/005-custom-ai-prompts/spec.md#ac-001
#[test]
fn create_stops_at_fifty_prompts() {
    let ids: Vec<String> = (0..50).map(|index| format!("p{index}")).collect();
    let prompts: Vec<(&str, &str, &str)> = ids
        .iter()
        .map(|id| (id.as_str(), id.as_str(), "x"))
        .collect();
    let mut settings = settings_with(&prompts);
    assert_eq!(
        create(&mut settings, "new", "New", "x"),
        Err(PromptError::LimitExceeded)
    );
    assert_eq!(settings.post_process_prompts.len(), 50);
}
#[test]
fn renaming_a_prompt_to_its_own_name_in_another_case_is_allowed() {
    let mut settings = settings_with(&[("a", "Notes", "x"), ("b", "Other", "y")]);
    assert_eq!(update(&mut settings, "a", Some("NOTES"), None), Ok(()));
    assert_eq!(
        update(&mut settings, "b", Some("notes"), None),
        invalid(PromptIssue::NameDuplicate)
    );
    assert_eq!(
        update(&mut settings, "zz", Some("x"), None),
        Err(PromptError::NotFound)
    );
}
// @spec AC-002: Invalid legacy prompts kept and flagged — .specs/features/005-custom-ai-prompts/spec.md#ac-002
#[test]
fn legacy_issues_are_reported_and_kept_until_an_edit_repairs_them() {
    let long = "n".repeat(61);
    let mut settings = settings_with(&[("a", &long, "x"), ("b", "Dup", " "), ("c", "dup", "y")]);
    let issues = |settings: &AppSettings, id: &str| {
        find(settings, id)
            .map(|p| issues_of(settings, p))
            .unwrap_or_default()
    };
    assert_eq!(issues(&settings, "a"), [PromptIssue::NameTooLong]);
    assert_eq!(
        issues(&settings, "b"),
        [PromptIssue::NameDuplicate, PromptIssue::TextEmpty]
    );
    assert_eq!(issues(&settings, "c"), [PromptIssue::NameDuplicate]);
    // A text-only edit of an invalid legacy prompt must also repair its name.
    assert_eq!(
        update(&mut settings, "a", None, Some("z")),
        invalid(PromptIssue::NameTooLong)
    );
    assert_eq!(update(&mut settings, "c", Some("Unique"), None), Ok(()));
    assert_eq!(issues(&settings, "b"), [PromptIssue::TextEmpty]);
}
#[test]
fn deleting_the_default_prompt_resets_it_to_none_even_for_the_last_prompt() {
    let mut settings = settings_with(&[("a", "Only", "x")]);
    settings.post_process_selected_prompt_id = Some("a".into());
    assert_eq!(remove(&mut settings, "a"), Ok(()));
    assert!(settings.post_process_prompts.is_empty());
    assert_eq!(settings.post_process_selected_prompt_id, None);
    assert_eq!(remove(&mut settings, "a"), Err(PromptError::NotFound));
}
#[test]
fn deleting_another_prompt_keeps_the_default() {
    let mut settings = settings_with(&[("a", "A", "x"), ("b", "B", "y")]);
    settings.post_process_selected_prompt_id = Some("a".into());
    assert_eq!(remove(&mut settings, "b"), Ok(()));
    assert_eq!(
        settings.post_process_selected_prompt_id.as_deref(),
        Some("a")
    );
}
#[test]
fn only_a_usable_prompt_can_become_the_default() {
    let mut settings = settings_with(&[("a", "A", "x"), ("bad", "", "y")]);
    assert_eq!(
        set_default(&mut settings, Some("bad")),
        Err(PromptError::NotUsable)
    );
    assert_eq!(
        set_default(&mut settings, Some("zz")),
        Err(PromptError::NotFound)
    );
    assert_eq!(set_default(&mut settings, Some("a")), Ok(()));
    assert_eq!(set_default(&mut settings, None), Ok(()));
    assert_eq!(settings.post_process_selected_prompt_id, None);
}
#[test]
fn an_override_needs_a_known_provider_and_a_model_and_clears_atomically() {
    let mut settings = settings_with(&[("a", "A", "x")]);
    let all = PlatformCapabilities::only(&[PlatformCapability::AppleIntelligence]);
    assert_eq!(
        set_override(&mut settings, "a", override_of("custom", " "), all),
        invalid(PromptIssue::OverrideIncomplete)
    );
    assert_eq!(
        set_override(&mut settings, "a", override_of("nope", "m"), all),
        invalid(PromptIssue::UnknownProvider)
    );
    assert_eq!(
        set_override(
            &mut settings,
            "a",
            override_of(" custom ", " llama3.2 "),
            all
        ),
        Ok(())
    );
    let stored = find(&settings, "a").and_then(|p| p.provider_override.clone());
    assert_eq!(stored, override_of("custom", "llama3.2"));
    assert_eq!(set_override(&mut settings, "a", None, all), Ok(()));
    assert_eq!(set_override(&mut settings, "a", None, all), Ok(()));
    assert_eq!(
        find(&settings, "a").and_then(|p| p.provider_override.clone()),
        None
    );
}
#[test]
fn an_apple_intelligence_override_is_refused_without_the_capability() {
    let mut settings = settings_with(&[("a", "A", "x")]);
    let none = PlatformCapabilities::only(&[]);
    assert_eq!(
        set_override(
            &mut settings,
            "a",
            override_of(APPLE_INTELLIGENCE_PROVIDER_ID, "Apple Intelligence"),
            none
        ),
        Err(PromptError::UnsupportedPlatform)
    );
}
#[test]
fn generated_ids_are_unique_within_a_process() {
    let first = generate_id();
    let second = generate_id();
    assert_ne!(first, second);
    assert!(first.starts_with("prompt_"));
}
