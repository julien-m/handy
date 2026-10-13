//! Binding preparation and compensation against a fake registry, alone and inside an ordered
//! settings write whose commit is refused: the user's previous shortcut always ends registered.
use super::{compensate, prepare, BindingRegistry};
use crate::settings::{KeyboardImplementation, ShortcutBinding};
use crate::settings_service::{
    edit::SettingsEdit,
    errors::{SettingsError, SettingsErrorCode},
    live::{LiveCommit, LiveSettings},
    protocol::Revision,
    sequence::{commit_prepared, ordered, LiveEffectsPort},
};
use std::{cell::RefCell, collections::BTreeSet};
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Registered shortcut texts; `refuse` is the text whose registration or validation fails.
#[derive(Default)]
struct FakeRegistry {
    registered: RefCell<BTreeSet<String>>,
    refuse_register: Option<&'static str>,
    refuse_validate: Option<&'static str>,
}
impl BindingRegistry for FakeRegistry {
    fn register(&self, binding: &ShortcutBinding) -> Result<(), String> {
        if self.refuse_register == Some(binding.current_binding.as_str()) {
            return Err("already taken".into());
        }
        self.registered
            .borrow_mut()
            .insert(binding.current_binding.clone());
        Ok(())
    }
    fn unregister(&self, binding: &ShortcutBinding) -> Result<(), String> {
        self.registered
            .borrow_mut()
            .remove(&binding.current_binding)
            .then_some(())
            .ok_or_else(|| "not registered".into())
    }
    fn validate(&self, text: &str, _: KeyboardImplementation) -> Result<(), String> {
        match self.refuse_validate {
            Some(refused) if refused == text => Err("unsupported key".into()),
            _ => Ok(()),
        }
    }
}
fn binding(text: &str) -> ShortcutBinding {
    ShortcutBinding {
        id: "transcribe".into(),
        name: "Transcribe".into(),
        description: String::new(),
        default_binding: "alt+space".into(),
        current_binding: text.into(),
    }
}
fn registry_with(previous: &str, refuse_register: Option<&'static str>) -> FakeRegistry {
    let registry = FakeRegistry {
        refuse_register,
        ..FakeRegistry::default()
    };
    registry.registered.borrow_mut().insert(previous.into());
    registry
}
fn registered(registry: &FakeRegistry) -> Vec<String> {
    registry.registered.borrow().iter().cloned().collect()
}
const TAURI: KeyboardImplementation = KeyboardImplementation::Tauri;

#[test]
fn preparation_swaps_the_registration_and_compensation_restores_it() {
    let registry = registry_with("alt+space", None);
    let (previous, updated) = (binding("alt+space"), binding("ctrl+k"));
    assert_eq!(prepare(&registry, &previous, &updated, TAURI), Ok(()));
    assert_eq!(registered(&registry), vec!["ctrl+k"]);
    compensate(&registry, &previous, &updated);
    assert_eq!(registered(&registry), vec!["alt+space"]);
}
#[test]
fn refused_registration_or_validation_keeps_the_previous_shortcut() {
    let (previous, updated) = (binding("alt+space"), binding("ctrl+k"));
    let taken = registry_with("alt+space", Some("ctrl+k"));
    assert!(prepare(&taken, &previous, &updated, TAURI).is_err());
    assert_eq!(registered(&taken), vec!["alt+space"]);
    let unsupported = FakeRegistry {
        refuse_validate: Some("ctrl+k"),
        ..registry_with("alt+space", None)
    };
    assert!(prepare(&unsupported, &previous, &updated, TAURI).is_err());
    assert_eq!(registered(&unsupported), vec!["alt+space"]);
}
/// Live state of bindings has no effect of its own: the preparation is the native change.
struct NoEffects;
impl LiveEffectsPort for NoEffects {
    fn apply(&self, _: &LiveCommit, _: &[&str]) -> Result<(), Vec<&'static str>> {
        Ok(())
    }
    fn notify(&self, _: Revision, _: &[&'static str]) {}
}
#[test]
fn refused_binding_commit_restores_the_previous_registration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = directory.path().join("settings_store.json");
    let live = LiveSettings::start_read_only(store, SettingsErrorCode::PersistenceFailed);
    let registry = registry_with("alt+space", None);
    let (previous, updated) = (binding("alt+space"), binding("ctrl+k"));
    let insert = |settings: &mut crate::settings::AppSettings| -> Result<(), SettingsError> {
        settings
            .bindings
            .insert("transcribe".into(), updated.clone());
        Ok(())
    };
    let result = ordered(&live, &NoEffects, || {
        let before = live.settings();
        prepare(&registry, &previous, &updated, TAURI).map_err(|_| "prepare refused")?;
        let edit = SettingsEdit::Apply(&insert);
        commit_prepared(&live, &NoEffects, "bindings", &edit, &before, || {
            compensate(&registry, &previous, &updated)
        })
        .map_err(|error| error.code())
        .err()
        .ok_or("read-only commit accepted")
    })?;
    assert_eq!(result, SettingsErrorCode::PersistenceFailed);
    assert_eq!(registered(&registry), vec!["alt+space"]);
    Ok(())
}
