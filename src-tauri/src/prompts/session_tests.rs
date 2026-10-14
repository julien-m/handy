//! Session rules of the dictation prompt (plan `prompt_sessions`): trigger, AI switch, choice on
//! the indicator, stop against choice, captured snapshot against edits and deletions, stale
//! sessions, and the orchestrator's rule that a prompt's shortcuts never change during a
//! dictation (decision 2026-10-14: deleting a prompt that has shortcuts goes through 004's
//! shortcut change, whose lease refuses it during a session).
use super::*;
use crate::prompts::library;
use crate::settings::{get_default_settings, LLMPrompt};
use crate::shortcut::prompt_targets::{empty_binding, store_binding};

/// AI on, default "Minimal correction", library: Minimal correction, Notes, Shorten.
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
    settings
}
fn prompt_id(view: &DictationPromptView) -> Option<&str> {
    view.prompt_id.as_deref()
}
fn frozen_id(sessions: &PromptSessions, id: u64) -> Option<String> {
    sessions.freeze(id).flatten().map(|prompt| prompt.id)
}

#[test]
fn the_trigger_decides_the_starting_prompt() {
    assert_eq!(Trigger::of_binding("transcribe"), Trigger::Plain);
    assert_eq!(
        Trigger::of_binding("transcribe_with_post_process"),
        Trigger::PostProcess
    );
    assert_eq!(
        Trigger::of_binding("prompt:notes"),
        Trigger::Prompt("notes".into())
    );
    let sessions = PromptSessions::default();
    assert_eq!(
        prompt_id(&sessions.start(&settings(), &Trigger::Plain)),
        None
    );
    assert_eq!(
        prompt_id(&sessions.start(&settings(), &Trigger::PostProcess)),
        Some("minimal")
    );
    let shortcut = sessions.start(&settings(), &Trigger::Prompt("notes".into()));
    assert_eq!(
        (prompt_id(&shortcut), shortcut.prompt_name.as_deref()),
        (Some("notes"), Some("NOTES"))
    );
}
#[test]
fn with_ai_off_no_trigger_processes_and_the_menu_is_hidden() {
    let mut off = settings();
    off.post_process_enabled = false;
    let sessions = PromptSessions::default();
    let view = sessions.start(&off, &Trigger::Prompt("notes".into()));
    assert_eq!((prompt_id(&view), view.selectable), (None, false));
    let refused = sessions.select(view.session_id, Some("shorten"), &off);
    assert_eq!(refused, Err(PromptChoiceRefusal::ProcessingOff));
}
#[test]
fn an_indicator_choice_applies_to_this_dictation_only() {
    let settings = settings();
    let sessions = PromptSessions::default();
    let view = sessions.start(&settings, &Trigger::PostProcess);
    let chosen = sessions.select(view.session_id, Some("shorten"), &settings);
    assert_eq!(chosen.map(|v| v.prompt_id), Ok(Some("shorten".into())));
    assert_eq!(
        frozen_id(&sessions, view.session_id).as_deref(),
        Some("shorten")
    );
    assert_eq!(
        settings.post_process_selected_prompt_id.as_deref(),
        Some("minimal")
    );
    let next = sessions.start(&settings, &Trigger::PostProcess);
    assert_eq!(
        prompt_id(&next),
        Some("minimal"),
        "the next dictation starts from the default"
    );
}
#[test]
fn a_plain_dictation_can_pick_a_prompt_and_none_skips_processing() {
    let settings = settings();
    let sessions = PromptSessions::default();
    let plain = sessions.start(&settings, &Trigger::Plain);
    assert!(sessions
        .select(plain.session_id, Some("notes"), &settings)
        .is_ok());
    assert_eq!(
        frozen_id(&sessions, plain.session_id).as_deref(),
        Some("notes")
    );
    let post = sessions.start(&settings, &Trigger::PostProcess);
    assert!(sessions.select(post.session_id, None, &settings).is_ok());
    let frozen = sessions.freeze(post.session_id);
    assert!(
        matches!(frozen, Some(None)),
        "None was chosen: no processing"
    );
}
#[test]
fn a_choice_after_the_stop_is_refused_and_the_frozen_prompt_stays() {
    let settings = settings();
    let sessions = PromptSessions::default();
    let view = sessions.start(&settings, &Trigger::Prompt("notes".into()));
    assert_eq!(
        frozen_id(&sessions, view.session_id).as_deref(),
        Some("notes")
    );
    let late = sessions.select(view.session_id, Some("shorten"), &settings);
    assert_eq!(late, Err(PromptChoiceRefusal::SessionEnded));
    assert_eq!(sessions.current().map(|v| v.selectable), Some(false));
}
#[test]
fn editing_or_deleting_a_captured_prompt_never_changes_it() -> Result<(), String> {
    let mut settings = settings();
    let sessions = PromptSessions::default();
    let view = sessions.start(&settings, &Trigger::Prompt("notes".into()));
    library::update(
        &mut settings,
        "notes",
        Some("Renamed"),
        Some("New: ${output}"),
    )
    .map_err(|e| format!("{e:?}"))?;
    library::remove(&mut settings, "notes").map_err(|e| format!("{e:?}"))?;
    let frozen = sessions
        .freeze(view.session_id)
        .flatten()
        .ok_or("no frozen prompt")?;
    assert_eq!(
        (frozen.name.as_str(), frozen.text.as_str()),
        ("NOTES", "notes: ${output}")
    );
    Ok(())
}
#[test]
fn a_stale_session_cannot_change_or_end_the_current_one() {
    let settings = settings();
    let sessions = PromptSessions::default();
    let old = sessions.start(&settings, &Trigger::Plain);
    let current = sessions.start(&settings, &Trigger::PostProcess);
    assert_eq!(
        sessions.select(old.session_id, Some("notes"), &settings),
        Err(PromptChoiceRefusal::SessionEnded)
    );
    sessions.end(old.session_id);
    assert_eq!(
        sessions.current().map(|v| v.session_id),
        Some(current.session_id)
    );
    sessions.cancel();
    assert_eq!(sessions.current(), None);
}
#[test]
fn missing_or_invalid_prompts_cannot_be_chosen_and_a_dangling_default_is_a_failure() {
    let mut settings = settings();
    settings
        .post_process_prompts
        .push(LLMPrompt::new("bad".into(), " ".into(), "x".into()));
    let sessions = PromptSessions::default();
    let view = sessions.start(&settings, &Trigger::Plain);
    assert_eq!(
        sessions.select(view.session_id, Some("gone"), &settings),
        Err(PromptChoiceRefusal::PromptMissing)
    );
    assert_eq!(
        sessions.select(view.session_id, Some("bad"), &settings),
        Err(PromptChoiceRefusal::PromptNotUsable)
    );
    settings.post_process_selected_prompt_id = Some("gone".into());
    let dangling = sessions.start(&settings, &Trigger::PostProcess);
    let frozen = sessions
        .freeze(dangling.session_id)
        .flatten()
        .map(|p| p.choice.err());
    assert_eq!(frozen, Some(Some(ReadinessIssue::PromptMissing)));
}
#[test]
fn only_deleting_a_prompt_with_shortcuts_changes_shortcuts_and_so_waits_for_the_dictation(
) -> Result<(), String> {
    use crate::shortcut::bindings_change::changed_actions;
    let mut before = settings();
    let mut binding = empty_binding("notes");
    binding.current_binding = "f13".into();
    store_binding(&mut before, binding);
    let mut with_shortcut = before.clone();
    library::remove(&mut with_shortcut, "notes").map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        changed_actions(&before, &with_shortcut).len(),
        1,
        "takes the shortcut lease"
    );
    let mut without = before.clone();
    library::remove(&mut without, "shorten").map_err(|e| format!("{e:?}"))?;
    assert!(
        changed_actions(&before, &without).is_empty(),
        "no shortcut change, no lease"
    );
    Ok(())
}
