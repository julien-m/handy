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

mod handler;
pub mod handy_keys;
pub mod tauri_impl;

use log::{debug, error, info, warn};
use serde::Serialize;
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::settings::APPLE_INTELLIGENCE_DEFAULT_MODEL_ID;
use crate::settings::{
    self, get_settings, ChineseScript, KeyboardImplementation, LLMPrompt, ShortcutActivation,
    ShortcutBinding, Theme, VadBackend, APPLE_INTELLIGENCE_PROVIDER_ID,
};
use crate::settings_service::{
    command_error::SettingsCommandError,
    errors::{SettingsError, ValidationReason},
    tauri_adapter as live_settings,
};

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

/// Register a shortcut using the appropriate implementation
pub fn register_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::register_shortcut(app, binding),
        KeyboardImplementation::HandyKeys => handy_keys::register_shortcut(app, binding),
    }
}

/// Unregister a shortcut using the appropriate implementation
pub fn unregister_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let settings = get_settings(app);
    match settings.keyboard_implementation {
        KeyboardImplementation::Tauri => tauri_impl::unregister_shortcut(app, binding),
        KeyboardImplementation::HandyKeys => handy_keys::unregister_shortcut(app, binding),
    }
}

// ============================================================================
// Binding Management Commands
// ============================================================================

#[derive(Serialize, Type)]
pub struct BindingResponse {
    success: bool,
    binding: Option<ShortcutBinding>,
    error: Option<String>,
}
impl BindingResponse {
    /// Refusal shown to the user (invalid shortcut, registration failure); nothing is stored.
    fn failed(error: String) -> Self {
        Self {
            success: false,
            binding: None,
            error: Some(error),
        }
    }
    /// The committed binding, registered (or managed by the recording lifecycle for "cancel").
    fn succeeded(binding: ShortcutBinding) -> Self {
        Self {
            success: true,
            binding: Some(binding),
            error: None,
        }
    }
}

