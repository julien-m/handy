//! Settings commands that edit the shortcut list of an action.
//!
//! Each edit is planned by the pure [`action_shortcuts`] rules (limits,
//! syntax, shared conflict check), then committed as one ordered write of the
//! settings service: the registrations are prepared with the active keyboard
//! implementation before the commit (see [`super::bindings_change`]) and undone
//! if the commit is refused, so a refused shortcut is never stored.

use log::warn;
use serde::Serialize;
use specta::Type;
use std::cell::RefCell;
use std::rc::Rc;
use tauri::AppHandle;

use super::action_shortcuts::{plan_edit, EditContext, ShortcutEdit, ShortcutEditError};
use super::conflicts::{find_stored_conflicts, inventory_of, StoredShortcutConflict};
use super::identity::ShortcutRegistration;
use super::prompt_targets::{empty_binding, is_live_target, prompt_of, store_binding};
use super::registration_delta::RegistrationPort;
use super::tauri_impl;
use crate::settings::{self, AppSettings, KeyboardImplementation, ShortcutBinding};
use crate::settings_service::{
    command_error::SettingsCommandError, tauri_adapter as live_settings,
};

/// Outcome of a shortcut list edit: the saved binding, or why nothing was
/// saved.
#[derive(Debug, Serialize, Type)]
pub struct ShortcutEditResponse {
    pub binding: Option<ShortcutBinding>,
    pub error: Option<ShortcutEditError>,
}

// @spec FR-001: Add, remove, replace, reset an action's shortcuts — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-001
#[tauri::command]
#[specta::specta]
/// Applies one edit to the shortcut list of action `id` and saves it, on the blocking pool.
/// Every refusal is in the response, typed: a refusal of the edit, or `settings_refused` when
/// the settings service could not save it (nothing is then saved either).
pub async fn edit_binding_shortcuts(
    app: AppHandle,
    id: String,
    edit: ShortcutEdit,
) -> ShortcutEditResponse {
    let action = id.clone();
    let result = live_settings::blocking(move || apply_shortcut_edit(&app, &id, &edit))
        .await
        .unwrap_or_else(|error| Err(ShortcutEditError::SettingsRefused { error }));
    match result {
        Ok(binding) => ShortcutEditResponse {
            binding: Some(binding),
            error: None,
        },
        Err(error) => {
            warn!("Shortcut edit for '{}' refused: {:?}", action, error);
            ShortcutEditResponse {
                binding: None,
                error: Some(error),
            }
        }
    }
}

// @spec AC-004: Validation before saving — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
#[tauri::command]
#[specta::specta]
/// Checks one edit of action `id` against the published settings without registering or saving
/// anything, so the settings can show a recorded shortcut as valid or invalid before it is added.
/// The system's own registration check runs only when the edit is applied.
pub fn check_binding_shortcut_edit(
    app: AppHandle,
    id: String,
    edit: ShortcutEdit,
) -> Option<ShortcutEditError> {
    check_shortcut_edit(&settings::get_settings(&app), &id, &edit)
}

/// The refusal `edit` of action `id` would get on `settings`, if any (limits, syntax, shared
/// conflict check). Pure: nothing is registered or saved.
pub(crate) fn check_shortcut_edit(
    settings: &AppSettings,
    id: &str,
    edit: &ShortcutEdit,
) -> Option<ShortcutEditError> {
    plan_shortcut_edit(settings, id, edit).err()
}

// @spec AC-002: Legacy conflicts kept and flagged — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
#[tauri::command]
#[specta::specta]
/// Stored shortcuts that conflict with another stored shortcut. They stay
/// registered as before; the settings flag them until the user changes them.
pub fn get_shortcut_conflicts(app: AppHandle) -> Vec<StoredShortcutConflict> {
    let settings = settings::get_settings(&app);
    find_stored_conflicts(
        &inventory_of(&settings.bindings),
        settings.keyboard_implementation,
    )
}

/// The stored binding of `id`, or its default when the store lacks it; a prompt target (005) of
/// an existing prompt without shortcuts is its empty binding.
pub(crate) fn current_binding_or_default(
    settings: &AppSettings,
    id: &str,
) -> Option<ShortcutBinding> {
    settings
        .bindings
        .get(id)
        .cloned()
        .or_else(|| settings::get_default_settings().bindings.remove(id))
        .or_else(|| {
            let prompt_id = prompt_of(id).filter(|_| is_live_target(settings, id))?;
            Some(empty_binding(prompt_id))
        })
}

