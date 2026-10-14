//! Tests of how shortcut edits and the post-processing toggle prepare the
//! keyboard backend, against a fake backend with one slot per parsed
//! combination.

use super::*;
use crate::shortcut::fake_backend::FakeBackend;
use crate::shortcut::identity::ShortcutRegistration;
use crate::shortcut::prompt_targets::POST_PROCESS_ACTION;

type TestResult = Result<(), String>;

const TAURI: KeyboardImplementation = KeyboardImplementation::Tauri;

fn state(is_static: bool, is_cancel_registered: bool, has_dynamic: bool) -> ActionState {
    ActionState {
        is_statically_registered: is_static,
        is_cancel_registered,
        has_dynamic_cancel: has_dynamic,
    }
}

#[test]
fn registered_actions_apply_the_delta() {
    let prepared = preparation_for("transcribe", state(true, false, true), TAURI);
    assert_eq!(prepared, Preparation::ApplyDelta);
}

#[test]
fn cancel_outside_recording_is_probed_with_tauri() {
    let prepared = preparation_for("cancel", state(false, false, true), TAURI);
    assert_eq!(prepared, Preparation::Probe);
}

#[test]
fn cancel_is_not_probed_while_recording_or_on_linux() {
    let recording = preparation_for("cancel", state(false, true, true), TAURI);
    assert_eq!(recording, Preparation::Nothing);
    let linux = preparation_for("cancel", state(false, false, false), TAURI);
    assert_eq!(linux, Preparation::Nothing);
}

#[test]
fn disabled_post_processing_is_probed_with_tauri_only() {
    let action = POST_PROCESS_ACTION;
    let tauri = preparation_for(action, state(false, false, true), TAURI);
    assert_eq!(tauri, Preparation::Probe);
    let handy_keys = KeyboardImplementation::HandyKeys;
    let cancel = preparation_for("cancel", state(false, false, true), handy_keys);
    assert_eq!(cancel, Preparation::Nothing);
    let post = preparation_for(action, state(false, false, true), handy_keys);
    assert_eq!(post, Preparation::Nothing);
}

/// Sets the shortcuts of `action_id` in `settings`, first one first.
fn set_shortcuts(settings: &mut AppSettings, action_id: &str, shortcuts: &[&str]) -> TestResult {
    let binding = settings
        .bindings
        .get_mut(action_id)
        .ok_or(format!("no default binding for {action_id}"))?;
    let (first, rest) = shortcuts.split_first().ok_or("no shortcut")?;
    binding.current_binding = first.to_string();
    binding.additional_bindings = rest.iter().map(|s| s.to_string()).collect();
    Ok(())
}

/// Default settings on the Tauri backend, on every platform: Transcribe on
/// `alt+space`, post-processing `enabled` with `shortcuts`.
fn settings_with_post_process(enabled: bool, shortcuts: &[&str]) -> Result<AppSettings, String> {
    let mut settings = settings::get_default_settings();
    settings.keyboard_implementation = TAURI;
    settings.post_process_enabled = enabled;
    set_shortcuts(&mut settings, "transcribe", &["alt+space"])?;
    set_shortcuts(&mut settings, POST_PROCESS_ACTION, shortcuts)?;
    Ok(settings)
}

#[test]
fn enabling_post_processing_succeeds_although_the_os_refuses_a_shortcut() -> TestResult {
    let settings = settings_with_post_process(false, &["f13", "f14"])?;
    let mut backend = FakeBackend {
        refused: vec!["f14".into()],
        ..FakeBackend::default()
    };
    let (updated, refused) = toggle_post_process(&mut backend, &settings, true);
    assert!(updated.post_process_enabled);
    let expected = RefusedShortcut {
        action_id: POST_PROCESS_ACTION.into(),
        shortcut: "f14".into(),
        reason: "refused by the OS".into(),
    };
    assert_eq!(refused, vec![expected]);
    let accepted = ShortcutRegistration::new(POST_PROCESS_ACTION, "f13");
    assert_eq!(backend.holder("f13"), Some(&accepted));
    assert!(backend.holder("f14").is_none());
    Ok(())
}

#[test]
fn enabling_post_processing_leaves_a_shared_slot_to_its_holder() -> TestResult {
    let settings = settings_with_post_process(false, &["option+space", "f13"])?;
    let transcribe = ShortcutRegistration::new("transcribe", "alt+space");
    let mut backend = FakeBackend::default();
    backend.register(&transcribe)?;
    let (updated, refused) = toggle_post_process(&mut backend, &settings, true);
    assert!(updated.post_process_enabled);
    assert!(refused.is_empty());
    assert_eq!(backend.holder("option+space"), Some(&transcribe));
    Ok(())
}

#[test]
fn disabling_post_processing_hands_a_shared_slot_back() -> TestResult {
    let settings = settings_with_post_process(true, &["option+space", "f13"])?;
    let post = ShortcutRegistration::new(POST_PROCESS_ACTION, "option+space");
    let mut backend = FakeBackend::default();
    backend.register(&post)?;
    backend.register(&ShortcutRegistration::new(POST_PROCESS_ACTION, "f13"))?;
    let (updated, refused) = toggle_post_process(&mut backend, &settings, false);
    assert!(!updated.post_process_enabled);
    assert!(refused.is_empty());
    let survivor = ShortcutRegistration::new("transcribe", "alt+space");
    assert_eq!(backend.holder("alt+space"), Some(&survivor));
    assert!(backend.holder("f13").is_none());
    Ok(())
}

#[test]
fn an_unchanged_toggle_touches_no_registration() -> TestResult {
    let settings = settings_with_post_process(true, &["f13"])?;
    let mut backend = FakeBackend::default();
    let (updated, refused) = toggle_post_process(&mut backend, &settings, true);
    assert!(updated.post_process_enabled);
    assert!(refused.is_empty());
    assert!(backend.slots.is_empty());
    Ok(())
}

#[test]
fn a_check_reports_a_conflict_and_saves_nothing() -> TestResult {
    let settings = settings_with_post_process(true, &["f13"])?;
    let edit = ShortcutEdit::Add {
        shortcut: "f13".into(),
    };
    let refusal = check_shortcut_edit(&settings, "transcribe", &edit);
    let Some(ShortcutEditError::ShortcutConflict { conflict }) = refusal else {
        return Err(format!("expected a conflict, got {refusal:?}"));
    };
    assert_eq!(conflict.owning_action, POST_PROCESS_ACTION);
    let transcribe = settings.bindings.get("transcribe").ok_or("no transcribe")?;
    assert_eq!(transcribe.shortcuts(), vec!["alt+space".to_string()]);
    Ok(())
}

#[test]
fn a_check_accepts_a_free_shortcut_and_enforces_the_limit() -> TestResult {
    let mut settings = settings_with_post_process(true, &["f13"])?;
    let add = |shortcut: &str| ShortcutEdit::Add {
        shortcut: shortcut.into(),
    };
    assert_eq!(
        check_shortcut_edit(&settings, "transcribe", &add("f14")),
        None
    );
    set_shortcuts(
        &mut settings,
        "transcribe",
        &["alt+space", "f15", "f16", "f17"],
    )?;
    let refusal = check_shortcut_edit(&settings, "transcribe", &add("f14"));
    assert_eq!(refusal, Some(ShortcutEditError::LimitExceeded { limit: 4 }));
    Ok(())
}