/// Change one shortcut. The native registration is prepared first (unregister the old one,
/// validate and register the new one); the binding is then committed on the latest settings,
/// and a refused commit restores the previous registration.
#[tauri::command]
#[specta::specta]
pub fn change_binding(
    app: AppHandle,
    id: String,
    binding: String,
) -> Result<BindingResponse, SettingsCommandError> {
    require_binding_text(&binding)?;
    let settings = settings::get_settings(&app);
    let Some(previous) = binding_or_default(&settings, &id) else {
        let error = format!("Binding with id '{}' not found in defaults", id);
        warn!("change_binding error: {}", error);
        return Ok(BindingResponse::failed(error));
    };
    let mut updated = previous.clone();
    updated.current_binding = binding;
    // The cancel binding is registered dynamically by the recording lifecycle, never here.
    let is_dynamic = id == "cancel";
    let prepared = if is_dynamic {
        Ok(())
    } else {
        prepare_binding(&app, &previous, &updated, settings.keyboard_implementation)
    };
    if let Err(error) = prepared {
        return Ok(BindingResponse::failed(error));
    }
    commit_binding(&app, &id, &previous, &updated, is_dynamic)?;
    Ok(BindingResponse::succeeded(updated))
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
fn commit_binding(
    app: &AppHandle,
    id: &str,
    previous: &ShortcutBinding,
    updated: &ShortcutBinding,
    is_dynamic: bool,
) -> Result<(), SettingsCommandError> {
    let committed = live_settings::edit(app, &|settings| {
        settings.bindings.insert(id.to_string(), updated.clone());
        Ok(())
    });
    if let Err(error) = committed {
        if !is_dynamic {
            compensate_binding(app, previous, updated);
        }
        return Err(error);
    }
    // Synchronize any active Secure Input shadows with the committed binding.
    crate::secure_input::reconcile_fallback(app);
    Ok(())
}

/// Stored binding, or its default when a newer build added it after the store was written.
fn binding_or_default(settings: &settings::AppSettings, id: &str) -> Option<ShortcutBinding> {
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
fn prepare_binding(
    app: &AppHandle,
    previous: &ShortcutBinding,
    updated: &ShortcutBinding,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    if let Err(e) = unregister_shortcut(app, previous.clone()) {
        error!("change_binding error: Failed to unregister shortcut: {}", e);
    }
    if let Err(e) = validate_shortcut_for_implementation(&updated.current_binding, implementation) {
        warn!("change_binding validation error: {}", e);
        restore_registration(app, previous);
        return Err(e);
    }
    if let Err(e) = register_shortcut(app, updated.clone()) {
        let error = format!("Failed to register shortcut: {}", e);
        error!("change_binding error: {}", error);
        restore_registration(app, previous);
        return Err(error);
    }
    Ok(())
}

/// Undo `prepare_binding` after a refused commit: the stored binding is still `previous`.
fn compensate_binding(app: &AppHandle, previous: &ShortcutBinding, updated: &ShortcutBinding) {
    if let Err(e) = unregister_shortcut(app, updated.clone()) {
        error!(
            "Failed to unregister refused binding '{}': {}",
            updated.id, e
        );
    }
    restore_registration(app, previous);
}

/// Best-effort re-register of the previous binding after a failed change,
/// so a failure leaves the user's shortcut working exactly as before.
fn restore_registration(app: &AppHandle, binding: &ShortcutBinding) {
    if let Err(e) = register_shortcut(app, binding.clone()) {
        error!(
            "Failed to restore previous binding '{}' ({}): {}",
            binding.id, binding.current_binding, e
        );
    }
}

#[tauri::command]
#[specta::specta]
pub fn reset_binding(app: AppHandle, id: String) -> Result<BindingResponse, SettingsCommandError> {
    let binding = settings::get_stored_binding(&settings::get_settings(&app), &id)
        .map_err(|_| SettingsCommandError::invalid("bindings", ValidationReason::CanonicalValue))?;
    change_binding(app, id, binding.default_binding)
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
        if id == "cancel" {
            continue;
        }
        if id == "transcribe_with_post_process" && !settings.post_process_enabled {
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
    /// List of binding IDs that were reset to defaults due to incompatibility
    pub reset_bindings: Vec<String>,
}

/// Change the keyboard implementation with runtime switching. The value is validated and
/// committed like any setting; its live effect (`switch_keyboard_implementation`) unregisters
/// the old shortcuts, registers them with the new implementation and resets incompatible ones.
#[tauri::command]
#[specta::specta]
pub fn change_keyboard_implementation_setting(
    app: AppHandle,
    implementation: String,
) -> Result<ImplementationChangeResult, SettingsCommandError> {
    let before = settings::get_settings(&app).bindings;
    live_settings::set_value(&app, "keyboard_implementation", implementation)?;
    let reset_bindings = settings::get_settings(&app)
        .bindings
        .into_iter()
        .filter(|(id, binding)| {
            before
                .get(id)
                .is_some_and(|old| old.current_binding != binding.current_binding)
        })
        .map(|(id, _)| id)
        .collect();
    Ok(ImplementationChangeResult {
        success: true,
        reset_bindings,
    })
}

/// Live effect of a committed `keyboard_implementation` change, shared by every surface.
///
/// # Errors
/// The HandyKeys initialization failure (the implementation is then reverted to Tauri by a
/// second commit and the shortcuts are registered with it).
pub(crate) fn switch_keyboard_implementation(
    app: &AppHandle,
    from: KeyboardImplementation,
    to: KeyboardImplementation,
) -> Result<(), String> {
    if from == to {
        return Ok(());
    }
    info!(
        "Switching keyboard implementation from {:?} to {:?}",
        from, to
    );
    unregister_all_shortcuts(app, from);
    // Carbon fallback registrations use the Tauri plugin. Remove them before
    // registering the full Tauri implementation to avoid duplicate conflicts.
    if to == KeyboardImplementation::Tauri {
        crate::secure_input::reconcile_fallback(app);
    }
    // Initialize new implementation if needed (HandyKeys needs state); it registers shortcuts.
    if to == KeyboardImplementation::HandyKeys && initialize_handy_keys_with_rollback(app)? {
        crate::secure_input::reconcile_fallback(app);
        return Ok(());
    }
    register_all_shortcuts_for_implementation(app, to);
    crate::secure_input::reconcile_fallback(app);
    info!("Keyboard implementation switched to {:?}", to);
    Ok(())
}

/// Persist the Tauri implementation after HandyKeys failed, so it is not retried at next launch.
/// The caller registers the Tauri shortcuts itself, so the switch effect must not run again.
fn persist_tauri_fallback(app: &AppHandle) {
    live_settings::repair_without_effects(app, "the Tauri shortcut fallback", &|settings| {
        settings.keyboard_implementation = KeyboardImplementation::Tauri;
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
fn validate_shortcut_for_implementation(
    raw: &str,
    implementation: KeyboardImplementation,
) -> Result<(), String> {
    match implementation {
        KeyboardImplementation::Tauri => tauri_impl::validate_shortcut(raw),
        KeyboardImplementation::HandyKeys => handy_keys::validate_shortcut(raw),
    }
}

/// Unregister all shortcuts for the current implementation
fn unregister_all_shortcuts(app: &AppHandle, implementation: KeyboardImplementation) {
    let bindings = settings::get_bindings(app);

    for (id, binding) in bindings {
        // Skip cancel shortcut as it's dynamically registered
        if id == "cancel" {
            continue;
        }

        let result = match implementation {
            KeyboardImplementation::Tauri => tauri_impl::unregister_shortcut(app, binding),
            KeyboardImplementation::HandyKeys => handy_keys::unregister_shortcut(app, binding),
        };

        if let Err(e) = result {
            warn!(
                "Failed to unregister shortcut '{}' during switch: {}",
                id, e
            );
        }
    }
}

/// Register all shortcuts for a specific implementation, validating and resetting invalid ones
/// to their defaults; the resets are committed as one edit of those bindings only.
fn register_all_shortcuts_for_implementation(
    app: &AppHandle,
    implementation: KeyboardImplementation,
) -> Vec<String> {
    let current_settings = settings::get_settings(app);
    let mut resets: Vec<(String, ShortcutBinding)> = Vec::new();
    for (id, default_binding) in &settings::get_default_settings().bindings {
        // Skip cancel shortcut as it's dynamically registered
        if id == "cancel" {
            continue;
        }
        // Skip post-processing shortcut when the feature is disabled
        if id == "transcribe_with_post_process" && !current_settings.post_process_enabled {
            continue;
        }
        let stored = current_settings.bindings.get(id).cloned();
        let (binding, is_reset) = binding_for_implementation(
            stored.unwrap_or_else(|| default_binding.clone()),
            default_binding,
            implementation,
        );
        if is_reset {
            resets.push((id.clone(), binding.clone()));
        }
        register_with(app, implementation, id, binding);
    }
    persist_binding_resets(app, &resets);
    resets.into_iter().map(|(id, _)| id).collect()
}

/// The binding to register with `implementation`, reset to its default when invalid there.
fn binding_for_implementation(
    mut binding: ShortcutBinding,
    default_binding: &ShortcutBinding,
    implementation: KeyboardImplementation,
) -> (ShortcutBinding, bool) {
    match validate_shortcut_for_implementation(&binding.current_binding, implementation) {
        Ok(()) => (binding, false),
        Err(e) => {
            info!(
                "Shortcut '{}' ({}) is invalid for {:?}: {}. Resetting to default.",
                binding.id, binding.current_binding, implementation, e
            );
            binding.current_binding = default_binding.current_binding.clone();
            (binding, true)
        }
    }
}

/// Register one binding with an explicit implementation; failures are logged.
fn register_with(
    app: &AppHandle,
    implementation: KeyboardImplementation,
    id: &str,
    binding: ShortcutBinding,
) {
    let result = match implementation {
        KeyboardImplementation::Tauri => tauri_impl::register_shortcut(app, binding),
        KeyboardImplementation::HandyKeys => handy_keys::register_shortcut(app, binding),
    };
    if let Err(e) = result {
        error!(
            "Failed to register shortcut '{}' for {:?}: {}",
            id, implementation, e
        );
    }
}

/// Commit the bindings reset to their defaults (only those entries), if any.
fn persist_binding_resets(app: &AppHandle, resets: &[(String, ShortcutBinding)]) {
    if resets.is_empty() {
        return;
    }
    live_settings::repair(
        app,
        "shortcuts reset for the keyboard implementation",
        &|settings| {
            for (id, binding) in resets {
                settings.bindings.insert(id.clone(), binding.clone());
            }
            Ok(())
        },
    );
}

/// Initialize HandyKeys if not already initialized, with rollback on failure
fn initialize_handy_keys_with_rollback(app: &AppHandle) -> Result<bool, String> {
    if app.try_state::<handy_keys::HandyKeysState>().is_some() {
        return Ok(false); // Already initialized, caller should continue
    }

    if let Err(e) = handy_keys::init_shortcuts(app) {
        error!("Failed to initialize HandyKeys: {}", e);
        // Rollback to Tauri
        persist_tauri_fallback(app);
        crate::secure_input::reconcile_fallback(app);
        tauri_impl::init_shortcuts(app);
        return Err(format!(
            "Failed to initialize HandyKeys: {}. Reverted to Tauri.",
            e
        ));
    }

    // init_shortcuts already registered shortcuts
    Ok(true)
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
pub fn change_shortcut_activation_setting(
    app: AppHandle,
    activation: ShortcutActivation,
) -> SettingsResult {
    live_settings::set_value(&app, "shortcut_activation", activation)
}

#[tauri::command]
#[specta::specta]
pub fn change_hold_threshold_ms_setting(app: AppHandle, ms: u64) -> SettingsResult {
    live_settings::set_value(&app, "hold_threshold_ms", ms)
}

#[tauri::command]
#[specta::specta]
pub fn change_audio_feedback_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "audio_feedback", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_audio_feedback_volume_setting(app: AppHandle, volume: f32) -> SettingsResult {
    live_settings::set_value(&app, "audio_feedback_volume", volume)
}

#[tauri::command]
#[specta::specta]
pub fn change_sound_theme_setting(app: AppHandle, theme: String) -> SettingsResult {
    live_settings::set_value(&app, "sound_theme", theme)
}

#[tauri::command]
#[specta::specta]
pub fn change_theme_setting(app: AppHandle, theme: String) -> SettingsResult {
    live_settings::set_value(&app, "theme", theme)
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
pub fn change_translate_to_english_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "translate_to_english", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_selected_language_setting(app: AppHandle, language: String) -> SettingsResult {
    live_settings::set_value(&app, "selected_language", language)
}

#[tauri::command]
#[specta::specta]
pub fn change_overlay_position_setting(app: AppHandle, position: String) -> SettingsResult {
    live_settings::set_value(&app, "overlay_position", position)
}

#[tauri::command]
#[specta::specta]
pub fn change_overlay_style_setting(app: AppHandle, style: String) -> SettingsResult {
    live_settings::set_value(&app, "overlay_style", style)
}

#[tauri::command]
#[specta::specta]
pub fn change_debug_mode_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "debug_mode", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_start_hidden_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "start_hidden", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_autostart_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "autostart_enabled", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_update_checks_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "update_checks_enabled", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_show_whats_new_on_update_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "show_whats_new_on_update", enabled)
}

/// UI-owned marker (read-only for the CLI); the trimmed version is stored as given.
#[tauri::command]
#[specta::specta]
pub fn change_whats_new_last_seen_version_setting(
    app: AppHandle,
    version: String,
) -> SettingsResult {
    let version = version.trim().to_string();
    live_settings::edit(&app, &|settings| {
        settings.whats_new_last_seen_version = version.clone();
        Ok(())
    })
    .map(|_| ())
}

/// Vocabulary list owned by the settings UI until its CLI domain ships.
#[tauri::command]
#[specta::specta]
pub fn update_custom_words(app: AppHandle, words: Vec<String>) -> SettingsResult {
    live_settings::edit(&app, &|settings| {
        settings.custom_words = words.clone();
        Ok(())
    })
    .map(|_| ())
}

#[tauri::command]
#[specta::specta]
pub fn change_word_correction_threshold_setting(app: AppHandle, threshold: f64) -> SettingsResult {
    live_settings::set_value(&app, "word_correction_threshold", threshold)
}

#[tauri::command]
#[specta::specta]
pub fn change_extra_recording_buffer_setting(app: AppHandle, ms: u64) -> SettingsResult {
    live_settings::set_value(&app, "extra_recording_buffer_ms", ms)
}

#[tauri::command]
#[specta::specta]
pub fn change_paste_delay_ms_setting(app: AppHandle, ms: u64) -> SettingsResult {
    live_settings::set_value(&app, "paste_delay_ms", ms)
}

#[tauri::command]
#[specta::specta]
pub fn change_paste_delay_after_ms_setting(app: AppHandle, ms: u64) -> SettingsResult {
    live_settings::set_value(&app, "paste_delay_after_ms", ms)
}

#[tauri::command]
#[specta::specta]
pub fn change_reliable_paste_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "reliable_paste", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_paste_method_setting(app: AppHandle, method: String) -> SettingsResult {
    live_settings::set_value(&app, "paste_method", method)
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
pub fn change_typing_tool_setting(app: AppHandle, tool: String) -> SettingsResult {
    live_settings::set_value(&app, "typing_tool", tool)
}

#[tauri::command]
#[specta::specta]
pub fn change_external_script_path_setting(app: AppHandle, path: Option<String>) -> SettingsResult {
    live_settings::set_value(&app, "external_script_path", path)
}

#[tauri::command]
#[specta::specta]
pub fn change_clipboard_handling_setting(app: AppHandle, handling: String) -> SettingsResult {
    live_settings::set_value(&app, "clipboard_handling", handling)
}

#[tauri::command]
#[specta::specta]
pub fn change_auto_submit_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "auto_submit", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_auto_submit_key_setting(app: AppHandle, key: String) -> SettingsResult {
    live_settings::set_value(&app, "auto_submit_key", key)
}

#[tauri::command]
#[specta::specta]
pub fn change_post_process_enabled_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "post_process_enabled", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_experimental_enabled_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "experimental_enabled", enabled)
}

/// Register or unregister the post-processing shortcut after `post_process_enabled` changed
/// (live effect shared by every surface).
pub(crate) fn apply_post_process_shortcut(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let binding = get_settings(app)
        .bindings
        .get("transcribe_with_post_process")
        .cloned();
    let applied = match binding {
        Some(binding) if enabled => register_shortcut(app, binding),
        Some(binding) => {
            // Unregistering a shortcut that is not registered leaves the desired state in place.
            if let Err(error) = unregister_shortcut(app, binding) {
                debug!("Post-process shortcut was not registered: {}", error);
            }
            Ok(())
        }
        None => Ok(()),
    };
    crate::secure_input::reconcile_fallback(app);
    applied
}

/// Provider configuration refusals name the provider list field with a fixed reason.
fn unknown_provider() -> SettingsError {
    SettingsError::validation("post_process_providers", ValidationReason::CanonicalValue)
}

/// Only the custom provider's base URL is editable.
#[tauri::command]
#[specta::specta]
pub fn change_post_process_base_url_setting(
    app: AppHandle,
    provider_id: String,
    base_url: String,
) -> SettingsResult {
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
}

/// Store a provider API key (secret, UI-only); the provider must exist.
#[tauri::command]
#[specta::specta]
pub fn change_post_process_api_key_setting(
    app: AppHandle,
    provider_id: String,
    api_key: String,
) -> SettingsResult {
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
}

/// Store the model chosen for an existing provider.
#[tauri::command]
#[specta::specta]
pub fn change_post_process_model_setting(
    app: AppHandle,
    provider_id: String,
    model: String,
) -> SettingsResult {
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
}

/// Select an existing post-processing provider.
#[tauri::command]
#[specta::specta]
pub fn set_post_process_provider(app: AppHandle, provider_id: String) -> SettingsResult {
    live_settings::edit(&app, &|settings| {
        settings
            .post_process_provider(&provider_id)
            .ok_or_else(unknown_provider)?;
        settings.post_process_provider_id = provider_id.clone();
        Ok(())
    })
    .map(|_| ())
}

/// Missing prompt ids name the prompt list with a fixed reason.
fn unknown_prompt() -> SettingsError {
    SettingsError::validation("post_process_prompts", ValidationReason::UnknownPrompt)
}

/// Append a prompt; its id is generated once, outside the (re-runnable) edit.
#[tauri::command]
#[specta::specta]
pub fn add_post_process_prompt(
    app: AppHandle,
    name: String,
    prompt: String,
) -> Result<LLMPrompt, SettingsCommandError> {
    // Unique id from the creation timestamp, as before the service existed.
    let new_prompt = LLMPrompt {
        id: format!("prompt_{}", chrono::Utc::now().timestamp_millis()),
        name,
        prompt,
    };
    live_settings::edit(&app, &|settings| {
        settings.post_process_prompts.push(new_prompt.clone());
        Ok(())
    })?;
    Ok(new_prompt)
}

/// Rename or rewrite an existing prompt.
#[tauri::command]
#[specta::specta]
pub fn update_post_process_prompt(
    app: AppHandle,
    id: String,
    name: String,
    prompt: String,
) -> SettingsResult {
    live_settings::edit(&app, &|settings| {
        let existing = settings
            .post_process_prompts
            .iter_mut()
            .find(|candidate| candidate.id == id)
            .ok_or_else(unknown_prompt)?;
        existing.name = name.clone();
        existing.prompt = prompt.clone();
        Ok(())
    })
    .map(|_| ())
}

/// Delete a prompt; the last prompt is kept, and a deleted selection moves to the first prompt.
#[tauri::command]
#[specta::specta]
pub fn delete_post_process_prompt(app: AppHandle, id: String) -> SettingsResult {
    live_settings::edit(&app, &|settings| {
        if settings.post_process_prompts.len() <= 1 {
            return Err(SettingsError::validation(
                "post_process_prompts",
                ValidationReason::CanonicalValue,
            ));
        }
        let original_len = settings.post_process_prompts.len();
        settings
            .post_process_prompts
            .retain(|prompt| prompt.id != id);
        if settings.post_process_prompts.len() == original_len {
            return Err(unknown_prompt());
        }
        if settings.post_process_selected_prompt_id.as_ref() == Some(&id) {
            settings.post_process_selected_prompt_id = settings
                .post_process_prompts
                .first()
                .map(|prompt| prompt.id.clone());
        }
        Ok(())
    })
    .map(|_| ())
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
pub fn set_post_process_selected_prompt(app: AppHandle, id: String) -> SettingsResult {
    live_settings::set_value(&app, "post_process_selected_prompt_id", id)
}

#[tauri::command]
#[specta::specta]
pub fn change_mute_while_recording_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "mute_while_recording", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_append_trailing_space_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "append_trailing_space", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_lazy_stream_close_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "lazy_stream_close", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_vad_enabled_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "vad_enabled", enabled)
}

/// Swap the VAD detector (and reopen the microphone when needed) before committing, off the
/// webview thread; a refused commit restores the previous backend.
#[tauri::command]
#[specta::specta]
pub async fn change_vad_backend_setting(app: AppHandle, backend: VadBackend) -> SettingsResult {
    let previous = settings::get_settings(&app).vad_backend;
    if previous == backend {
        return Ok(());
    }
    let manager = app
        .state::<std::sync::Arc<crate::managers::audio::AudioRecordingManager>>()
        .inner()
        .clone();
    tokio::task::spawn_blocking(move || {
        live_settings::set_prepared(
            &app,
            "vad_backend",
            backend,
            || {
                manager
                    .update_vad_backend(backend)
                    .map_err(|e| e.to_string())
            },
            || {
                if let Err(error) = manager.update_vad_backend(previous) {
                    error!("Failed to restore the previous VAD backend: {}", error);
                }
            },
        )
        .map(|_| ())
    })
    .await
    .map_err(|_| SettingsCommandError::unavailable("vad_backend"))?
}

#[tauri::command]
#[specta::specta]
pub fn change_filler_word_removal_enabled_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "filler_word_removal_enabled", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_chinese_script_setting(app: AppHandle, script: ChineseScript) -> SettingsResult {
    live_settings::set_value(&app, "chinese_script", script)
}

#[tauri::command]
#[specta::specta]
pub fn change_app_language_setting(app: AppHandle, language: String) -> SettingsResult {
    live_settings::set_value(&app, "app_language", language)
}

#[tauri::command]
#[specta::specta]
pub fn change_show_tray_icon_setting(app: AppHandle, enabled: bool) -> SettingsResult {
    live_settings::set_value(&app, "show_tray_icon", enabled)
}

#[tauri::command]
#[specta::specta]
pub fn change_transcribe_accelerator_setting(
    app: AppHandle,
    accelerator: settings::TranscribeAcceleratorSetting,
) -> SettingsResult {
    live_settings::set_value(&app, "transcribe_accelerator", accelerator)
}

#[tauri::command]
#[specta::specta]
pub fn change_ort_accelerator_setting(
    app: AppHandle,
    accelerator: settings::OrtAcceleratorSetting,
) -> SettingsResult {
    live_settings::set_value(&app, "ort_accelerator", accelerator)
}

#[tauri::command]
#[specta::specta]
pub fn change_transcribe_gpu_device(app: AppHandle, device: Option<String>) -> SettingsResult {
    live_settings::set_value(&app, "transcribe_gpu_device", device)
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