/// The current list of action `id` and the list `edit` would store, planned on `settings`.
///
/// # Errors
/// Any planning refusal, or `ActionNotFound`.
pub(crate) fn plan_shortcut_edit(
    settings: &AppSettings,
    id: &str,
    edit: &ShortcutEdit,
) -> Result<(ShortcutBinding, ShortcutBinding), ShortcutEditError> {
    let current = current_binding_or_default(settings, id).ok_or_else(|| {
        ShortcutEditError::ActionNotFound {
            action_id: id.to_string(),
        }
    })?;
    let inventory = inventory_of(&settings.bindings);
    let context = EditContext {
        inventory: &inventory,
        implementation: settings.keyboard_implementation,
    };
    let planned = plan_edit(&current, edit, &context)?;
    Ok((current, planned))
}

/// Plans `edit` on the published settings, then commits the list as one ordered write whose
/// registrations are prepared first (see [`super::live_change::bindings`]): one revision, a no-op
/// commits nothing.
///
/// # Errors
/// Outer: a refusal of the settings service (a dictation session in progress, persistence);
/// the registrations are then as before. Inner: any planning refusal, `ActionNotFound`, or
/// `RegistrationFailed` with the system's reason; nothing is then registered or saved.
pub(crate) fn apply_shortcut_edit(
    app: &AppHandle,
    id: &str,
    edit: &ShortcutEdit,
) -> Result<Result<ShortcutBinding, ShortcutEditError>, SettingsCommandError> {
    live_settings::ordered(app, || {
        let before = settings::get_settings(app);
        let (current, planned) = match plan_shortcut_edit(&before, id, edit) {
            Ok(plan) => plan,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if planned.shortcuts() == current.shortcuts() {
            return Ok(Ok(current));
        }
        commit_planned(app, planned)
    })
}

/// Commit `planned` with its registrations prepared first; a system refusal of the preparation
/// is the inner `RegistrationFailed` with the system's reason.
fn commit_planned(
    app: &AppHandle,
    planned: ShortcutBinding,
) -> Result<Result<ShortcutBinding, ShortcutEditError>, SettingsCommandError> {
    let refusal = Rc::new(RefCell::new(None));
    let native = super::live_change::bindings(app.clone(), Some(Rc::clone(&refusal)));
    let apply = |settings: &mut AppSettings| {
        store_binding(settings, planned.clone());
        Ok(())
    };
    match live_settings::edit_prepared(app, "bindings", &apply, native) {
        Ok(_) => Ok(Ok(planned)),
        Err(error) => match refusal.take() {
            Some(message) => Ok(Err(ShortcutEditError::RegistrationFailed { message })),
            None => Err(error),
        },
    }
}

/// How the edit of action `id` is prepared with `settings` in place (see [`preparation_for`]).
pub(crate) fn preparation_of(settings: &AppSettings, id: &str) -> Preparation {
    let state = ActionState {
        is_statically_registered: super::is_statically_registered(id, settings),
        is_cancel_registered: super::is_cancel_registered(),
        has_dynamic_cancel: !cfg!(target_os = "linux"),
    };
    preparation_for(id, state, settings.keyboard_implementation)
}

/// How an edit prepares the keyboard backend before it is saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Preparation {
    /// Register the delta with rollback: the action is registered now.
    ApplyDelta,
    /// Only check additions with the OS: the action is registered later.
    Probe,
    /// No backend check before saving.
    Nothing,
}

/// Where an action's registrations stand when it is edited.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActionState {
    /// Registered outside recording (see `is_statically_registered`).
    pub is_statically_registered: bool,
    /// The cancel shortcut is registered right now (a recording runs).
    pub is_cancel_registered: bool,
    /// The platform registers cancel dynamically (every platform but Linux).
    pub has_dynamic_cancel: bool,
}

// @spec AC-004: OS check of an inactive action's shortcut — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-004
/// Chooses how an edit of `action_id` is prepared. A registered action gets
/// the delta. With the Tauri backend, an inactive action is probed: disabled
/// post-processing, and cancel outside recording where it is registered
/// dynamically. HandyKeys performs no OS registration; while a recording
/// runs, cancel keeps its registration and the new list applies at the next
/// recording; Linux never registers cancel.
pub(crate) fn preparation_for(
    action_id: &str,
    state: ActionState,
    implementation: KeyboardImplementation,
) -> Preparation {
    if state.is_statically_registered {
        return Preparation::ApplyDelta;
    }
    // 014: a key read during a dictation is never registered, so there is nothing to check.
    if crate::dictation_keys::is_dictation_key(action_id) {
        return Preparation::Nothing;
    }
    let can_probe = match action_id {
        "cancel" => state.has_dynamic_cancel && !state.is_cancel_registered,
        _ => true,
    };
    if implementation == KeyboardImplementation::Tauri && can_probe {
        Preparation::Probe
    } else {
        Preparation::Nothing
    }
}

