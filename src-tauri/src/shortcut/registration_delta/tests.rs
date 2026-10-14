//! Tests of the registration delta, against a fake backend that keeps one
//! native slot per parsed combination, as the Tauri plugin does.

use super::*;
use crate::shortcut::tauri_impl::same_registration_slot;
use std::collections::HashMap;
use tauri_plugin_global_shortcut::Shortcut;

type TestResult = Result<(), String>;

#[derive(Default)]
struct FakeBackend {
    /// Holder of each native slot, keyed by the parsed combination id.
    slots: HashMap<u32, ShortcutRegistration>,
    /// Accelerators the OS refuses (owned by another application).
    refused: Vec<String>,
    probed: Vec<String>,
}

fn slot_of(registration: &ShortcutRegistration) -> Result<u32, String> {
    registration
        .accelerator
        .parse::<Shortcut>()
        .map(|shortcut| shortcut.id())
        .map_err(|e| e.to_string())
}

impl FakeBackend {
    fn holder(&self, accelerator: &str) -> Option<&ShortcutRegistration> {
        let slot = slot_of(&reg("any", accelerator)).ok()?;
        self.slots.get(&slot)
    }

    fn check_free(&self, registration: &ShortcutRegistration) -> Result<u32, String> {
        if self.refused.contains(&registration.accelerator) {
            return Err("refused by the OS".into());
        }
        let slot = slot_of(registration)?;
        match self.slots.contains_key(&slot) {
            true => Err("already in use".into()),
            false => Ok(slot),
        }
    }
}

impl RegistrationPort for FakeBackend {
    fn register(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        let slot = self.check_free(registration)?;
        self.slots.insert(slot, registration.clone());
        Ok(())
    }

    fn unregister(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        let slot = slot_of(registration)?;
        self.slots
            .remove(&slot)
            .map(|_| ())
            .ok_or("not registered".into())
    }

    fn probe(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        self.check_free(registration)?;
        self.probed.push(registration.accelerator.clone());
        Ok(())
    }
}

fn reg(action_id: &str, shortcut: &str) -> ShortcutRegistration {
    ShortcutRegistration::new(action_id, shortcut)
}

fn regs(action_id: &str, shortcuts: &[&str]) -> Vec<ShortcutRegistration> {
    shortcuts.iter().map(|s| reg(action_id, s)).collect()
}

fn tauri_slot(a: &ShortcutRegistration, b: &ShortcutRegistration) -> bool {
    same_registration_slot(&a.accelerator, &b.accelerator)
}

fn backend_with(registered: &[ShortcutRegistration]) -> Result<FakeBackend, String> {
    let mut backend = FakeBackend::default();
    registered.iter().try_for_each(|r| backend.register(r))?;
    Ok(backend)
}

