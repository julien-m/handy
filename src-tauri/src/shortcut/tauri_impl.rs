//! Tauri global-shortcut implementation
//!
//! This module provides shortcut functionality using Tauri's built-in
//! global-shortcut plugin.

use log::{debug, error, warn};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::handler::handle_shortcut_event;
use super::identity::ShortcutRegistration;

/// Initialize shortcuts using Tauri's global-shortcut plugin
pub fn init_shortcuts(app: &AppHandle) {
    for registration in super::startup_registrations(app) {
        if let Err(e) = register_shortcut(app, &registration) {
            error!(
                "Failed to register shortcut {} ({}) during init: {}",
                registration.action_id, registration.shortcut_id, e
            );
        }
    }
}

/// Validate a shortcut string for the Tauri global-shortcut implementation.
/// Tauri requires at least one non-modifier key and doesn't support the fn key.
pub fn validate_shortcut(raw: &str) -> Result<(), String> {
    if raw.trim().is_empty() {
        return Err("Shortcut cannot be empty".into());
    }

    let modifiers = [
        "ctrl", "control", "shift", "alt", "option", "meta", "command", "cmd", "super", "win",
        "windows",
    ];

    // Check for fn key which Tauri doesn't support
    let parts: Vec<String> = raw.split('+').map(|p| p.trim().to_lowercase()).collect();
    for part in &parts {
        if part == "fn" || part == "function" {
            return Err("The 'fn' key is not supported by Tauri global shortcuts".into());
        }
    }

    // Check for at least one non-modifier key
    let has_non_modifier = parts.iter().any(|part| !modifiers.contains(&part.as_str()));

    if !has_non_modifier {
        return Err("Tauri shortcuts must include a main key (letter, number, F-key, etc.) in addition to modifiers".into());
    }

    // The name check above passes side-specific modifiers such as
    // `option_left`, which the handy-keys recorder saves but the accelerator
    // parser rejects. Parse here so a binding carried over from handy-keys is
    // reset to the default instead of failing to register.
    raw.parse::<Shortcut>()
        .map(|_| ())
        .map_err(|e| format!("Failed to parse shortcut '{}': {}", raw, e))
}

/// Parses `accelerator` after the Tauri-specific validation.
fn parse_registrable(accelerator: &str) -> Result<Shortcut, String> {
    validate_shortcut(accelerator).map_err(|e| {
        warn!(
            "register_tauri_shortcut validation error for binding '{}': {}",
            accelerator, e
        );
        e
    })?;
    accelerator.parse::<Shortcut>().map_err(|e| {
        let error_msg = format!("Failed to parse shortcut '{}': {}", accelerator, e);
        error!("register_tauri_shortcut parse error: {}", error_msg);
        error_msg
    })
}

/// Register one shortcut of one action using Tauri's global-shortcut plugin.
/// The callback carries the registration identity, so the handler knows
/// which shortcut of the action fired.
///
/// # Errors
/// Returns why the shortcut is invalid, already registered by Handy, or
/// refused by the OS (the plugin's reason).
pub fn register_shortcut(
    app: &AppHandle,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    let shortcut = parse_registrable(&registration.accelerator)?;

    // Prevent duplicate registrations that would silently shadow one another
    if app.global_shortcut().is_registered(shortcut) {
        let error_msg = format!("Shortcut '{}' is already in use", registration.accelerator);
        warn!("register_tauri_shortcut duplicate error: {}", error_msg);
        return Err(error_msg);
    }

    let identity = registration.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |app_handle, scut, event| {
            if scut == &shortcut {
                dispatch_event(app_handle, &identity, scut, event.state);
            }
        })
        .map_err(|e| {
            let error_msg = format!(
                "Couldn't register shortcut '{}': {}",
                registration.accelerator, e
            );
            error!("register_tauri_shortcut registration error: {}", error_msg);
            error_msg
        })
}

