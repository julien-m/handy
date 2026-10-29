use crate::audio_feedback::{play_feedback_sound, play_feedback_sound_blocking, SoundType};
use crate::audio_toolkit::{is_microphone_access_denied, is_no_input_device_error, VadPolicy};
use crate::destination::delivery::{insert_final_text, DictationTicket, HistoryRecord};
use crate::dictation_output::{deliver, Delivery, OutputPorts, ProcessedTranscription};
use crate::managers::audio::AudioRecordingManager;
use crate::managers::history::HistoryManager;
use crate::managers::history_store::NewHistoryEntry;
use crate::managers::model::ModelManager;
use crate::managers::transcription::StreamWorkKind;
use crate::managers::transcription::TranscriptionManager;
use crate::prompts::dictation;
use crate::prompts::processor::complete_unless_cancelled;
use crate::settings::{get_settings, OverlayStyle};
use crate::shortcut;
use crate::tray::{set_tray_state, TrayIconState};
use crate::utils::{
    self, show_processing_overlay, show_recording_overlay, show_transcribing_overlay,
};
use crate::word_replacements::dictation::{finalize_or_transcribe, DictationRuleCapture};
use crate::word_replacements::published::current_matcher;
use crate::TranscriptionCoordinator;
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Manager;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct RecordingErrorEvent {
    error_type: String,
    detail: Option<String>,
}

/// Drop guard ending the dictation's prompt session when its processing task ends, however it
/// ends (005): later indicator choices and late results are refused.
struct SessionEnd(AppHandle, Option<u64>);
impl Drop for SessionEnd {
    fn drop(&mut self) {
        dictation::end_session(&self.0, self.1);
    }
}

/// Drop guard that finishes the transcription pipeline, including immediate
/// model unloading on early exits.
struct FinishGuard(AppHandle, Arc<TranscriptionManager>);
impl Drop for FinishGuard {
    fn drop(&mut self) {
        self.1.maybe_unload_immediately("transcription session");
        if let Some(c) = self.0.try_state::<TranscriptionCoordinator>() {
            c.notify_processing_finished();
        }
        // The pipeline just freed its large transient buffers (captured PCM,
        // WAV copy, engine scratch); hand the cached pages back to the OS so
        // they don't sit in malloc arenas until they get swapped out (#1792).
        crate::memory::trim_freed_memory();
    }
}

// Shortcut Action Trait
pub trait ShortcutAction: Send + Sync {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
}

// Transcribe Action. Whether its AI step runs, and with which prompt, is decided by the
// dictation's prompt session (005), from the binding that started it.
struct TranscribeAction {
    /// Replacement rules in force when each binding's recording started.
    rules_at_start: DictationRuleCapture,
}

fn should_use_streaming_overlay(style: OverlayStyle, is_streaming: bool) -> bool {
    style == OverlayStyle::Live && is_streaming
}

/// Real output ports of one dictation (see `dictation_output`): the AI step with the prompt
/// frozen at the stop (005) and its overlay, the history manager and the destination insertion
/// (006).
struct AppOutputPorts {
    app: AppHandle,
    recording: Arc<AudioRecordingManager>,
    transcription: Arc<TranscriptionManager>,
    history: Arc<HistoryManager>,
    cancel_generation: u64,
    /// Prompt session frozen at the stop: its prompt (`None`: no AI step) and its id.
    frozen: dictation::FrozenSession,
    use_streaming_overlay: bool,
    /// The verified WAV of this dictation, or `None` for a text-only history row.
    file_name: Option<String>,
    destination_ticket: Option<DictationTicket>,
}