#[test]
fn replace_releases_the_old_shortcut_and_registers_the_new() -> TestResult {
    let before = regs("transcribe", &["alt+space", "f13"]);
    let after = regs("transcribe", &["f5", "f13"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    assert_eq!(delta.released, regs("transcribe", &["alt+space"]));
    assert_eq!(delta.added, regs("transcribe", &["f5"]));
    let mut backend = backend_with(&before)?;
    apply_registration_delta(&mut backend, &delta)?;
    assert!(backend.holder("alt+space").is_none());
    assert_eq!(backend.holder("f5"), Some(&reg("transcribe", "f5")));
    assert_eq!(backend.holder("f13"), Some(&reg("transcribe", "f13")));
    Ok(())
}

#[test]
fn an_equivalent_replacement_reuses_the_released_slot() -> TestResult {
    let before = regs("transcribe", &["alt+space"]);
    let after = regs("transcribe", &["option+space"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    let mut backend = backend_with(&before)?;
    apply_registration_delta(&mut backend, &delta)?;
    assert_eq!(
        backend.holder("alt+space"),
        Some(&reg("transcribe", "option+space"))
    );
    Ok(())
}

#[test]
fn removing_the_first_shortcut_keeps_the_promoted_registration() {
    let before = regs("transcribe", &["f13", "f14"]);
    let after = regs("transcribe", &["f14"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    assert_eq!(delta.released, regs("transcribe", &["f13"]));
    assert!(delta.added.is_empty() && delta.handovers.is_empty());
}

#[test]
fn reset_to_a_default_already_listed_only_releases_the_others() {
    let before = regs("transcribe", &["f13", "option+space"]);
    let after = regs("transcribe", &["option+space"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    assert_eq!(delta.released, regs("transcribe", &["f13"]));
    assert!(delta.added.is_empty());
}

#[test]
fn refused_addition_restores_the_previous_registrations() -> TestResult {
    let before = regs("transcribe", &["f13", "f14"]);
    let after = regs("transcribe", &["f5", "f14"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    let mut backend = backend_with(&before)?;
    backend.refused.push("f5".into());
    let result = apply_registration_delta(&mut backend, &delta);
    assert_eq!(result, Err("refused by the OS".to_string()));
    assert_eq!(backend.holder("f13"), Some(&reg("transcribe", "f13")));
    assert_eq!(backend.holder("f14"), Some(&reg("transcribe", "f14")));
    assert!(backend.holder("f5").is_none());
    Ok(())
}

#[test]
fn a_later_refusal_unregisters_the_additions_already_made() -> TestResult {
    let before = regs("transcribe", &["f13"]);
    let after = regs("transcribe", &["f5", "f6"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    let mut backend = backend_with(&before)?;
    backend.refused.push("f6".into());
    assert!(apply_registration_delta(&mut backend, &delta).is_err());
    assert!(backend.holder("f5").is_none());
    assert_eq!(backend.holder("f13"), Some(&reg("transcribe", "f13")));
    assert_eq!(backend.slots.len(), 1);
    Ok(())
}

#[test]
fn changing_an_inherited_conflict_keeps_the_other_action_registered() -> TestResult {
    let other = reg("transcribe_with_post_process", "option+space");
    let before = regs("transcribe", &["alt+space"]);
    let after = regs("transcribe", &["f5"]);
    let delta = plan_registration_delta(&before, &after, std::slice::from_ref(&other), tauri_slot);
    assert!(delta.released.is_empty());
    // Whichever action won the shared slot at startup, the other action
    // holds it after the change.
    for holder in [other.clone(), before[0].clone()] {
        let mut backend = backend_with(&[holder])?;
        apply_registration_delta(&mut backend, &delta)?;
        assert_eq!(backend.holder("option+space"), Some(&other));
        assert_eq!(backend.holder("f5"), Some(&reg("transcribe", "f5")));
    }
    Ok(())
}

#[test]
fn refused_change_of_an_inherited_conflict_leaves_the_shared_slot_alone() -> TestResult {
    let other = reg("transcribe_with_post_process", "option+space");
    let before = regs("transcribe", &["alt+space"]);
    let after = regs("transcribe", &["f5"]);
    let delta = plan_registration_delta(&before, &after, std::slice::from_ref(&other), tauri_slot);
    let mut backend = backend_with(std::slice::from_ref(&other))?;
    backend.refused.push("f5".into());
    assert!(apply_registration_delta(&mut backend, &delta).is_err());
    assert_eq!(backend.holder("option+space"), Some(&other));
    assert_eq!(backend.slots.len(), 1);
    Ok(())
}

#[test]
fn removing_a_duplicate_within_the_action_hands_the_slot_to_the_kept_one() -> TestResult {
    let before = regs("transcribe", &["alt+space", "option+space"]);
    let after = regs("transcribe", &["option+space"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    assert_eq!(delta.handovers.len(), 1);
    let mut backend = backend_with(&before[..1])?;
    apply_registration_delta(&mut backend, &delta)?;
    assert_eq!(
        backend.holder("option+space"),
        Some(&reg("transcribe", "option+space"))
    );
    Ok(())
}

#[test]
fn handy_keys_registrations_never_share_a_slot() {
    let other = reg("transcribe_with_post_process", "option+space");
    let before = regs("transcribe", &["alt+space"]);
    let after = regs("transcribe", &["f5"]);
    let delta = plan_registration_delta(&before, &after, &[other], |_, _| false);
    assert_eq!(delta.released, before);
    assert!(delta.handovers.is_empty());
}

#[test]
fn probe_checks_every_addition_and_leaves_nothing_registered() -> TestResult {
    let before = regs("transcribe_with_post_process", &["option+shift+space"]);
    let after = regs("transcribe_with_post_process", &["f5", "f6"]);
    let delta = plan_registration_delta(&before, &after, &[], tauri_slot);
    let mut backend = FakeBackend::default();
    probe_additions(&mut backend, &delta)?;
    assert_eq!(backend.probed, vec!["f5".to_string(), "f6".to_string()]);
    assert!(backend.slots.is_empty());
    backend.refused.push("f6".into());
    let refusal = probe_additions(&mut backend, &delta);
    assert_eq!(refusal, Err("refused by the OS".to_string()));
    Ok(())
}

#[test]
fn disabling_an_action_hands_its_shared_slot_back_to_the_other_action() -> TestResult {
    let transcribe = reg("transcribe", "alt+space");
    let post = regs("transcribe_with_post_process", &["option+space", "f13"]);
    let delta = plan_deactivation(&post, std::slice::from_ref(&transcribe), tauri_slot);
    assert_eq!(
        delta.released,
        regs("transcribe_with_post_process", &["f13"])
    );
    assert_eq!(delta.handovers.len(), 1);
    // Whichever action won the shared slot at startup, Transcribe holds it
    // once post-processing is disabled.
    for holder in [transcribe.clone(), post[0].clone()] {
        let mut backend = backend_with(&[holder, post[1].clone()])?;
        apply_registration_delta(&mut backend, &delta)?;
        assert_eq!(backend.holder("alt+space"), Some(&transcribe));
        assert!(backend.holder("f13").is_none());
        assert_eq!(backend.slots.len(), 1);
    }
    Ok(())
}

#[test]
fn enabling_an_action_leaves_a_slot_held_by_another_action_alone() -> TestResult {
    let transcribe = reg("transcribe", "alt+space");
    let post = regs("transcribe_with_post_process", &["option+space", "f13"]);
    let delta = plan_activation(&post, std::slice::from_ref(&transcribe), tauri_slot);
    assert_eq!(delta.added, regs("transcribe_with_post_process", &["f13"]));
    let mut backend = backend_with(std::slice::from_ref(&transcribe))?;
    assert_eq!(register_activation(&mut backend, &delta), Vec::new());
    assert_eq!(backend.holder("alt+space"), Some(&transcribe));
    assert_eq!(backend.holder("f13"), Some(&post[1]));
    Ok(())
}

#[test]
fn a_refused_activation_keeps_the_accepted_shortcuts_and_reports_the_refusal() {
    let post = regs("transcribe_with_post_process", &["f13", "f14", "f15"]);
    let delta = plan_activation(&post, &[], tauri_slot);
    let mut backend = FakeBackend::default();
    backend.refused.push("f14".into());
    let refusals = register_activation(&mut backend, &delta);
    let expected = Refusal {
        registration: post[1].clone(),
        reason: "refused by the OS".into(),
    };
    assert_eq!(refusals, vec![expected]);
    assert_eq!(backend.holder("f13"), Some(&post[0]));
    assert!(backend.holder("f14").is_none());
    assert_eq!(backend.holder("f15"), Some(&post[2]));
}

#[test]
fn a_failed_slot_release_does_not_register_the_survivor() -> TestResult {
    // Every shortcut is suspended while a recorder captures keys: the
    // shared slot is free, the release fails and resume registers the
    // survivor later.
    let other = reg("transcribe_with_post_process", "option+space");
    let before = regs("transcribe", &["alt+space"]);
    let after = regs("transcribe", &["f5"]);
    let delta = plan_registration_delta(&before, &after, std::slice::from_ref(&other), tauri_slot);
    let mut backend = FakeBackend::default();
    apply_registration_delta(&mut backend, &delta)?;
    assert!(backend.holder("option+space").is_none());
    assert_eq!(backend.holder("f5"), Some(&reg("transcribe", "f5")));
    Ok(())
}
