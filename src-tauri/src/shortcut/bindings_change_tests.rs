//! Preparation and compensation of a shortcut list change against a fake backend with one
//! slot per parsed combination.
use super::*;
use crate::shortcut::fake_backend::FakeBackend;

type TestResult = Result<(), String>;

/// Default settings on the Tauri backend: Transcribe on `alt+space`, post-processing disabled.
fn tauri_settings() -> AppSettings {
    let mut settings = settings::get_default_settings();
    settings.keyboard_implementation = KeyboardImplementation::Tauri;
    settings.post_process_enabled = false;
    if let Some(transcribe) = settings.bindings.get_mut("transcribe") {
        transcribe.current_binding = "alt+space".into();
        transcribe.additional_bindings.clear();
    }
    settings
}

/// `settings` with the shortcuts of `action_id` replaced, first one first.
fn with_list(
    settings: &AppSettings,
    action_id: &str,
    shortcuts: &[&str],
) -> Result<AppSettings, String> {
    let mut changed = settings.clone();
    let binding = changed
        .bindings
        .get_mut(action_id)
        .ok_or(format!("no binding {action_id}"))?;
    let (first, rest) = shortcuts.split_first().ok_or("no shortcut")?;
    binding.current_binding = first.to_string();
    binding.additional_bindings = rest.iter().map(|s| s.to_string()).collect();
    Ok(changed)
}

fn registered(backend: &mut FakeBackend, action_id: &str, shortcut: &str) -> TestResult {
    backend.register(&ShortcutRegistration::new(action_id, shortcut))
}

fn apply_delta_everywhere(_: &str) -> Preparation {
    Preparation::ApplyDelta
}

#[test]
fn only_actions_whose_list_changed_are_prepared() -> TestResult {
    let before = tauri_settings();
    let after = with_list(&before, "transcribe", &["alt+space", "f13"])?;
    let changes = changed_actions(&before, &after);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].after.shortcuts(), vec!["alt+space", "f13"]);
    assert!(changed_actions(&before, &before).is_empty());
    Ok(())
}

#[test]
fn an_added_shortcut_is_registered_before_the_commit() -> TestResult {
    let before = tauri_settings();
    let after = with_list(&before, "transcribe", &["alt+space", "f13"])?;
    let mut backend = FakeBackend::default();
    registered(&mut backend, "transcribe", "alt+space")?;
    let changes = changed_actions(&before, &after);
    prepare_changes(&mut backend, &before, &changes, &apply_delta_everywhere).map_err(|e| e.1)?;
    let added = ShortcutRegistration::new("transcribe", "f13");
    assert_eq!(backend.holder("f13"), Some(&added));
    Ok(())
}

#[test]
fn a_refused_commit_restores_the_previous_registrations() -> TestResult {
    let before = tauri_settings();
    let after = with_list(&before, "transcribe", &["f13"])?;
    let mut backend = FakeBackend::default();
    registered(&mut backend, "transcribe", "alt+space")?;
    let changes = changed_actions(&before, &after);
    prepare_changes(&mut backend, &before, &changes, &apply_delta_everywhere).map_err(|e| e.1)?;
    assert!(backend.holder("alt+space").is_none());
    compensate_changes(&mut backend, &before, &changes, &apply_delta_everywhere);
    let original = ShortcutRegistration::new("transcribe", "alt+space");
    assert_eq!(backend.holder("alt+space"), Some(&original));
    assert!(backend.holder("f13").is_none());
    Ok(())
}

#[test]
fn an_os_refusal_undoes_the_actions_prepared_before_it() -> TestResult {
    let mut before = tauri_settings();
    before.post_process_enabled = true;
    let before = with_list(&before, "transcribe_with_post_process", &["f20"])?;
    let partial = with_list(&before, "transcribe", &["alt+space", "f13"])?;
    let after = with_list(&partial, "transcribe_with_post_process", &["f20", "f14"])?;
    let mut backend = FakeBackend {
        refused: vec!["f14".into()],
        ..FakeBackend::default()
    };
    registered(&mut backend, "transcribe", "alt+space")?;
    registered(&mut backend, "transcribe_with_post_process", "f20")?;
    let changes = changed_actions(&before, &after);
    let refused = prepare_changes(&mut backend, &before, &changes, &apply_delta_everywhere);
    let refusal = (
        "transcribe_with_post_process".to_string(),
        "refused by the OS".to_string(),
    );
    assert_eq!(refused, Err(refusal));
    assert!(backend.holder("f13").is_none());
    assert!(backend.holder("f14").is_none());
    assert!(backend.holder("alt+space").is_some() && backend.holder("f20").is_some());
    Ok(())
}

#[test]
fn a_probed_action_keeps_nothing_registered_and_needs_no_undo() -> TestResult {
    let before = tauri_settings();
    let after = with_list(&before, "transcribe_with_post_process", &["f15"])?;
    let mut backend = FakeBackend::default();
    let changes = changed_actions(&before, &after);
    let probe = |_: &str| Preparation::Probe;
    prepare_changes(&mut backend, &before, &changes, &probe).map_err(|e| e.1)?;
    assert_eq!(backend.probed.len(), 1);
    assert!(backend.slots.is_empty());
    compensate_changes(&mut backend, &before, &changes, &probe);
    assert!(backend.slots.is_empty());
    Ok(())
}

#[test]
fn a_probe_refusal_is_returned_without_registering() -> TestResult {
    let before = tauri_settings();
    let after = with_list(&before, "transcribe_with_post_process", &["f15"])?;
    let mut backend = FakeBackend {
        refused: vec!["f15".into()],
        ..FakeBackend::default()
    };
    let changes = changed_actions(&before, &after);
    let refused = prepare_changes(&mut backend, &before, &changes, &|_: &str| {
        Preparation::Probe
    });
    let refusal = (
        "transcribe_with_post_process".to_string(),
        "refused by the OS".to_string(),
    );
    assert_eq!(refused, Err(refusal));
    assert!(backend.slots.is_empty());
    Ok(())
}