/// Checks that the OS accepts `accelerator`, then releases it at once. The
/// probe registers no handler, so a press during the check reaches no
/// action; it is swallowed during the two main-thread round trips.
///
/// A release that still fails after one retry leaves the combination
/// registered without a handler until Handy restarts: it triggers nothing
/// but is captured system-wide, and Handy refuses it as already in use. The
/// plugin reports the failure before dropping its entry, so the entry stays.
///
/// # Errors
/// Returns why the shortcut is invalid, already registered by Handy or
/// refused by the OS, or that the probe could not be released (in which
/// case the shortcut must not be treated as available).
pub fn probe_shortcut(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let shortcut = parse_registrable(accelerator)?;
    if app.global_shortcut().is_registered(shortcut) {
        return Err(format!("Shortcut '{}' is already in use", accelerator));
    }
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Couldn't register shortcut '{}': {}", accelerator, e))?;
    let release = || app.global_shortcut().unregister(shortcut);
    release()
        .or_else(|first| {
            warn!("Retrying the release of probe '{}': {}", accelerator, first);
            release()
        })
        .map_err(|e| {
            let error_msg = format!(
                "Couldn't release the availability check of '{}': {}",
                accelerator, e
            );
            error!("probe_tauri_shortcut cleanup error: {}", error_msg);
            error_msg
        })
}

/// Whether two accelerators occupy the same native registration: the
/// plugin keeps one registration per parsed combination, so `alt+space` and
/// `option+space` share one.
pub fn same_registration_slot(a: &str, b: &str) -> bool {
    match (a.parse::<Shortcut>(), b.parse::<Shortcut>()) {
        (Ok(first), Ok(second)) => first.id() == second.id(),
        _ => false,
    }
}

fn dispatch_event(
    app: &AppHandle,
    registration: &ShortcutRegistration,
    shortcut: &Shortcut,
    state: ShortcutState,
) {
    let shortcut_string = shortcut.into_string();
    // Mirrors the handy-keys event log line; the distinct prefix makes it
    // possible to tell which backend fired a shortcut (e.g. when diagnosing
    // the Secure Input fallback)
    debug!(
        "tauri global-shortcut event: binding={}, shortcut={}, state={:?}",
        registration.action_id, shortcut_string, state
    );
    handle_shortcut_event(
        app,
        registration,
        &shortcut_string,
        state == ShortcutState::Pressed,
    );
}

/// Unregister one shortcut from Tauri's global-shortcut plugin
///
/// # Errors
/// Returns the parse or plugin error.
pub fn unregister_shortcut(
    app: &AppHandle,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    let accelerator = &registration.accelerator;
    let shortcut = accelerator.parse::<Shortcut>().map_err(|e| {
        let error_msg = format!(
            "Failed to parse shortcut '{}' for unregistration: {}",
            accelerator, e
        );
        error!("unregister_tauri_shortcut parse error: {}", error_msg);
        error_msg
    })?;

    app.global_shortcut().unregister(shortcut).map_err(|e| {
        let error_msg = format!("Failed to unregister shortcut '{}': {}", accelerator, e);
        error!("unregister_tauri_shortcut error: {}", error_msg);
        error_msg
    })
}

#[cfg(test)]
mod tests {
    use super::{same_registration_slot, validate_shortcut};

    #[test]
    fn aliases_share_one_native_registration() {
        assert!(same_registration_slot("alt+space", "option+space"));
        assert!(same_registration_slot("shift+cmd+a", "super+shift+a"));
        assert!(!same_registration_slot("alt+space", "ctrl+space"));
        assert!(!same_registration_slot("option_left+space", "alt+space"));
    }

    #[test]
    fn rejects_side_specific_modifiers_the_parser_cannot_register() {
        for raw in ["option_left+space", "ctrl_right+space"] {
            assert!(validate_shortcut(raw).is_err(), "{raw} should be rejected");
        }
    }

    #[test]
    fn accepts_the_default_shortcuts() {
        for raw in [
            "option+space",
            "option+shift+space",
            "ctrl+space",
            "ctrl+shift+space",
            "alt+space",
            "escape",
        ] {
            assert_eq!(validate_shortcut(raw), Ok(()), "{raw} should be accepted");
        }
    }
}
