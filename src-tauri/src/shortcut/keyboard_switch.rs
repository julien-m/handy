//! Keyboard implementation switch: the lists repaired for the destination implementation (pure,
//! committed with the implementation in one revision) and the move of every registration from
//! one implementation to the other, from explicit settings so that it runs before the commit
//! (prepared change) and back again when the commit is refused.

use log::{error, info, warn};
use tauri::AppHandle;

use super::identity::ShortcutRegistration;
use super::{conflicts, handy_keys, implementation_switch};
use crate::settings::{self, AppSettings, KeyboardImplementation};

// @spec AC-002: Switch repair keeps inherited conflicts — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Repair the lists of `settings` for its keyboard implementation (see
/// [`implementation_switch::plan_switch_repair`]). Returns the ids of the repaired bindings.
/// Pure: it may run twice (preview, then the locked commit).
pub(crate) fn repair_lists(settings: &mut AppSettings) -> Vec<String> {
    let implementation = settings.keyboard_implementation;
    let repair = implementation_switch::plan_switch_repair(
        &settings.bindings,
        &settings::get_default_settings().bindings,
        |id| super::is_statically_registered(id, settings),
        |raw| switch_form(raw, implementation),
    );
    settings.bindings = repair.bindings;
    repair.repaired
}

/// Comparable form of `raw` for the switch repair, `None` (logged) when
/// `implementation` cannot register it.
fn switch_form(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Option<conflicts::NormalizedCombination> {
    implementation_switch::destination_form(raw, implementation)
        .map_err(|e| {
            info!(
                "Shortcut '{}' is invalid for {:?}: {}. Resetting or removing it.",
                raw, implementation, e
            );
        })
        .ok()
}

/// Registrations of `settings` registered outside recording, using the default binding for any
/// action missing from the store.
pub(crate) fn static_registrations(settings: &AppSettings) -> Vec<ShortcutRegistration> {
    settings::get_default_settings()
        .bindings
        .into_iter()
        .filter(|(id, _)| super::is_statically_registered(id, settings))
        .flat_map(|(id, default_binding)| {
            let binding = settings
                .bindings
                .get(&id)
                .cloned()
                .unwrap_or(default_binding);
            ShortcutRegistration::all_of(&binding)
        })
        .collect()
}

/// Move every registration from `from` (its implementation and lists) to `to`. Order matters:
/// the old backend releases every shortcut and the Carbon shadows are removed before the new
/// backend registers them.
///
/// # Errors
/// HandyKeys could not start; the registrations of `from` are then restored.
pub(crate) fn switch_registrations(
    app: &AppHandle,
    from: &AppSettings,
    to: &AppSettings,
) -> Result<(), String> {
    info!(
        "Switching keyboard implementation from {:?} to {:?}",
        from.keyboard_implementation, to.keyboard_implementation
    );
    register_all(app, from, false);
    if to.keyboard_implementation == KeyboardImplementation::Tauri {
        crate::secure_input::reconcile_fallback_with(app, to);
    }
    if to.keyboard_implementation == KeyboardImplementation::HandyKeys {
        if let Err(e) = handy_keys::init_state(app) {
            error!("Failed to initialize HandyKeys: {}", e);
            register_all(app, from, true);
            return Err(format!("Failed to initialize HandyKeys: {}", e));
        }
    }
    register_all(app, to, true);
    Ok(())
}

/// Register (or release) every static registration of `settings` with its implementation;
/// failures are logged, as at startup.
fn register_all(app: &AppHandle, settings: &AppSettings, register: bool) {
    let implementation = settings.keyboard_implementation;
    for registration in static_registrations(settings) {
        let result = if register {
            super::register_with(app, implementation, &registration)
        } else {
            super::unregister_with(app, implementation, &registration)
        };
        if let Err(e) = result {
            warn!(
                "Shortcut '{}' of '{}' not {} with {:?}: {}",
                registration.shortcut_id,
                registration.action_id,
                if register { "registered" } else { "released" },
                implementation,
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tauri_settings() -> AppSettings {
        let mut settings = settings::get_default_settings();
        settings.keyboard_implementation = KeyboardImplementation::Tauri;
        settings.post_process_enabled = false;
        settings
    }

    #[test]
    fn repair_resets_what_the_destination_cannot_register_and_keeps_the_rest() {
        let mut settings = tauri_settings();
        if let Some(transcribe) = settings.bindings.get_mut("transcribe") {
            transcribe.current_binding = "option".into();
            transcribe.additional_bindings = vec!["f13".into(), "fn+f14".into()];
        }
        let repaired = repair_lists(&mut settings);
        assert_eq!(repaired, vec!["transcribe".to_string()]);
        let transcribe = &settings.bindings["transcribe"];
        let default = transcribe.default_binding.clone();
        assert_eq!(transcribe.shortcuts(), vec![default, "f13".to_string()]);
    }

    #[test]
    fn valid_lists_are_not_repaired_twice() {
        let mut settings = tauri_settings();
        if let Some(transcribe) = settings.bindings.get_mut("transcribe") {
            transcribe.additional_bindings = vec!["fn+f14".into()];
        }
        assert_eq!(repair_lists(&mut settings), vec!["transcribe".to_string()]);
        let repaired = settings.clone();
        assert!(repair_lists(&mut settings).is_empty());
        assert_eq!(
            settings.bindings["transcribe"].shortcuts(),
            repaired.bindings["transcribe"].shortcuts()
        );
    }

    #[test]
    fn static_registrations_follow_the_post_processing_switch() {
        let mut settings = tauri_settings();
        let actions = |settings: &AppSettings| {
            let mut ids: Vec<String> = static_registrations(settings)
                .into_iter()
                .map(|registration| registration.action_id)
                .collect();
            ids.sort();
            ids.dedup();
            ids
        };
        assert!(!actions(&settings).contains(&"transcribe_with_post_process".to_string()));
        settings.post_process_enabled = true;
        assert!(actions(&settings).contains(&"transcribe_with_post_process".to_string()));
        assert!(!actions(&settings).contains(&"cancel".to_string()));
    }
}