impl OutputPorts for AppOutputPorts {
    type History = HistoryRecord;
    async fn post_process(&self, native: &str) -> Option<ProcessedTranscription> {
        if self.frozen.prompt.is_some() {
            if self.use_streaming_overlay {
                self.transcription
                    .emit_stream_working(StreamWorkKind::Polishing);
            } else {
                show_processing_overlay(&self.app);
            }
        }
        let processing = dictation::process_frozen(native, self.frozen.prompt.as_ref());
        complete_unless_cancelled(processing, || self.is_cancelled()).await
    }
    fn is_cancelled(&self) -> bool {
        self.recording.was_cancelled_since(self.cancel_generation)
    }
    fn save_history(&self, native: String, processed: &ProcessedTranscription) -> HistoryRecord {
        let entry = NewHistoryEntry {
            file_name: self.file_name.clone(),
            transcription_text: native,
            post_process_requested: self.frozen.prompt.is_some(),
            post_processed_text: processed.post_processed_text.clone(),
            post_process_prompt: processed.post_process_prompt.clone(),
            processing: processed.processing.clone(),
        };
        let cancelled = || self.is_cancelled();
        let history =
            crate::destination::history::save_dictation(self.history.as_ref(), entry, &cancelled);
        dictation::notify_failure(&self.app, self.frozen.session_id, processed);
        history
    }
    fn arrange_paragraphs(&self, text: String) -> String {
        // 016: read once per dictation here, so a change applies from the next dictation.
        let enabled = get_settings(&self.app).smart_paragraphs_enabled;
        crate::smart_paragraphs::for_dictation(enabled, text)
    }
    fn abandon(&self) {
        crate::destination::delivery::abandon(&self.app);
        utils::hide_recording_overlay(&self.app);
        set_tray_state(&self.app, TrayIconState::Idle);
    }
    async fn insert(&self, text: String, history: HistoryRecord) {
        let cancelled = Box::new({
            let (recording, generation) = (Arc::clone(&self.recording), self.cancel_generation);
            move || recording.was_cancelled_since(generation)
        });
        let insertion = (self.destination_ticket, text, history);
        insert_final_text(&self.app, insertion, cancelled).await;
    }
}

