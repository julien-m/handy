//! Live effects of committed settings, applied after the durable write and before the UI event,
//! for every surface (settings UI, manager repairs and, from Step 5, routed CLI commands).
use super::live::LiveCommit;
use crate::managers::audio::AudioRecordingManager;
use crate::settings::AppSettings;
use std::sync::{atomic::Ordering, Arc};
use tauri::{AppHandle, Emitter, Manager};

/// Cached or registered runtime state refreshed after a key changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LiveEffect {
    Theme,
    OverlayPosition,
    OverlayStyle,
    Logging,
    Autostart,
    TrayMenu,
    TrayVisibility,
    TranscriptionReload,
    TranscribeGpuRetry,
    PostProcessShortcut,
    KeyboardImplementation,
    MicrophoneMode,
    MicrophoneDevice,
    MicrophoneChannel,
    VadBackend,
    HistoryCleanup,
}

/// Business rule per key; keys absent here are read at use time (paste, language, prompts,
/// feedback sounds, shortcut activation...) and need no live effect.
fn live_effect(key: &str) -> Option<LiveEffect> {
    Some(match key {
        "theme" => LiveEffect::Theme,
        "overlay_position" => LiveEffect::OverlayPosition,
        "overlay_style" => LiveEffect::OverlayStyle,
        "debug_mode" | "log_level" => LiveEffect::Logging,
        "autostart_enabled" => LiveEffect::Autostart,
        "app_language" => LiveEffect::TrayMenu,
        "show_tray_icon" => LiveEffect::TrayVisibility,
        "transcribe_accelerator" | "transcribe_gpu_device" => LiveEffect::TranscribeGpuRetry,
        "ort_accelerator" => LiveEffect::TranscriptionReload,
        "post_process_enabled" => LiveEffect::PostProcessShortcut,
        "keyboard_implementation" => LiveEffect::KeyboardImplementation,
        "always_on_microphone" => LiveEffect::MicrophoneMode,
        "selected_microphone" => LiveEffect::MicrophoneDevice,
        "selected_channel" => LiveEffect::MicrophoneChannel,
        "vad_backend" => LiveEffect::VadBackend,
        "history_limit" | "recording_retention_period" => LiveEffect::HistoryCleanup,
        _ => return None,
    })
}

