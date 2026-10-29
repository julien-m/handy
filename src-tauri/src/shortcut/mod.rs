//! Keyboard shortcut management module
//!
//! This module provides a unified interface for keyboard shortcuts with
//! multiple backend implementations:
//!
//! - `tauri`: Uses Tauri's built-in global-shortcut plugin
//! - `handy_keys`: Uses the handy-keys library for more control
//!
//! The active implementation is determined by the `keyboard_implementation`
//! setting and can be changed at runtime.

pub mod action_shortcuts;
pub(crate) mod bindings_change;
pub mod conflicts;
pub mod edits;
#[cfg(test)]
mod fake_backend;
mod gated;
mod handler;
pub mod handy_keys;
pub mod identity;
mod implementation_switch;
mod keyboard_switch;
pub(crate) mod live_change;
#[cfg(test)]
mod migration_tests;
pub mod prompt_targets;
mod registration_delta;
pub mod tauri_impl;

use log::{debug, error, warn};
use serde::Serialize;
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::settings::APPLE_INTELLIGENCE_DEFAULT_MODEL_ID;
use crate::settings::{
    self, get_settings, ChineseScript, KeyboardImplementation, ShortcutActivation, ShortcutBinding,
    Theme, VadBackend, APPLE_INTELLIGENCE_PROVIDER_ID,
};
use crate::settings_service::{
    command_error::SettingsCommandError,
    errors::{SettingsError, ValidationReason},
    tauri_adapter as live_settings,
    ui_value::UiValue,
};
use identity::ShortcutRegistration;

// Note: Commands are accessed via shortcut::handy_keys:: in lib.rs

/// Initialize shortcuts using the configured implementation
pub fn init_shortcuts(app: &AppHandle) {
    let user_settings = settings::load_or_create_app_settings(app);

    // Check which implementation to use
    match user_settings.keyboard_implementation {
        KeyboardImplementation::Tauri => {
            tauri_impl::init_shortcuts(app);
        }
        KeyboardImplementation::HandyKeys => {
            if let Err(e) = handy_keys::init_shortcuts(app) {
                error!("Failed to initialize handy-keys shortcuts: {}", e);
                // Fall back to Tauri implementation and persist this fallback
                warn!("Falling back to Tauri global shortcut implementation and saving fallback to settings");

                // Update settings to persist the fallback so we don't retry HandyKeys on next launch
                persist_tauri_fallback(app);

                tauri_impl::init_shortcuts(app);
            }
        }
    }
}

/// Whether the recording lifecycle currently wants the cancel shortcut.
/// Written synchronously by start/stop, so it always holds the latest request.
static CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Whether the cancel shortcut is actually registered with the backend.
/// The lock also serializes reconciliation passes.
#[cfg(not(target_os = "linux"))]
static CANCEL_REGISTERED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

/// Whether the cancel shortcut is registered right now (a recording runs).
/// Always false on Linux, which never registers it. Never blocks: a
/// reconcile pass holds the lock while it registers or releases cancel and
/// may wait for the main thread, where settings commands run, so a held
/// lock counts as registered (no probe while cancel changes state).
pub(crate) fn is_cancel_registered() -> bool {
    #[cfg(target_os = "linux")]
    {
        false
    }
    #[cfg(not(target_os = "linux"))]
    {
        match CANCEL_REGISTERED.try_lock() {
            Ok(registered) => *registered,
            Err(std::sync::TryLockError::Poisoned(e)) => *e.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => true,
        }
    }
}

/// Register the cancel shortcut (called when recording starts)
pub fn register_cancel_shortcut(app: &AppHandle) {
    // Track recording lifecycle independently of the current implementation so
    // switching implementations mid-recording cannot leave stale fallback state.
    crate::secure_input::register_cancel_fallback(app);

    CANCEL_REQUESTED.store(true, Ordering::SeqCst);
    schedule_cancel_reconcile(app);
}

/// Unregister the cancel shortcut (called when recording stops)
pub fn unregister_cancel_shortcut(app: &AppHandle) {
    crate::secure_input::unregister_cancel_fallback(app);

    CANCEL_REQUESTED.store(false, Ordering::SeqCst);
    schedule_cancel_reconcile(app);
}

