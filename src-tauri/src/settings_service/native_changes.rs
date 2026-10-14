//! Native changes the live state can refuse (swapping the VAD detector or the input channel
//! during a recording): prepared before the commit and undone when the commit is refused. The
//! settings UI commands and CLI writes routed to the instance take them from here, so a refused
//! change is `unavailable` with nothing written on both surfaces.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    sequence::PreparedChange,
};
use crate::managers::audio::AudioRecordingManager;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Keys whose live change is prepared before their commit.
pub(crate) const PREPARED_KEYS: [&str; 2] = ["vad_backend", "selected_channel"];

/// The prepared native change of `key`, or `None` when its live effect simply follows the commit.
///
/// # Errors
/// `unavailable` (inside `Some`) when the audio manager is not running.
pub(crate) fn prepared_change(
    app: &AppHandle,
    key: &str,
) -> Option<Result<PreparedChange<'static>, SettingsError>> {
    if !PREPARED_KEYS.contains(&key) {
        return None;
    }
    let Some(manager) = app.try_state::<Arc<AudioRecordingManager>>() else {
        return Some(Err(SettingsError::new(SettingsErrorCode::Unavailable)));
    };
    let manager = Arc::clone(manager.inner());
    Some(Ok(match key {
        "vad_backend" => vad_backend(manager),
        _ => selected_channel(manager),
    }))
}
/// Swap the VAD detector (reopening the microphone when needed); restore it on refusal.
fn vad_backend(manager: Arc<AudioRecordingManager>) -> PreparedChange<'static> {
    let restore = Arc::clone(&manager);
    PreparedChange {
        prepare: Box::new(move |_, requested| {
            manager
                .update_vad_backend(requested.vad_backend)
                .map_err(|error| refused("vad_backend", &error))
        }),
        compensate: Box::new(move |before| {
            if let Err(error) = restore.update_vad_backend(before.vad_backend) {
                log::error!("Failed to restore the previous VAD backend: {error}");
            }
        }),
    }
}
/// Restart capture on the requested channel; restore the previous one on refusal.
fn selected_channel(manager: Arc<AudioRecordingManager>) -> PreparedChange<'static> {
    let restore = Arc::clone(&manager);
    PreparedChange {
        prepare: Box::new(move |_, requested| {
            manager
                .update_selected_channel(requested.selected_channel)
                .map_err(|error| refused("selected_channel", &error))
        }),
        compensate: Box::new(move |before| {
            if let Err(error) = restore.update_selected_channel(before.selected_channel) {
                log::error!("Failed to restore the previous channel: {error}");
            }
        }),
    }
}
/// The refusal of the live state, logged with its detail, reported without it.
fn refused(key: &'static str, detail: &anyhow::Error) -> SettingsError {
    log::warn!("Live change of setting '{key}' was refused: {detail}");
    SettingsError::live_state_refused(key)
}