impl ShortcutAction for TranscribeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        let start_time = Instant::now();
        debug!("TranscribeAction::start called for binding: {}", binding_id);

        // Load model in the background
        let tm = app.state::<Arc<TranscriptionManager>>();
        let rm = app.state::<Arc<AudioRecordingManager>>();

        // Load ASR model and VAD model in parallel
        let kickoff_started = Instant::now();
        tm.initiate_model_load();
        let rm_clone = Arc::clone(&rm);
        std::thread::spawn(move || {
            if let Err(e) = rm_clone.preload_vad() {
                debug!("VAD pre-load failed: {}", e);
            }
        });
        let kickoff_elapsed = kickoff_started.elapsed();

        // Don't open the mic if nothing can transcribe the recording; the load
        // kicked off above fails and reports why.
        if !tm.is_model_loaded() {
            let selected_model = get_settings(app).selected_model;
            if let Err(e) = app
                .state::<Arc<ModelManager>>()
                .get_model_path(&selected_model)
            {
                warn!("Not starting recording: no model can transcribe it ({})", e);
                return;
            }
        }

        let binding_id = binding_id.to_string();
        let tray_started = Instant::now();
        set_tray_state(app, TrayIconState::Recording);
        let tray_elapsed = tray_started.elapsed();

        // Get the microphone mode to determine audio feedback timing
        let plan_started = Instant::now();
        let settings = get_settings(app);
        // Read the destination before any Handy surface is shown; committed only on success.
        let destination =
            crate::destination::app::capture_at_start(app, settings.return_to_dictation_app);
        let is_always_on = settings.always_on_microphone;

        let selected_model_info = app
            .state::<Arc<ModelManager>>()
            .get_model_info(&settings.selected_model);

        // Use the app-facing model capability as the single pre-recording source
        // for live streaming decisions. Unknown support is represented as false
        // until the model registry is updated by discovery or runtime load.
        let model_supports_streaming = selected_model_info
            .as_ref()
            .map(|m| m.supports_streaming)
            .unwrap_or(false);
        let vad_policy = if !settings.vad_enabled {
            VadPolicy::Disabled
        } else if model_supports_streaming {
            VadPolicy::Streaming
        } else {
            VadPolicy::Offline
        };
        if model_supports_streaming {
            tm.start_stream();
        }
        let plan_elapsed = plan_started.elapsed();

        // Sizing the overlay follows the same advertised capability. A model that
        // doesn't stream (or whose capability is not known yet) gets the compact
        // pill instead of an oversized transparent live window.
        let overlay_started = Instant::now();
        // 010: freeze the no-text-field setting for this dictation (the bubble stays open
        // until the recording really starts, so a refused start loses nothing).
        crate::recovery::app::dictation_starting(app, &settings);
        // 006 AC-009: this dictation's first placement follows the cursor again.
        crate::overlay::begin_dictation_placement();
        match settings.overlay_style {
            OverlayStyle::Live if model_supports_streaming => utils::show_streaming_overlay(app),
            OverlayStyle::Live | OverlayStyle::Minimal => show_recording_overlay(app),
            OverlayStyle::None => {} // show_overlay_state no-ops on None anyway
        }
        // Everything above runs before capture can begin, so each span here is
        // added keypress->capture latency.
        debug!(
            "start-path pre-recording steps: model_kickoff={:?} tray={:?} settings+stream_plan={:?} overlay={:?}",
            kickoff_elapsed,
            tray_elapsed,
            plan_elapsed,
            overlay_started.elapsed()
        );
        debug!("Microphone mode - always_on: {}", is_always_on);

        let mut recording_error: Option<String> = None;
        let recording_start_time = Instant::now();
        match rm.try_start_recording(&binding_id, vad_policy) {
            Ok(readiness) => {
                debug!(
                    "Recording request accepted in {:?}; waiting for first microphone samples",
                    recording_start_time.elapsed()
                );
                // AC-007: this dictation uses the rules committed before it started.
                self.rules_at_start
                    .capture(&binding_id, current_matcher(app));
                if let Some(destination) = destination {
                    let started = destination.commit(app);
                    // @spec FR-005: Keys read from start to insertion — .specs/features/014-in-dictation-keys/spec.md#fr-005
                    crate::dictation_keys::watcher::start(
                        app,
                        started.dictation_id,
                        started.destination_generation,
                    );
                }
                // 010: the recording started, so the recovery bubble closes now.
                crate::recovery::app::dictation_started(app);
                dictation::start_session(app, &settings, &binding_id);
                let generation = readiness.generation();
                let app_clone = app.clone();
                let rm_clone = Arc::clone(&rm);
                std::thread::spawn(move || {
                    if !readiness.wait() {
                        debug!("Microphone readiness wait ended without receiving samples");
                        return;
                    }

                    // Development-only preview hook for evaluating the brief
                    // arming animation on hardware that normally starts too fast
                    // to make it visible.
                    #[cfg(debug_assertions)]
                    if let Ok(delay_ms) = std::env::var("HANDY_DEBUG_MIC_READY_DELAY_MS")
                        .unwrap_or_default()
                        .parse::<u64>()
                    {
                        let delay_ms = delay_ms.min(10_000);
                        if delay_ms > 0 {
                            debug!("Delaying microphone-ready cue by {delay_ms}ms for UI preview");
                            std::thread::sleep(Duration::from_millis(delay_ms));
                        }
                    }

                    if !rm_clone.is_recording_readiness_current(generation) {
                        debug!("Microphone became ready for an inactive recording");
                        return;
                    }

                    debug!("Microphone is receiving samples; recording is ready");
                    utils::emit_recording_ready(&app_clone);

                    // The start chime is a readiness cue, so it must follow the
                    // first real input callback rather than Stream::play() or a
                    // fixed delay. The helper returns immediately when feedback
                    // is disabled; mute still follows the same readiness point.
                    if rm_clone.is_recording_readiness_current(generation) {
                        play_feedback_sound_blocking(&app_clone, SoundType::Start);
                    }
                    if rm_clone.is_recording_readiness_current(generation) {
                        rm_clone.apply_mute();
                    }
                });
            }
            Err(e) => {
                debug!("Failed to start recording: {}", e);
                recording_error = Some(e);
            }
        }

        if recording_error.is_none() {
            // Dynamically register the cancel shortcut in a separate task to avoid deadlock
            shortcut::register_cancel_shortcut(app);
        } else {
            // Starting failed (for example due to blocked microphone permissions).
            // Revert UI state so we don't stay stuck in the recording overlay.
            tm.cancel_stream();
            utils::hide_recording_overlay(app);
            set_tray_state(app, TrayIconState::Idle);
            if let Some(err) = recording_error {
                let error_type = if is_microphone_access_denied(&err) {
                    "microphone_permission_denied"
                } else if is_no_input_device_error(&err) {
                    "no_input_device"
                } else {
                    "unknown"
                };
                let _ = app.emit(
                    "recording-error",
                    RecordingErrorEvent {
                        error_type: error_type.to_string(),
                        detail: Some(err),
                    },
                );
            }
        }

        debug!(
            "TranscribeAction::start completed in {:?}",
            start_time.elapsed()
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        // Prevent a slow microphone from emitting a ready event or start chime
        // after the user has already requested stop.
        app.state::<Arc<AudioRecordingManager>>()
            .invalidate_recording_readiness();

        // Unregister the cancel shortcut when transcription stops
        shortcut::unregister_cancel_shortcut(app);

        let stop_time = Instant::now();
        debug!("TranscribeAction::stop called for binding: {}", binding_id);

        let ah = app.clone();
        let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
        let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
        let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

        set_tray_state(app, TrayIconState::Transcribing);
        // Stop should give immediate visual feedback. Live streaming can keep
        // the larger panel, but it still switches from listening to a working
        // spinner while the stream finalizes. Non-streaming paths use the
        // compact transcribing pill (None no-ops in show_*).
        let style = get_settings(app).overlay_style;
        // Capture this before finalizing the stream so every later working state
        // targets the same overlay that was shown for this transcription.
        let use_streaming_overlay = should_use_streaming_overlay(style, tm.is_streaming());
        if use_streaming_overlay {
            tm.emit_stream_working(StreamWorkKind::Transcribing);
        } else {
            show_transcribing_overlay(app);
        }

        // Unmute before playing audio feedback so the stop sound is audible
        rm.remove_mute();

        // Play audio feedback for recording stop
        play_feedback_sound(app, SoundType::Stop);

        let binding_id = binding_id.to_string(); // Clone binding_id for the async task
                                                 // The stop is the linearization point: the prompt shown now is the one processed.
        let frozen = dictation::freeze_session(app);
        let post_process = frozen.prompt.is_some();
        let captured_rules = self
            .rules_at_start
            .rules_for_stop(&binding_id, || current_matcher(app));
        let cancel_generation = rm.cancel_generation();
        let destination_ticket = crate::destination::delivery::ticket_at_stop(app);

        tauri::async_runtime::spawn(async move {
            let _guard = FinishGuard(ah.clone(), Arc::clone(&tm));
            let _session = SessionEnd(ah.clone(), frozen.session_id);
            debug!(
                "Starting async transcription task for binding: {}",
                binding_id
            );

            let stop_recording_time = Instant::now();
            if let Some(samples) = rm.stop_recording(&binding_id, cancel_generation) {
                debug!(
                    "Recording stopped and samples retrieved in {:?}, sample count: {}",
                    stop_recording_time.elapsed(),
                    samples.len()
                );

                if rm.was_cancelled_since(cancel_generation) {
                    debug!("Transcription operation cancelled after recording stop");
                    tm.cancel_stream();
                    utils::hide_recording_overlay(&ah);
                    set_tray_state(&ah, TrayIconState::Idle);
                    return;
                }

                if samples.is_empty() {
                    debug!("Recording produced no audio samples; skipping persistence");
                    crate::destination::delivery::abandon(&ah);
                    // Tear down any streaming worker so its channel doesn't leak
                    // and block the next start_stream.
                    tm.cancel_stream();
                    utils::hide_recording_overlay(&ah);
                    set_tray_state(&ah, TrayIconState::Idle);
                } else {
                    // Save WAV concurrently with transcription
                    let sample_count = samples.len();
                    let file_name = format!("handy-{}.wav", chrono::Utc::now().timestamp());
                    let wav_path = hm.recordings_dir().join(&file_name);
                    let wav_path_for_verify = wav_path.clone();
                    let samples_for_wav = samples.clone();
                    let wav_handle = tauri::async_runtime::spawn_blocking(move || {
                        crate::audio_toolkit::save_wav_file(&wav_path, &samples_for_wav)
                    });

                    // Transcribe concurrently with WAV save. If a live stream was
                    // running, finalize it and use its text (all audio was already
                    // fed to the stream); otherwise batch-transcribe the samples.
                    let transcription_time = Instant::now();
                    // One compiled matcher serves the stream finalize and its batch
                    // fallback (worker crash or empty stream) of the same audio.
                    let transcription_result = finalize_or_transcribe(
                        &captured_rules,
                        |replacements| tm.finalize_stream(replacements),
                        |replacements| tm.transcribe(samples, replacements),
                    );

                    // Await WAV save and verify
                    let wav_saved = match wav_handle.await {
                        Ok(Ok(())) => {
                            match crate::audio_toolkit::verify_wav_file(
                                &wav_path_for_verify,
                                sample_count,
                            ) {
                                Ok(()) => true,
                                Err(e) => {
                                    error!("WAV verification failed: {}", e);
                                    false
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Failed to save WAV file: {}", e);
                            false
                        }
                        Err(e) => {
                            error!("WAV save task panicked: {}", e);
                            false
                        }
                    };

                    if rm.was_cancelled_since(cancel_generation) {
                        debug!("Transcription operation cancelled before output handling");
                        utils::hide_recording_overlay(&ah);
                        set_tray_state(&ah, TrayIconState::Idle);
                        return;
                    }

                    match transcription_result {
                        Ok(transcription) => {
                            debug!(
                                "Transcription completed in {:?}: '{}'",
                                transcription_time.elapsed(),
                                utils::redact_text(&transcription)
                            );
                            let ports = AppOutputPorts {
                                app: ah.clone(),
                                recording: Arc::clone(&rm),
                                transcription: Arc::clone(&tm),
                                history: Arc::clone(&hm),
                                cancel_generation,
                                frozen,
                                use_streaming_overlay,
                                file_name: wav_saved.then_some(file_name),
                                destination_ticket,
                            };
                            match deliver(&ports, transcription).await {
                                // Insertion and an empty text end the indicator themselves.
                                Delivery::Inserted | Delivery::NothingToInsert => {}
                                Delivery::Cancelled => {
                                    debug!(
                                        "Transcription operation cancelled during output handling"
                                    );
                                    utils::hide_recording_overlay(&ah);
                                    set_tray_state(&ah, TrayIconState::Idle);
                                }
                            }
                        }
                        Err(err) => {
                            if rm.was_cancelled_since(cancel_generation) {
                                debug!(
                                    "Transcription operation cancelled after transcription error"
                                );
                                utils::hide_recording_overlay(&ah);
                                set_tray_state(&ah, TrayIconState::Idle);
                                return;
                            }

                            error!("Transcription failed: {}", err);
                            crate::destination::delivery::abandon(&ah);
                            // Surface the failure to the UI (toast). The full
                            // message is also in handy.log via the line above.
                            let _ = ah.emit("transcription-error", err.to_string());
                            // Save entry with empty text so user can retry (needs the audio)
                            if wav_saved {
                                let entry = NewHistoryEntry {
                                    file_name: Some(file_name),
                                    post_process_requested: post_process,
                                    ..NewHistoryEntry::default()
                                };
                                if let Err(save_err) = hm.save_entry(entry) {
                                    error!("Failed to save failed history entry: {}", save_err);
                                }
                            }
                            utils::hide_recording_overlay(&ah);
                            set_tray_state(&ah, TrayIconState::Idle);
                        }
                    }
                }
            } else {
                debug!("No samples retrieved from recording stop");
                crate::destination::delivery::abandon(&ah);
                // Tear down any streaming worker so its channel doesn't leak.
                tm.cancel_stream();
                utils::hide_recording_overlay(&ah);
                set_tray_state(&ah, TrayIconState::Idle);
            }
        });

        debug!(
            "TranscribeAction::stop completed in {:?}",
            stop_time.elapsed()
        );
    }
}

// Cancel Action
struct CancelAction;

impl ShortcutAction for CancelAction {
    fn start(&self, app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        utils::cancel_current_operation(app);
    }

    fn stop(&self, _app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        // Nothing to do on stop for cancel
    }
}

// Test Action
struct TestAction;

impl ShortcutAction for TestAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Started - {} (App: {})", // Changed "Pressed" to "Started" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Stopped - {} (App: {})", // Changed "Released" to "Stopped" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }
}