/// Apply the requested cancel state off the calling thread. Start and stop run
/// on the shortcut handler thread, which must not block on registration, so the
/// work is spawned. Spawned tasks may run in any order (a very short recording
/// can see its unregister task run before its register task), so each pass
/// applies the latest request instead of the action that scheduled it.
fn schedule_cancel_reconcile(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        reconcile_cancel_shortcut(&app);
    });
}

fn reconcile_cancel_shortcut(app: &AppHandle) {
    // Cancel shortcut is disabled on Linux due to instability with dynamic shortcut registration
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        return;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let mut registered = CANCEL_REGISTERED.lock().unwrap_or_else(|e| e.into_inner());
        let requested = CANCEL_REQUESTED.load(Ordering::SeqCst);
        if requested == *registered {
            return;
        }

        let Some(cancel_binding) = get_settings(app).bindings.get("cancel").cloned() else {
            return;
        };

        if requested {
            match register_shortcut(app, cancel_binding) {
                Ok(()) => *registered = true,
                Err(e) => error!("Failed to register cancel shortcut: {}", e),
            }
        } else {
            match unregister_shortcut(app, cancel_binding) {
                Ok(()) => *registered = false,
                Err(e) => error!("Failed to unregister cancel shortcut: {}", e),
            }
        }
    }
}

// @spec AC-005: Cancel keeps its dynamic registration — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-005
/// Whether `id`'s shortcuts are registered with the backend outside
/// recording: cancel is registered dynamically while recording, and the
/// post-processing action and the prompt shortcuts (005) only while
/// post-processing is enabled.
pub(crate) fn is_statically_registered(id: &str, settings: &settings::AppSettings) -> bool {
    match id {
        "cancel" => false,
        _ if prompt_targets::is_post_process_gated(id) => settings.post_process_enabled,
        _ => true,
    }
}

/// Registrations of every shortcut registered at startup, using the
/// default binding for any action missing from the store.
pub(crate) fn startup_registrations(app: &AppHandle) -> Vec<ShortcutRegistration> {
    keyboard_switch::static_registrations(&settings::load_or_create_app_settings(app))
}

/// Register one shortcut of one action with the active implementation
///
/// # Errors
/// Returns the backend's refusal.
pub fn register_registration(
    app: &AppHandle,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    register_with(app, get_settings(app).keyboard_implementation, registration)
}

fn register_with(
    app: &AppHandle,
    implementation: KeyboardImplementation,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    match implementation {
        KeyboardImplementation::Tauri => tauri_impl::register_shortcut(app, registration),
        KeyboardImplementation::HandyKeys => handy_keys::register_shortcut(app, registration),
    }
}

/// Unregister one shortcut of one action from the active implementation
///
/// # Errors
/// Returns the backend's refusal.
pub fn unregister_registration(
    app: &AppHandle,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    unregister_with(app, get_settings(app).keyboard_implementation, registration)
}

fn unregister_with(
    app: &AppHandle,
    implementation: KeyboardImplementation,
    registration: &ShortcutRegistration,
) -> Result<(), String> {
    match implementation {
        KeyboardImplementation::Tauri => tauri_impl::unregister_shortcut(app, registration),
        KeyboardImplementation::HandyKeys => handy_keys::unregister_shortcut(app, registration),
    }
}

/// Register every shortcut of `binding` using the appropriate implementation.
/// Each shortcut is attempted independently, so an already-registered one
/// does not keep the others from registering.
///
/// # Errors
/// Returns the first refusal after attempting every shortcut.
pub fn register_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    first_error(
        ShortcutRegistration::all_of(&binding)
            .iter()
            .map(|registration| register_registration(app, registration)),
    )
}

/// Unregister every shortcut of `binding` using the appropriate implementation
///
/// # Errors
/// Returns the first refusal after attempting every shortcut.
pub fn unregister_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    first_error(
        ShortcutRegistration::all_of(&binding)
            .iter()
            .map(|registration| unregister_registration(app, registration)),
    )
}

/// Consumes every result (so every operation runs) and keeps the first error.
fn first_error(results: impl Iterator<Item = Result<(), String>>) -> Result<(), String> {
    let mut first = Ok(());
    for result in results {
        if first.is_ok() {
            first = result;
        }
    }
    first
}

// ============================================================================
// Binding Management Commands
// ============================================================================

