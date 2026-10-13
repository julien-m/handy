//! One shortcut change as an ordered settings write: the native registration is prepared from the
//! settings published when the write starts (unregister the old shortcut, validate and register
//! the new one), the binding is committed on the latest settings, and a refused commit restores
//! the previous registration. Registration sits behind `BindingRegistry` so the preparation and
//! the compensation are testable without a running app.
use super::{BindingResponse, ShortcutBinding};
use crate::settings::{self, AppSettings, KeyboardImplementation};
use crate::settings_service::{
    command_error::SettingsCommandError, errors::ValidationReason, tauri_adapter as live_settings,
};
use log::{error, warn};
use tauri::AppHandle;

/// Native shortcut registration used by a binding change.
pub(super) trait BindingRegistry {
    /// Register `binding` with the active keyboard implementation.
    fn register(&self, binding: &ShortcutBinding) -> Result<(), String>;
    /// Unregister `binding` from the active keyboard implementation.
    fn unregister(&self, binding: &ShortcutBinding) -> Result<(), String>;
    /// Refuse a shortcut text the implementation cannot register.
    fn validate(&self, text: &str, implementation: KeyboardImplementation) -> Result<(), String>;
}
/// The running app's registration (Tauri global shortcut or HandyKeys).
pub(super) struct AppRegistry<'a>(pub(super) &'a AppHandle);
impl BindingRegistry for AppRegistry<'_> {
    fn register(&self, binding: &ShortcutBinding) -> Result<(), String> {
        super::register_shortcut(self.0, binding.clone())
    }
    fn unregister(&self, binding: &ShortcutBinding) -> Result<(), String> {
        super::unregister_shortcut(self.0, binding.clone())
    }
    fn validate(&self, text: &str, implementation: KeyboardImplementation) -> Result<(), String> {
        super::validate_shortcut_for_implementation(text, implementation)
    }
}

/// Change one shortcut as one ordered write (see the module documentation).
///
/// # Errors
/// An empty binding, or a refused commit (the previous registration is then restored).
pub(super) fn change(
    app: &AppHandle,
    id: &str,
    binding: String,
) -> Result<BindingResponse, SettingsCommandError> {
    require_binding_text(&binding)?;
    live_settings::ordered(app, || {
        let settings = settings::get_settings(app);
        let Some(previous) = binding_or_default(&settings, id) else {
            let error = format!("Binding with id '{}' not found in defaults", id);
            warn!("change_binding error: {}", error);
            return Ok(BindingResponse::failed(error));
        };
        let mut updated = previous.clone();
        updated.current_binding = binding;
        // The cancel binding is registered dynamically by the recording lifecycle, never here.
        let registry = (id != "cancel").then_some(AppRegistry(app));
        if let Some(registry) = &registry {
            let implementation = settings.keyboard_implementation;
            if let Err(error) = prepare(registry, &previous, &updated, implementation) {
                return Ok(BindingResponse::failed(error));
            }
        }
        commit(app, id, &settings, (&previous, &updated), registry.as_ref())?;
        Ok(BindingResponse::succeeded(updated))
    })
}
/// Reject empty bindings: every shortcut must have a value.
fn require_binding_text(binding: &str) -> Result<(), SettingsCommandError> {
    if binding.trim().is_empty() {
        return Err(SettingsCommandError::invalid(
            "bindings",
            ValidationReason::CanonicalValue,
        ));
    }
    Ok(())
}
/// Commit the binding on the latest settings; a refusal undoes the prepared registration.
fn commit(
    app: &AppHandle,
    id: &str,
    before: &AppSettings,
    (previous, updated): (&ShortcutBinding, &ShortcutBinding),
    registry: Option<&AppRegistry<'_>>,
) -> Result<(), SettingsCommandError> {
    let edit = |settings: &mut AppSettings| {
        settings.bindings.insert(id.to_string(), updated.clone());
        Ok(())
    };
    live_settings::commit_prepared_edit(app, "bindings", &edit, before, || {
        if let Some(registry) = registry {
            compensate(registry, previous, updated);
        }
    })?;
    // Synchronize any active Secure Input shadows with the committed binding.
    crate::secure_input::reconcile_fallback(app);
    Ok(())
}
/// Stored binding, or its default when a newer build added it after the store was written.
fn binding_or_default(settings: &AppSettings, id: &str) -> Option<ShortcutBinding> {
    settings.bindings.get(id).cloned().or_else(|| {
        let default = settings::get_default_settings().bindings.get(id).cloned();
        if default.is_some() {
            warn!(
                "Binding '{}' not found in settings, creating from defaults",
                id
            );
        }
        default
    })
}
/// Swap the native registration from `previous` to `updated`; on refusal the previous shortcut is
/// registered again and the user-facing reason is returned.
pub(super) fn prepare(
    registry: &impl BindingRegistry,
    previous: &ShortcutBinding,
    updated: &ShortcutBinding,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    if let Err(e) = registry.unregister(previous) {
        error!("change_binding error: Failed to unregister shortcut: {}", e);
    }
    if let Err(e) = registry.validate(&updated.current_binding, implementation) {
        warn!("change_binding validation error: {}", e);
        restore(registry, previous);
        return Err(e);
    }
    if let Err(e) = registry.register(updated) {
        let error = format!("Failed to register shortcut: {}", e);
        error!("change_binding error: {}", error);
        restore(registry, previous);
        return Err(error);
    }
    Ok(())
}
/// Undo `prepare` after a refused commit: the stored binding is still `previous`.
pub(super) fn compensate(
    registry: &impl BindingRegistry,
    previous: &ShortcutBinding,
    updated: &ShortcutBinding,
) {
    if let Err(e) = registry.unregister(updated) {
        error!(
            "Failed to unregister refused binding '{}': {}",
            updated.id, e
        );
    }
    restore(registry, previous);
}
/// Best-effort re-register of the previous binding after a failed change,
/// so a failure leaves the user's shortcut working exactly as before.
fn restore(registry: &impl BindingRegistry, binding: &ShortcutBinding) {
    if let Err(e) = registry.register(binding) {
        error!(
            "Failed to restore previous binding '{}' ({}): {}",
            binding.id, binding.current_binding, e
        );
    }
}
#[cfg(test)]
#[path = "binding_change_tests.rs"]
mod tests;