/// The action a shortcut of `binding_id` runs: its own entry of `ACTION_MAP`, or for a prompt
/// shortcut (005) the post-processing dictation, whose session then takes that prompt.
pub fn action_for(binding_id: &str) -> Option<Arc<dyn ShortcutAction>> {
    let id = match crate::shortcut::prompt_targets::prompt_of(binding_id) {
        Some(_) => crate::shortcut::prompt_targets::POST_PROCESS_ACTION,
        None => binding_id,
    };
    ACTION_MAP.get(id).cloned()
}

// Static Action Map
pub static ACTION_MAP: Lazy<HashMap<String, Arc<dyn ShortcutAction>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "transcribe".to_string(),
        Arc::new(TranscribeAction {
            rules_at_start: DictationRuleCapture::default(),
        }) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "transcribe_with_post_process".to_string(),
        Arc::new(TranscribeAction {
            rules_at_start: DictationRuleCapture::default(),
        }) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "cancel".to_string(),
        Arc::new(CancelAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "test".to_string(),
        Arc::new(TestAction) as Arc<dyn ShortcutAction>,
    );
    map
});

#[cfg(test)]
mod tests {
    use super::should_use_streaming_overlay;
    use crate::settings::OverlayStyle;

    #[test]
    fn live_overlay_uses_streaming_states_only_for_streaming_models() {
        assert!(should_use_streaming_overlay(OverlayStyle::Live, true));
        assert!(!should_use_streaming_overlay(OverlayStyle::Live, false));
        assert!(!should_use_streaming_overlay(OverlayStyle::Minimal, true));
        assert!(!should_use_streaming_overlay(OverlayStyle::None, true));
    }
}