#[derive(Serialize, Type)]
pub struct BindingResponse {
    success: bool,
    binding: Option<ShortcutBinding>,
    /// English description of the refusal, for older callers.
    error: Option<String>,
    /// Typed refusal, which the settings UI translates.
    edit_error: Option<action_shortcuts::ShortcutEditError>,
}
#[tauri::command]
#[specta::specta]
/// Replaces the first shortcut of action `id`, as the settings recorder did before multiple
/// shortcuts, as one ordered settings write (see [`edits::apply_shortcut_edit`]). Alternatives
/// are kept; the new shortcut goes through the shared validation and conflict check. Runs on the
/// blocking pool, never on the main thread.
///
/// # Errors
/// An empty shortcut, or a refusal of the settings service (busy, persistence); every refusal of
/// the edit itself, invalid syntax included, is reported in the response.
pub async fn change_binding(
    app: AppHandle,
    id: String,
    binding: String,
) -> Result<BindingResponse, SettingsCommandError> {
    if binding.trim().is_empty() {
        return Err(SettingsCommandError::invalid(
            "bindings",
            ValidationReason::CanonicalValue,
        ));
    }
    live_settings::blocking(move || {
        let previous = edits::current_binding_or_default(&settings::get_settings(&app), &id)
            .map(|current| current.current_binding)
            .unwrap_or_default();
        let edit = action_shortcuts::ShortcutEdit::Replace {
            previous,
            shortcut: binding,
        };
        let result = edits::apply_shortcut_edit(&app, &id, &edit)?;
        Ok(binding_response(&app, result))
    })
    .await
}

/// Maps an edit result to the legacy response: every refusal, invalid
/// syntax included, is in the response with its typed reason so the
/// settings translate it; `error` keeps the English text for older callers.
fn binding_response(
    app: &AppHandle,
    result: Result<ShortcutBinding, action_shortcuts::ShortcutEditError>,
) -> BindingResponse {
    match result {
        Ok(binding) => BindingResponse {
            success: true,
            binding: Some(binding),
            error: None,
            edit_error: None,
        },
        Err(error) => {
            let message = edits::describe_edit_error(app, &error);
            warn!("change_binding error: {}", message);
            BindingResponse {
                success: false,
                binding: None,
                error: Some(message),
                edit_error: Some(error),
            }
        }
    }
}

#[tauri::command]
#[specta::specta]
/// Restores the single default shortcut of action `id`; alternatives are removed.
///
/// # Errors
/// An unknown action or a refusal of the settings service; refusals of the edit are in the
/// response.
pub async fn reset_binding(
    app: AppHandle,
    id: String,
) -> Result<BindingResponse, SettingsCommandError> {
    live_settings::blocking(move || {
        settings::get_stored_binding(&settings::get_settings(&app), &id).map_err(|_| {
            SettingsCommandError::invalid("bindings", ValidationReason::CanonicalValue)
        })?;
        let result = edits::apply_shortcut_edit(&app, &id, &action_shortcuts::ShortcutEdit::Reset)?;
        Ok(binding_response(&app, result))
    })
    .await
}

/// Unregister every binding while the user is recording a new shortcut in
/// the UI, so no existing shortcut can fire — or swallow the keystrokes —
/// mid-capture. The "cancel" binding is untouched: it is managed dynamically
/// by the recording lifecycle.
pub fn suspend_all_shortcuts(app: &AppHandle) {
    for (id, binding) in settings::get_bindings(app) {
        if id == "cancel" {
            continue;
        }
        if let Err(e) = unregister_shortcut(app, binding) {
            debug!(
                "suspend_all_shortcuts: could not unregister '{}': {}",
                id, e
            );
        }
    }
}

/// Re-register every binding from settings after shortcut recording ends.
/// Registering an already-registered shortcut fails cleanly in both
/// implementations, so this is idempotent and safe on every exit path.
pub fn resume_all_shortcuts(app: &AppHandle) {
    let settings = get_settings(app);
    for (id, binding) in &settings.bindings {
        if !is_statically_registered(id, &settings) {
            continue;
        }
        if let Err(e) = register_shortcut(app, binding.clone()) {
            debug!("resume_all_shortcuts: could not register '{}': {}", id, e);
        }
    }
}

/// Temporarily unregister all bindings while the user is recording a
/// shortcut in the UI. This avoids firing actions while keys are recorded.
#[tauri::command]
#[specta::specta]
pub fn suspend_all_bindings(app: AppHandle) -> Result<(), String> {
    suspend_all_shortcuts(&app);
    Ok(())
}

