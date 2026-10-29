//! Remembered plain-dictation prompt (005 FR-007, AC-009): the prompt chosen on the indicator
//! during a plain dictation starts the next plain dictations, until None is chosen; the
//! post-processing and prompt shortcuts keep their own prompt; AI off, a deleted or an unusable
//! remembered prompt starts on None.
use super::*;
use crate::prompts::library;
use crate::settings::{get_default_settings, LLMPrompt};

/// AI on, default "minimal", library: minimal, notes, shorten; "notes" remembered for plain.
fn settings() -> AppSettings {
    let mut settings = get_default_settings();
    settings.post_process_enabled = true;
    settings.post_process_prompts = ["minimal", "notes", "shorten"]
        .iter()
        .map(|id| {
            LLMPrompt::new(
                (*id).into(),
                id.to_uppercase(),
                format!("{id}: ${{output}}"),
            )
        })
        .collect();
    settings.post_process_selected_prompt_id = Some("minimal".into());
    settings.plain_dictation_prompt_id = Some("notes".into());
    settings
}
fn start_id(settings: &AppSettings, trigger: &Trigger) -> Option<String> {
    PromptSessions::default().start(settings, trigger).prompt_id
}

// @spec AC-009: Plain dictation starts with the remembered prompt — .specs/features/005-custom-ai-prompts/spec.md#ac-009
#[test]
fn a_plain_dictation_starts_with_the_remembered_prompt_and_shortcuts_keep_theirs() {
    let settings = settings();
    let sessions = PromptSessions::default();
    assert!(sessions.start(&settings, &Trigger::Plain).remembered);
    assert!(!sessions.start(&settings, &Trigger::PostProcess).remembered);
    let mut off = settings.clone();
    off.post_process_enabled = false;
    assert!(!sessions.start(&off, &Trigger::Plain).remembered);
    assert_eq!(
        start_id(&settings, &Trigger::Plain).as_deref(),
        Some("notes")
    );
    assert_eq!(
        start_id(&settings, &Trigger::PostProcess).as_deref(),
        Some("minimal")
    );
    assert_eq!(
        start_id(&settings, &Trigger::Prompt("shorten".into())).as_deref(),
        Some("shorten")
    );
    let mut nothing = settings.clone();
    nothing.plain_dictation_prompt_id = None;
    assert_eq!(start_id(&nothing, &Trigger::Plain), None);
}

// @spec AC-009: AI off, deleted or unusable remembered prompt starts on None — .specs/features/005-custom-ai-prompts/spec.md#ac-009
#[test]
fn ai_off_or_a_missing_or_unusable_remembered_prompt_starts_on_none() {
    let mut off = settings();
    off.post_process_enabled = false;
    assert_eq!(start_id(&off, &Trigger::Plain), None);
    let mut gone = settings();
    gone.plain_dictation_prompt_id = Some("gone".into());
    assert_eq!(start_id(&gone, &Trigger::Plain), None);
    let mut bad = settings();
    bad.post_process_prompts
        .push(LLMPrompt::new("bad".into(), " ".into(), "x".into()));
    bad.plain_dictation_prompt_id = Some("bad".into());
    assert_eq!(start_id(&bad, &Trigger::Plain), None);
}

// @spec AC-009: Deleting the remembered prompt forgets it — .specs/features/005-custom-ai-prompts/spec.md#ac-009
#[test]
fn deleting_the_remembered_prompt_forgets_it_and_keeps_the_default() -> Result<(), String> {
    let mut settings = settings();
    library::remove(&mut settings, "notes").map_err(|e| format!("{e:?}"))?;
    assert_eq!(settings.plain_dictation_prompt_id, None);
    assert_eq!(
        settings.post_process_selected_prompt_id.as_deref(),
        Some("minimal")
    );
    Ok(())
}

// @spec AC-009: Only an indicator choice on a plain dictation is remembered — .specs/features/005-custom-ai-prompts/spec.md#ac-009
#[test]
fn only_an_indicator_choice_during_a_plain_dictation_is_a_candidate_to_remember() {
    let settings = settings();
    let sessions = PromptSessions::default();
    let chosen_on_plain = |trigger: &Trigger, choice: Option<Option<&str>>, s: &AppSettings| {
        let view = sessions.start(s, trigger);
        if let Some(choice) = choice {
            assert!(sessions.select(view.session_id, choice, s).is_ok());
        }
        sessions
            .freeze_current()
            .map(|frozen| frozen.chosen_on_plain)
    };
    assert_eq!(
        chosen_on_plain(&Trigger::Plain, Some(Some("shorten")), &settings),
        Some(true)
    );
    assert_eq!(
        chosen_on_plain(&Trigger::Plain, Some(None), &settings),
        Some(true)
    );
    assert_eq!(
        chosen_on_plain(&Trigger::Plain, None, &settings),
        Some(false)
    );
    let post = Trigger::PostProcess;
    assert_eq!(
        chosen_on_plain(&post, Some(Some("shorten")), &settings),
        Some(false)
    );
    let shortcut = Trigger::Prompt("notes".into());
    assert_eq!(
        chosen_on_plain(&shortcut, Some(None), &settings),
        Some(false)
    );
    let mut off = settings.clone();
    off.post_process_enabled = false;
    assert_eq!(chosen_on_plain(&Trigger::Plain, None, &off), Some(false));
}

// @spec FR-007: Remembered prompt update rule — .specs/features/005-custom-ai-prompts/spec.md#fr-007
#[test]
fn the_remembered_prompt_changes_only_when_a_plain_choice_differs_from_it() {
    assert_eq!(
        plain_memory_update(true, Some("notes"), None),
        Some(Some("notes".to_owned()))
    );
    assert_eq!(
        plain_memory_update(true, Some("shorten"), Some("notes")),
        Some(Some("shorten".to_owned()))
    );
    assert_eq!(plain_memory_update(true, None, Some("notes")), Some(None));
    assert_eq!(
        plain_memory_update(true, Some("notes"), Some("notes")),
        None
    );
    assert_eq!(plain_memory_update(true, None, None), None);
    assert_eq!(
        plain_memory_update(false, Some("shorten"), Some("notes")),
        None
    );
}