/// Apply the live effect of every changed key except `skip` (effects a command prepared before
/// committing). Every key runs even after a failure; the first failing key is returned.
pub(crate) fn apply_changed(
    app: &AppHandle,
    commit: &LiveCommit,
    skip: &[&str],
) -> Result<(), &'static str> {
    let mut failed = None;
    for key in commit.changed.iter().copied() {
        // Early exit for this key only: its native change was prepared before the commit.
        if skip.contains(&key) {
            continue;
        }
        let Some(effect) = live_effect(key) else {
            continue;
        };
        if let Err(detail) = apply_effect(app, effect, commit) {
            log::error!("Live effect of setting '{key}' failed after commit: {detail}");
            failed.get_or_insert(key);
        }
    }
    failed.map_or(Ok(()), Err)
}
fn apply_effect(app: &AppHandle, effect: LiveEffect, commit: &LiveCommit) -> Result<(), String> {
    let after = &commit.after;
    match effect {
        LiveEffect::Theme => apply_theme(app, after.theme),
        LiveEffect::OverlayPosition => crate::utils::update_overlay_position(app),
        LiveEffect::OverlayStyle => apply_overlay_style(app, after),
        LiveEffect::Logging => apply_logging(after),
        LiveEffect::Autostart => crate::autostart::apply_autostart(app, after.autostart_enabled),
        LiveEffect::TrayMenu => crate::tray::update_tray_menu(app),
        LiveEffect::TrayVisibility => apply_tray_visibility(app, after.show_tray_icon),
        LiveEffect::TranscriptionReload => reload_transcription(app, false),
        LiveEffect::TranscribeGpuRetry => reload_transcription(app, true),
        LiveEffect::PostProcessShortcut => {
            return crate::shortcut::apply_post_process_shortcut(app, after.post_process_enabled)
        }
        LiveEffect::KeyboardImplementation => {
            return crate::shortcut::switch_keyboard_implementation(
                app,
                commit.before.keyboard_implementation,
                after.keyboard_implementation,
            )
        }
        LiveEffect::HistoryCleanup => return cleanup_history(app),
        audio => return apply_audio(app, audio, after),
    }
    Ok(())
}
fn apply_audio(app: &AppHandle, effect: LiveEffect, after: &AppSettings) -> Result<(), String> {
    use crate::managers::audio::MicrophoneMode;
    let Some(manager) = app.try_state::<Arc<AudioRecordingManager>>() else {
        // Before the audio manager exists the stored value is read when it starts.
        return Ok(());
    };
    let applied = match effect {
        LiveEffect::MicrophoneMode => manager.update_mode(if after.always_on_microphone {
            MicrophoneMode::AlwaysOn
        } else {
            MicrophoneMode::OnDemand
        }),
        LiveEffect::MicrophoneDevice => manager.update_selected_device(),
        LiveEffect::MicrophoneChannel => manager.update_selected_channel(after.selected_channel),
        LiveEffect::VadBackend => manager.update_vad_backend(after.vad_backend),
        _ => Ok(()),
    };
    applied.map_err(|error| error.to_string())
}
/// The audio path reads the enabled flag from a cache; the window may need re-centering.
fn apply_overlay_style(app: &AppHandle, after: &AppSettings) {
    crate::overlay::update_overlay_enabled_cache(
        after.overlay_style != crate::settings::OverlayStyle::None,
    );
    crate::utils::update_overlay_position(app);
}
fn apply_theme(app: &AppHandle, theme: crate::settings::Theme) {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    crate::shortcut::apply_window_theme(app, theme);
    // Other webviews (the recording overlay) re-apply the palette from this event; a webview
    // that is not open yet reads the theme when it starts.
    if let Err(error) = app.emit("theme-changed", theme) {
        log::warn!("Could not notify webviews of the theme change: {error}");
    }
}
/// Runtime logging state follows the stored settings; the `--debug` override applies only at
/// startup and is not reapplied here (same as before the settings service).
fn apply_logging(after: &AppSettings) {
    let level: tauri_plugin_log::LogLevel = after.log_level.into();
    let level: log::Level = level.into();
    crate::FILE_LOG_LEVEL.store(level.to_level_filter() as u8, Ordering::Relaxed);
    crate::WEBVIEW_LOG_STREAMING.store(after.debug_mode, Ordering::Relaxed);
}
fn apply_tray_visibility(app: &AppHandle, visible: bool) {
    // A headless run has no tray; the stored value applies at the next GUI start.
    if app.try_state::<tauri::tray::TrayIcon>().is_some() {
        crate::tray::set_tray_visibility(app, visible);
    }
}
/// The next model use reloads with the new accelerators; a running transcription keeps its
/// engine. A transcribe.cpp accelerator change also retries a GPU disabled by an earlier failure.
fn reload_transcription(app: &AppHandle, retry_gpu: bool) {
    use crate::managers::transcription::TranscriptionManager;
    if let Some(manager) = app.try_state::<Arc<TranscriptionManager>>() {
        manager.reload_model_on_next_use();
        if retry_gpu {
            manager.retry_transcribe_gpu("the transcribe.cpp accelerator settings changed");
        }
    }
}
fn cleanup_history(app: &AppHandle) -> Result<(), String> {
    use crate::managers::history::HistoryManager;
    match app.try_state::<Arc<HistoryManager>>() {
        Some(manager) => manager
            .cleanup_old_entries()
            .map_err(|error| error.to_string()),
        None => Ok(()),
    }
}
#[cfg(test)]
mod tests {
    use super::live_effect;
    use crate::settings_service::descriptor::{descriptors, RuntimeEffect};

    /// Keys whose descriptor declares a runtime effect but whose value is read at use time.
    const READ_AT_USE: &[&str] = &[
        "shortcut_activation",
        "hold_threshold_ms",
        "audio_feedback",
        "audio_feedback_volume",
        "sound_theme",
        "update_checks_enabled",
        "clamshell_microphone",
        "selected_output_device",
        "model_unload_timeout",
    ];
    #[test]
    fn every_declared_runtime_effect_is_applied_or_read_at_use() {
        for entry in descriptors() {
            let declared = entry.runtime_effect != RuntimeEffect::None;
            let applied = live_effect(entry.key).is_some();
            let read_at_use = READ_AT_USE.contains(&entry.key);
            assert!(
                applied || !declared || read_at_use,
                "{} declares {:?} without a live effect",
                entry.key,
                entry.runtime_effect
            );
            assert!(!(applied && read_at_use), "{} listed twice", entry.key);
        }
    }
    #[test]
    fn every_live_effect_key_is_a_descriptor_key() {
        let keys = descriptors()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>();
        for key in [
            "theme",
            "keyboard_implementation",
            "selected_channel",
            "recording_retention_period",
        ] {
            assert!(keys.contains(&key) && live_effect(key).is_some());
        }
        assert!(live_effect("paste_method").is_none());
        assert!(live_effect("not_a_setting").is_none());
    }
}