/// Re-register all bindings after the user has finished recording.
#[tauri::command]
#[specta::specta]
pub fn resume_all_bindings(app: AppHandle) -> Result<(), String> {
    resume_all_shortcuts(&app);
    Ok(())
}

// ============================================================================
// Keyboard Implementation Switching
// ============================================================================

/// Result of changing keyboard implementation
#[derive(Serialize, Type)]
pub struct ImplementationChangeResult {
    pub success: bool,
    /// Binding IDs whose shortcuts were reset to the default or removed
    /// because the new implementation cannot register them
    pub reset_bindings: Vec<String>,
    /// Stored shortcuts that conflict under the new implementation and did
    /// not before. They are kept, as inherited conflicts are, and flagged in
    /// the settings.
    pub conflicts: Vec<conflicts::StoredShortcutConflict>,
}

#[tauri::command]
#[specta::specta]
/// Change the keyboard implementation with runtime switching, as one ordered write: the shared
/// validator checks the value, every registration moves to the new implementation with the
/// lists repaired for it, then the implementation and the repaired lists are committed in one
/// revision (see [`live_change::keyboard_implementation`]); a refused commit moves them back.
///
/// # Errors
/// A refused value, `unavailable` during a dictation session or when HandyKeys cannot start
/// (nothing changed), or a refusal of the commit.
pub async fn change_keyboard_implementation_setting(
    app: AppHandle,
    implementation: UiValue<String>,
) -> Result<ImplementationChangeResult, SettingsCommandError> {
    let implementation = implementation.into_json();
    live_settings::blocking(move || {
        live_settings::ordered(&app, || {
            let previous = settings::get_settings(&app);
            live_settings::set_prepared(&app, "keyboard_implementation", implementation)?;
            Ok(switch_result(&app, &previous))
        })
    })
    .await
}

/// Live effect of a committed `keyboard_implementation` change that was not prepared (the
/// prepared write of every surface skips it): the registrations move to the new implementation
/// and the lists it cannot register are repaired by an internal commit.
///
/// # Errors
/// The HandyKeys initialization failure (the implementation is then reverted to Tauri by a
/// second commit and the shortcuts are registered with it).
pub(crate) fn switch_keyboard_implementation(
    app: &AppHandle,
    before: &settings::AppSettings,
    after: &settings::AppSettings,
) -> Result<(), String> {
    if before.keyboard_implementation == after.keyboard_implementation {
        return Ok(());
    }
    let mut repaired = after.clone();
    let ids = keyboard_switch::repair_lists(&mut repaired);
    if let Err(error) = keyboard_switch::switch_registrations(app, before, &repaired) {
        persist_tauri_fallback(app);
        crate::secure_input::reconcile_fallback(app);
        return Err(format!("{error}. Reverted to Tauri."));
    }
    persist_binding_repairs(app, &repaired, &ids);
    crate::secure_input::reconcile_fallback(app);
    Ok(())
}

/// Ids of the stored bindings whose shortcut list differs from `previous`, sorted.
fn changed_lists(previous: &settings::AppSettings, current: &settings::AppSettings) -> Vec<String> {
    let mut ids: Vec<String> = current
        .bindings
        .iter()
        .filter(|(id, binding)| {
            previous
                .bindings
                .get(*id)
                .is_some_and(|old| old.shortcuts() != binding.shortcuts())
        })
        .map(|(id, _)| id.clone())
        .collect();
    ids.sort();
    ids
}

/// Commit the repaired bindings `ids` of `repaired` (only those entries), if any.
fn persist_binding_repairs(app: &AppHandle, repaired: &settings::AppSettings, ids: &[String]) {
    if ids.is_empty() {
        return;
    }
    live_settings::repair(
        app,
        "shortcuts repaired for the keyboard implementation",
        &|settings| {
            for id in ids {
                if let Some(binding) = repaired.bindings.get(id) {
                    settings.bindings.insert(id.clone(), binding.clone());
                }
            }
            Ok(())
        },
    );
}

/// The switch outcome: the bindings the switch repaired (reset to the default or with removed
/// alternatives) and the conflicts it introduced, compared with the `previous` settings.
fn switch_result(app: &AppHandle, previous: &settings::AppSettings) -> ImplementationChangeResult {
    let current = settings::get_settings(app);
    let reset_bindings = changed_lists(previous, &current);
    let conflicts = implementation_switch::introduced_conflicts(
        &previous.bindings,
        previous.keyboard_implementation,
        &current.bindings,
        current.keyboard_implementation,
    );
    if !conflicts.is_empty() {
        warn!(
            "{} stored shortcut(s) conflict under {:?}; kept and flagged",
            conflicts.len(),
            current.keyboard_implementation
        );
    }
    ImplementationChangeResult {
        success: true,
        reset_bindings,
        conflicts,
    }
}