/// A post-processing shortcut the backend refused when post-processing was
/// enabled. It stays stored and editable; the settings flag its row.
/// `reason` is the backend's message, in English.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct RefusedShortcut {
    pub action_id: String,
    pub shortcut: String,
    pub reason: String,
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Toggles post-processing on the active keyboard backend; see
/// [`toggle_post_process`].
pub(crate) fn toggle_post_process_on_backend(
    app: &AppHandle,
    settings: &AppSettings,
    enabled: bool,
) -> (AppSettings, Vec<RefusedShortcut>) {
    toggle_post_process(&mut BackendRegistry { app }, settings, enabled)
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Sets `post_process_enabled` as one ordered write whose registrations are prepared first (see
/// [`super::live_change::post_process`]); a refused commit toggles them back. Returns the
/// shortcuts the system refused, which stay stored and editable.
///
/// # Errors
/// A refusal of the settings service (a dictation session in progress, persistence); the
/// registrations are then as before.
pub(crate) fn set_post_process_enabled(
    app: &AppHandle,
    enabled: bool,
) -> Result<Vec<RefusedShortcut>, SettingsCommandError> {
    let refused: super::live_change::RefusalSink = Rc::default();
    let native = super::live_change::post_process(app.clone(), Some(Rc::clone(&refused)));
    let value = serde_json::Value::Bool(enabled);
    live_settings::set_prepared_with(app, "post_process_enabled", value, native)?;
    Ok(refused.take())
}

/// Log each post-processing shortcut the system refused at activation.
pub(crate) fn log_refusals(refused: &[RefusedShortcut]) {
    for refusal in refused {
        warn!(
            "Post-processing shortcut '{}' refused by the system: {}",
            refusal.shortcut, refusal.reason
        );
    }
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
/// Settings with post-processing set to `enabled`, after registering or
/// releasing its shortcuts through `port`. The toggle always applies, as
/// before this feature, so the post-processing settings (where its shortcuts
/// are edited) stay reachable: a shortcut the backend refuses is returned,
/// not a reason to keep post-processing off. An unchanged value touches no
/// registration.
pub(crate) fn toggle_post_process(
    port: &mut impl RegistrationPort,
    settings: &AppSettings,
    enabled: bool,
) -> (AppSettings, Vec<RefusedShortcut>) {
    let refused = match (settings.post_process_enabled, enabled) {
        (false, true) => super::gated::register_post_process(port, settings),
        (true, false) => {
            super::gated::release_post_process(port, settings);
            Vec::new()
        }
        _ => Vec::new(),
    };
    let mut updated = settings.clone();
    updated.post_process_enabled = enabled;
    (updated, refused)
}

/// The active keyboard backend, as used by shortcut edits.
pub(crate) struct BackendRegistry<'a> {
    pub app: &'a AppHandle,
}

impl RegistrationPort for BackendRegistry<'_> {
    fn register(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        super::register_registration(self.app, registration)
    }

    fn unregister(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        super::unregister_registration(self.app, registration)
    }

    fn probe(&mut self, registration: &ShortcutRegistration) -> Result<(), String> {
        tauri_impl::probe_shortcut(self.app, &registration.accelerator)
    }
}

/// English description of a refusal for the legacy `change_binding`
/// response, naming the owning action of a conflict.
pub(crate) fn describe_edit_error(app: &AppHandle, error: &ShortcutEditError) -> String {
    match error {
        ShortcutEditError::ShortcutConflict { conflict } => {
            let owner = settings::get_settings(app)
                .bindings
                .get(&conflict.owning_action)
                .map(|binding| binding.name.clone())
                .unwrap_or_else(|| conflict.owning_action.clone());
            format!("Conflicts with '{}' of {}", conflict.owning_shortcut, owner)
        }
        ShortcutEditError::RegistrationFailed { message }
        | ShortcutEditError::InvalidShortcut { message } => message.clone(),
        ShortcutEditError::SettingsRefused { error } => error.message.clone(),
        other => format!("{:?}", other),
    }
}

#[cfg(test)]
mod tests;
