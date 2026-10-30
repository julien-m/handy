//! Prompt shortcuts (005 AC-003, AC-006) through 004's shared machinery and a fake backend with
//! one slot per parsed combination: 0–2 shortcuts per prompt, the shared conflict check, the
//! post-processing switch registering and releasing them, and a prompt deletion releasing its
//! shortcuts before the commit and registering them again on refusal.
use super::*;
use crate::prompts::library;
use crate::settings::{get_default_settings, KeyboardImplementation, LLMPrompt};
use crate::shortcut::action_shortcuts::{ShortcutEdit, ShortcutEditError};
use crate::shortcut::bindings_change::{changed_actions, compensate_changes, prepare_changes};
use crate::shortcut::edits::{plan_shortcut_edit, toggle_post_process, Preparation};
use crate::shortcut::fake_backend::FakeBackend;
use crate::shortcut::identity::ShortcutRegistration;
use crate::shortcut::registration_delta::RegistrationPort;

type TestResult = Result<(), String>;

/// Tauri backend, "transcribe" on alt+space, one prompt "notes" with `shortcuts`.
fn settings_with(shortcuts: &[&str], enabled: bool) -> AppSettings {
    let mut settings = get_default_settings();
    settings.keyboard_implementation = KeyboardImplementation::Tauri;
    settings.post_process_enabled = enabled;
    if let Some(transcribe) = settings.bindings.get_mut("transcribe") {
        transcribe.current_binding = "alt+space".into();
    }
    settings.post_process_prompts =
        vec![LLMPrompt::new("notes".into(), "Notes".into(), "x".into())];
    let mut binding = empty_binding("notes");
    let mut list = shortcuts.iter().map(|s| (*s).to_owned());
    binding.current_binding = list.next().unwrap_or_default();
    binding.additional_bindings = list.collect();
    store_binding(&mut settings, binding);
    settings
}
fn add(settings: &AppSettings, shortcut: &str) -> Result<ShortcutBinding, ShortcutEditError> {
    let edit = ShortcutEdit::Add {
        shortcut: shortcut.into(),
    };
    plan_shortcut_edit(settings, "prompt:notes", &edit).map(|(_, planned)| planned)
}

// @spec AC-006: 0-2 shortcuts, shared conflict check — .specs/features/005-custom-ai-prompts/spec.md#ac-006
#[test]
fn a_prompt_takes_up_to_two_shortcuts_checked_against_every_other_shortcut() {
    let empty = settings_with(&[], true);
    let first = add(&empty, "f13").map(|binding| binding.shortcuts());
    assert_eq!(first, Ok(vec!["f13".to_string()]));
    let two = settings_with(&["f13", "f14"], true);
    assert_eq!(
        add(&two, "f15").err(),
        Some(ShortcutEditError::LimitExceeded { limit: 2 })
    );
    let conflict = add(&empty, "option+space");
    assert!(matches!(
        conflict,
        Err(ShortcutEditError::ShortcutConflict { .. })
    ));
}
#[test]
fn a_prompt_may_lose_its_last_shortcut_and_reset_removes_them_all() {
    let one = settings_with(&["f13"], true);
    let edit = ShortcutEdit::Remove {
        shortcut: "f13".into(),
    };
    let removed = plan_shortcut_edit(&one, "prompt:notes", &edit).map(|(_, p)| p.shortcuts());
    assert_eq!(removed, Ok(Vec::<String>::new()));
    let two = settings_with(&["f13", "f14"], true);
    let reset = plan_shortcut_edit(&two, "prompt:notes", &ShortcutEdit::Reset);
    assert_eq!(reset.map(|(_, p)| p.shortcuts()), Ok(Vec::<String>::new()));
    let gone = plan_shortcut_edit(&one, "prompt:other", &ShortcutEdit::Reset);
    assert!(matches!(
        gone,
        Err(ShortcutEditError::ActionNotFound { .. })
    ));
}
#[test]
fn prompt_shortcuts_follow_the_post_processing_switch() -> TestResult {
    let off = settings_with(&["f13"], false);
    assert!(!crate::shortcut::is_statically_registered(
        "prompt:notes",
        &off
    ));
    let mut backend = FakeBackend::default();
    let (on, refused) = toggle_post_process(&mut backend, &off, true);
    assert!(refused.is_empty());
    let registration = ShortcutRegistration::new("prompt:notes", "f13");
    assert_eq!(backend.holder("f13"), Some(&registration));
    assert!(crate::shortcut::is_statically_registered(
        "prompt:notes",
        &on
    ));
    toggle_post_process(&mut backend, &on, false);
    assert!(backend.holder("f13").is_none());
    Ok(())
}
// @spec AC-006: Deleting a prompt removes its shortcuts — .specs/features/005-custom-ai-prompts/spec.md#ac-006
#[test]
fn deleting_a_prompt_releases_its_shortcuts_and_a_refused_commit_restores_them() -> TestResult {
    let before = settings_with(&["f13", "f14"], true);
    let mut backend = FakeBackend::default();
    for shortcut in ["f13", "f14"] {
        backend.register(&ShortcutRegistration::new("prompt:notes", shortcut))?;
    }
    let mut after = before.clone();
    library::remove(&mut after, "notes").map_err(|e| format!("{e:?}"))?;
    let changes = changed_actions(&before, &after);
    assert_eq!(changes.len(), 1);
    let apply = |_: &str| Preparation::ApplyDelta;
    prepare_changes(&mut backend, &before, &changes, &apply).map_err(|e| e.1)?;
    assert!(backend.slots.is_empty());
    compensate_changes(&mut backend, &before, &changes, &apply);
    assert_eq!(
        backend.holder("f14"),
        Some(&ShortcutRegistration::new("prompt:notes", "f14"))
    );
    Ok(())
}
#[test]
fn prompt_shortcuts_drive_a_post_processing_dictation() {
    assert!(crate::transcription_coordinator::is_transcribe_binding(
        "prompt:notes"
    ));
    assert!(crate::actions::action_for("prompt:notes").is_some());
    assert!(crate::actions::action_for("prompt:").is_some());
    assert!(crate::actions::action_for("unknown").is_none());
}