/// Persist the Tauri implementation after HandyKeys failed, so it is not retried at next launch,
/// with the lists repaired for Tauri in the same commit, as a switch does. The caller registers
/// the Tauri shortcuts itself, so the switch effect must not run again.
fn persist_tauri_fallback(app: &AppHandle) {
    live_settings::repair(app, "the Tauri shortcut fallback", &|settings| {
        settings.keyboard_implementation = KeyboardImplementation::Tauri;
        keyboard_switch::repair_lists(settings);
        Ok(())
    });
}

/// Get the current keyboard implementation
#[tauri::command]
#[specta::specta]
pub fn get_keyboard_implementation(app: AppHandle) -> String {
    let settings = settings::get_settings(&app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => "tauri".to_string(),
        KeyboardImplementation::HandyKeys => "handy_keys".to_string(),
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate a shortcut for a specific implementation
pub(crate) fn validate_shortcut_for_implementation(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    match implementation {
        KeyboardImplementation::Tauri => tauri_impl::validate_shortcut(raw),
        KeyboardImplementation::HandyKeys => handy_keys::validate_shortcut(raw),
    }
}

// ============================================================================
// General Settings Commands
// ============================================================================
//
// Every write goes through the live settings service: the value is checked by the validator
// shared with the CLI, committed on the latest stored settings (one revision), then its live
// effect runs and the UI is notified. A refused value changes nothing and is returned as a
// typed error; no command replaces an invalid value by a default any more.

type SettingsResult = Result<(), SettingsCommandError>;

#[tauri::command]
#[specta::specta]
pub async fn change_shortcut_activation_setting(
    app: AppHandle,
    activation: UiValue<ShortcutActivation>,
) -> SettingsResult {
    live_settings::write(app, "shortcut_activation", activation).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_hold_threshold_ms_setting(app: AppHandle, ms: UiValue<u64>) -> SettingsResult {
    live_settings::write(app, "hold_threshold_ms", ms).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_audio_feedback_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "audio_feedback", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_audio_feedback_volume_setting(
    app: AppHandle,
    volume: UiValue<f32>,
) -> SettingsResult {
    live_settings::write(app, "audio_feedback_volume", volume).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_sound_theme_setting(app: AppHandle, theme: UiValue<String>) -> SettingsResult {
    live_settings::write(app, "sound_theme", theme).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_theme_setting(app: AppHandle, theme: UiValue<String>) -> SettingsResult {
    live_settings::write(app, "theme", theme).await
}

/// Applies the appearance setting to the native window chrome (title bar), which
/// CSS `data-theme` cannot reach. `System` clears the override so the window
/// follows the OS. Call this on startup and whenever the setting changes to keep
/// the title bar in sync with the in-app palette.
///
/// On Windows this themes the title bar only. On macOS `set_theme` sets
/// `NSApp.appearance` app-wide, which is what we want here: it darkens the title
/// bar and keeps the overlay in step. Linux is left to `data-theme` alone, since
/// its window theming is backend-dependent and unreliable.
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn apply_window_theme(app: &AppHandle, theme: Theme) {
    let window_theme = match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    };
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.set_theme(window_theme) {
            warn!("Failed to apply window theme: {}", e);
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn change_translate_to_english_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "translate_to_english", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_selected_language_setting(
    app: AppHandle,
    language: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "selected_language", language).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_overlay_position_setting(
    app: AppHandle,
    position: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "overlay_position", position).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_overlay_style_setting(
    app: AppHandle,
    style: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "overlay_style", style).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_debug_mode_setting(app: AppHandle, enabled: UiValue<bool>) -> SettingsResult {
    live_settings::write(app, "debug_mode", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_start_hidden_setting(app: AppHandle, enabled: UiValue<bool>) -> SettingsResult {
    live_settings::write(app, "start_hidden", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_autostart_setting(app: AppHandle, enabled: UiValue<bool>) -> SettingsResult {
    live_settings::write(app, "autostart_enabled", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_update_checks_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "update_checks_enabled", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_show_whats_new_on_update_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "show_whats_new_on_update", enabled).await
}

/// UI-owned marker (read-only for the CLI); the trimmed version is stored as given.
#[tauri::command]
#[specta::specta]
pub async fn change_whats_new_last_seen_version_setting(
    app: AppHandle,
    version: String,
) -> SettingsResult {
    live_settings::blocking(move || {
        let version = version.trim().to_string();
        live_settings::edit(&app, &|settings| {
            settings.whats_new_last_seen_version = version.clone();
            Ok(())
        })
        .map(|_| ())
    })
    .await
}

/// Whole vocabulary list from the settings UI, committed as the vocabulary domain's change: same
/// validator as the CLI (unchanged legacy rows stay exempt), compare-and-set on
/// `expected_revision` when the UI passes its base revision.
// @spec FR-001: UI list commit through the shared change — .specs/features/003-recognition-vocabulary/spec.md#fr-001
#[tauri::command]
#[specta::specta]
pub async fn update_custom_words(
    app: AppHandle,
    words: Vec<String>,
    expected_revision: Option<u64>,
) -> SettingsResult {
    live_settings::blocking(move || {
        let change =
            crate::recognition_vocabulary::edits::VocabularyEdit::Replace(words).into_change();
        let expected = expected_revision.map(crate::settings_service::protocol::Revision);
        live_settings::write_change(&app, &change, expected).map(|_| ())
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn change_word_correction_threshold_setting(
    app: AppHandle,
    threshold: UiValue<f64>,
) -> SettingsResult {
    live_settings::write(app, "word_correction_threshold", threshold).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_extra_recording_buffer_setting(
    app: AppHandle,
    ms: UiValue<u64>,
) -> SettingsResult {
    live_settings::write(app, "extra_recording_buffer_ms", ms).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_paste_delay_ms_setting(app: AppHandle, ms: UiValue<u64>) -> SettingsResult {
    live_settings::write(app, "paste_delay_ms", ms).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_paste_delay_after_ms_setting(
    app: AppHandle,
    ms: UiValue<u64>,
) -> SettingsResult {
    live_settings::write(app, "paste_delay_after_ms", ms).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_reliable_paste_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "reliable_paste", enabled).await
}

/// 007 AC-019 (experimental): paste verification after the smart-paste Cmd+V.
#[tauri::command]
#[specta::specta]
pub async fn change_paste_verification_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "paste_verification", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_paste_method_setting(
    app: AppHandle,
    method: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "paste_method", method).await
}

/// Whether the paste method selector offers smart paste: macOS only, and only once dictation
/// delivers through it (feature 007 rollout gate, `smart_paste::setting_policy`).
#[tauri::command]
#[specta::specta]
pub fn is_smart_paste_available() -> bool {
    crate::smart_paste::setting_policy::Rollout::CURRENT.is_active()
}

#[tauri::command]
#[specta::specta]
pub fn get_available_typing_tools() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        crate::clipboard::get_available_typing_tools()
    }
    #[cfg(not(target_os = "linux"))]
    {
        vec!["auto".to_string()]
    }
}

#[tauri::command]
#[specta::specta]
pub async fn change_typing_tool_setting(app: AppHandle, tool: UiValue<String>) -> SettingsResult {
    live_settings::write(app, "typing_tool", tool).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_external_script_path_setting(
    app: AppHandle,
    path: UiValue<Option<String>>,
) -> SettingsResult {
    live_settings::write(app, "external_script_path", path).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_clipboard_handling_setting(
    app: AppHandle,
    handling: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "clipboard_handling", handling).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_auto_submit_setting(app: AppHandle, enabled: UiValue<bool>) -> SettingsResult {
    live_settings::write(app, "auto_submit", enabled).await
}

// @spec AC-004: One Automatic Enter setting in the UI — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
#[tauri::command]
#[specta::specta]
/// Automatic Enter (009): `never`, `always` or `pin_only`, with the validation shared with the CLI.
/// Registered in both gate states; a closed build has no descriptor for it and refuses it with
/// `not_found`, as an open build refuses `change_auto_submit_setting`.
pub async fn change_automatic_enter_setting(
    app: AppHandle,
    policy: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "automatic_enter", policy).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_auto_submit_key_setting(
    app: AppHandle,
    key: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "auto_submit_key", key).await
}

// @spec AC-002: Post-processing activation behaves as before — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-002
#[tauri::command]
#[specta::specta]
/// Enables or disables post-processing and always saves the choice, as
/// before this feature, as one ordered settings write (see
/// [`edits::set_post_process_enabled`]). A shortcut the system refuses stays
/// stored and editable, and is returned so the settings say so.
///
/// # Errors
/// A refusal of the settings service (persistence); the toggle itself is never refused.
pub async fn change_post_process_enabled_setting(
    app: AppHandle,
    enabled: bool,
) -> Result<Vec<edits::RefusedShortcut>, SettingsCommandError> {
    live_settings::blocking(move || edits::set_post_process_enabled(&app, enabled)).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_experimental_enabled_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "experimental_enabled", enabled).await
}

/// Register or release the post-processing shortcuts after `post_process_enabled` changed from
/// `before` (live effect shared by every surface that commits the scalar alone). As in the
/// settings, a shortcut the system refuses is logged and stays stored; the toggle stands.
pub(crate) fn apply_post_process_shortcut(
    app: &AppHandle,
    before: &settings::AppSettings,
    enabled: bool,
) -> Result<(), String> {
    let (_, refused) = edits::toggle_post_process_on_backend(app, before, enabled);
    edits::log_refusals(&refused);
    crate::secure_input::reconcile_fallback(app);
    Ok(())
}

/// Provider configuration refusals name the provider list field with a fixed reason.
fn unknown_provider() -> SettingsError {
    SettingsError::validation("post_process_providers", ValidationReason::CanonicalValue)
}

/// Only the custom provider's base URL is editable.
#[tauri::command]
#[specta::specta]
pub async fn change_post_process_base_url_setting(
    app: AppHandle,
    provider_id: String,
    base_url: String,
) -> SettingsResult {
    live_settings::blocking(move || {
        live_settings::edit(&app, &|settings| {
            let provider = settings
                .post_process_provider_mut(&provider_id)
                .ok_or_else(unknown_provider)?;
            if provider.id != "custom" {
                return Err(unknown_provider());
            }
            provider.base_url = base_url.clone();
            Ok(())
        })
        .map(|_| ())
    })
    .await
}

/// Store a provider API key (secret, UI-only); the provider must exist.
#[tauri::command]
#[specta::specta]
pub async fn change_post_process_api_key_setting(
    app: AppHandle,
    provider_id: String,
    api_key: String,
) -> SettingsResult {
    live_settings::blocking(move || {
        live_settings::edit(&app, &|settings| {
            settings
                .post_process_provider(&provider_id)
                .ok_or_else(unknown_provider)?;
            settings
                .post_process_api_keys
                .insert(provider_id.clone(), api_key.clone());
            Ok(())
        })
        .map(|_| ())
    })
    .await
}

/// Store the model chosen for an existing provider.
#[tauri::command]
#[specta::specta]
pub async fn change_post_process_model_setting(
    app: AppHandle,
    provider_id: String,
    model: String,
) -> SettingsResult {
    live_settings::blocking(move || {
        live_settings::edit(&app, &|settings| {
            settings
                .post_process_provider(&provider_id)
                .ok_or_else(unknown_provider)?;
            settings
                .post_process_models
                .insert(provider_id.clone(), model.clone());
            Ok(())
        })
        .map(|_| ())
    })
    .await
}

/// Select an existing post-processing provider.
#[tauri::command]
#[specta::specta]
pub async fn set_post_process_provider(app: AppHandle, provider_id: String) -> SettingsResult {
    live_settings::blocking(move || {
        live_settings::edit(&app, &|settings| {
            settings
                .post_process_provider(&provider_id)
                .ok_or_else(unknown_provider)?;
            settings.post_process_provider_id = provider_id.clone();
            Ok(())
        })
        .map(|_| ())
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_post_process_models(
    app: AppHandle,
    provider_id: String,
) -> Result<Vec<String>, String> {
    let settings = settings::get_settings(&app);

    // Find the provider
    let provider = settings
        .post_process_providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

    if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            return Ok(vec![APPLE_INTELLIGENCE_DEFAULT_MODEL_ID.to_string()]);
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            return Err("Apple Intelligence is only available on Apple silicon Macs running macOS 15 or later.".to_string());
        }
    }

    // Get API key
    let api_key = settings
        .post_process_api_keys
        .get(&provider_id)
        .cloned()
        .unwrap_or_default();

    // Skip fetching if no API key for providers that typically need one
    if api_key.trim().is_empty() && provider.id != "custom" {
        return Err(format!(
            "API key is required for {}. Please add an API key to list available models.",
            provider.label
        ));
    }

    crate::llm_client::fetch_models(provider, api_key).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_mute_while_recording_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "mute_while_recording", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_append_trailing_space_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "append_trailing_space", enabled).await
}

#[tauri::command]
#[specta::specta]
// @spec FR-006: Settings UI write of the destination setting — .specs/features/006-destination-application/spec.md#fr-006
/// Settings UI write of `return_to_dictation_app`: the shared UI validator (a build without the
/// destination capability refuses `true` and `false` with `validation_failed` and the typed reason
/// `unsupported_platform`), one durable commit, then `settings-changed`. The value is read when a
/// dictation starts, so no live effect runs.
pub async fn change_return_to_dictation_app_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "return_to_dictation_app", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_lazy_stream_close_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "lazy_stream_close", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_vad_enabled_setting(app: AppHandle, enabled: UiValue<bool>) -> SettingsResult {
    live_settings::write(app, "vad_enabled", enabled).await
}

/// Swap the VAD detector (and reopen the microphone when needed) before committing, off the
/// webview thread; a refused commit restores the backend published when the write started. The
/// preparation is shared with routed CLI writes (`settings_service::native_changes`).
#[tauri::command]
#[specta::specta]
pub async fn change_vad_backend_setting(
    app: AppHandle,
    backend: UiValue<VadBackend>,
) -> SettingsResult {
    let backend = backend.into_json();
    live_settings::blocking(move || {
        live_settings::set_prepared(&app, "vad_backend", backend).map(|_| ())
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn change_filler_word_removal_enabled_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "filler_word_removal_enabled", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_chinese_script_setting(
    app: AppHandle,
    script: UiValue<ChineseScript>,
) -> SettingsResult {
    live_settings::write(app, "chinese_script", script).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_app_language_setting(
    app: AppHandle,
    language: UiValue<String>,
) -> SettingsResult {
    live_settings::write(app, "app_language", language).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_show_tray_icon_setting(
    app: AppHandle,
    enabled: UiValue<bool>,
) -> SettingsResult {
    live_settings::write(app, "show_tray_icon", enabled).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_transcribe_accelerator_setting(
    app: AppHandle,
    accelerator: UiValue<settings::TranscribeAcceleratorSetting>,
) -> SettingsResult {
    live_settings::write(app, "transcribe_accelerator", accelerator).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_ort_accelerator_setting(
    app: AppHandle,
    accelerator: UiValue<settings::OrtAcceleratorSetting>,
) -> SettingsResult {
    live_settings::write(app, "ort_accelerator", accelerator).await
}

#[tauri::command]
#[specta::specta]
pub async fn change_transcribe_gpu_device(
    app: AppHandle,
    device: UiValue<Option<String>>,
) -> SettingsResult {
    live_settings::write(app, "transcribe_gpu_device", device).await
}

/// Return which accelerators and GPU devices are available for this build.
///
/// First-call cost is dominated by enumerating GPU devices through the
/// transcribe.cpp Metal/Vulkan backend, which loads dynamic libraries and
/// probes hardware. Run it on the blocking pool so the webview thread
/// stays responsive — see also the startup pre-warm in `lib.rs`.
#[tauri::command]
#[specta::specta]
pub async fn get_available_accelerators(
    app: AppHandle,
) -> crate::managers::transcription::AvailableAccelerators {
    tauri::async_runtime::spawn_blocking(move || {
        let tm =
            app.state::<std::sync::Arc<crate::managers::transcription::TranscriptionManager>>();
        crate::managers::transcription::get_available_accelerators(&tm)
    })
    .await
    .expect("get_available_accelerators panicked")
}

#[cfg(test)]
mod tests {
    use handy_keys::Hotkey;
    use tauri_plugin_global_shortcut::Shortcut;

    #[test]
    fn compound_shortcut_keys_parse_on_both_backends() {
        for key in [
            "scrolllock",
            "capslock",
            "numlock",
            "pageup",
            "pagedown",
            "printscreen",
        ] {
            assert!(key.parse::<Shortcut>().is_ok(), "Tauri rejected {key}");
            assert!(key.parse::<Hotkey>().is_ok(), "HandyKeys rejected {key}");
        }
    }
}
