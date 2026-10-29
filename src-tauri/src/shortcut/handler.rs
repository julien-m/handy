//! Shared shortcut event handling logic
//!
//! This module contains the common logic for handling shortcut events,
//! used by both the Tauri and handy-keys implementations.

use log::warn;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Manager};

use super::identity::ShortcutRegistration;
use crate::actions::ACTION_MAP;
use crate::managers::audio::AudioRecordingManager;
use crate::settings::get_settings;
use crate::transcription_coordinator::{is_transcribe_binding, KeyEdge};
use crate::TranscriptionCoordinator;

/// Handle a shortcut event from either implementation.
///
/// This function contains the shared logic for:
/// - Looking up the action in ACTION_MAP
/// - Handling the cancel binding (only fires when recording)
/// - Routing transcribe bindings to the coordinator, which applies the
///   configured activation mode (toggle / push-to-talk / hold-or-toggle)
///
/// # Arguments
/// * `app` - The Tauri app handle
/// * `registration` - The action and the shortcut of that action that fired
/// * `hotkey_string` - The string representation of the hotkey
/// * `is_pressed` - Whether this is a key press (true) or release (false)
/// * `received_at` - When the backend received the edge; the coordinator
///   measures hold durations between these instants (native session N13).
///   Other actions run at once and do not use it.
pub fn handle_shortcut_event(
    app: &AppHandle,
    registration: &ShortcutRegistration,
    hotkey_string: &str,
    is_pressed: bool,
    received_at: Instant,
) {
    let binding_id = registration.action_id.as_str();
    // 014: a key pressed alone during a dictation is no longer alone once a shortcut fired.
    crate::dictation_keys::watcher::note_shortcut_event();

    // Transcribe bindings are handled by the coordinator.
    if is_transcribe_binding(binding_id) {
        send_to_coordinator(app, registration, hotkey_string, is_pressed, received_at);
        return;
    }

    run_action(app, binding_id, hotkey_string, is_pressed);
}

/// Runs a non-transcribe action from ACTION_MAP for a shortcut edge.
fn run_action(app: &AppHandle, binding_id: &str, hotkey_string: &str, is_pressed: bool) {
    let Some(action) = ACTION_MAP.get(binding_id) else {
        warn!(
            "No action defined in ACTION_MAP for shortcut ID '{}'. Shortcut: '{}', Pressed: {}",
            binding_id, hotkey_string, is_pressed
        );
        return;
    };

    // Cancel binding: only fires when recording and key is pressed
    if binding_id == "cancel" {
        let audio_manager = app.state::<Arc<AudioRecordingManager>>();
        if audio_manager.is_recording() && is_pressed {
            action.start(app, binding_id, hotkey_string);
        }
        return;
    }

    // Remaining bindings (e.g. "test") use simple start/stop on press/release.
    if is_pressed {
        action.start(app, binding_id, hotkey_string);
    } else {
        action.stop(app, binding_id, hotkey_string);
    }
}

/// Forwards a transcribe shortcut edge, with its shortcut identity, to the
/// coordinator using the global activation mode and hold threshold.
fn send_to_coordinator(
    app: &AppHandle,
    registration: &ShortcutRegistration,
    hotkey_string: &str,
    is_pressed: bool,
    received_at: Instant,
) {
    let Some(coordinator) = app.try_state::<TranscriptionCoordinator>() else {
        warn!("TranscriptionCoordinator is not initialized");
        return;
    };
    let settings = get_settings(app);
    let edge = KeyEdge {
        binding_id: &registration.action_id,
        shortcut_id: &registration.shortcut_id,
        hotkey_string,
        is_pressed,
        received_at,
    };
    coordinator.send_input(
        edge,
        settings.shortcut_activation,
        std::time::Duration::from_millis(settings.hold_threshold_ms),
    );
}
