//! Microphone priority inside the audio manager (feature 001). Every stream opening resolves the
//! committed priority with the bounded resolver (`microphones::policy`) and records the effective
//! input apart from the saved order. A priority change or a lost stream reopens the stream only
//! at a recording boundary, on a worker, never inside a recording: the current session keeps its
//! device and its captured audio. A stream lost during a recording ends it through the normal
//! stop path, so the captured prefix is transcribed and the user is told it was cut short.
use super::{AudioRecordingManager, MicrophoneMode, RecordingState};
use crate::audio_toolkit::AudioRecorder;
use crate::microphones::{
    boundary::{self, DeviceChange, LossAction, StreamFacts},
    catalog::Catalog,
    effective::{EffectiveInput, EffectiveView},
    native,
    policy::{self, Attempt, AttemptResult, OpenFailure, OpenTarget, Preferences, Resolution},
    status::{self, MicrophoneStatus},
};
use crate::settings::{get_settings, AppSettings};
use log::{debug, error, info, warn};
use std::sync::{atomic::Ordering, Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// The lock's data even after a panic elsewhere: every guarded value here stays consistent.
pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Native resolution ports bound to the manager's recorder; `None` probes only (no open).
struct NativePorts<'a> {
    recorder: Option<&'a mut AudioRecorder>,
    started: Instant,
}
impl policy::ResolutionPorts for NativePorts<'_> {
    type Device = cpal::Device;
    /// Name read from the opened stream.
    type Stream = Option<String>;
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
    fn lid_closed(&mut self, budget: Duration) -> Option<bool> {
        native::lid_closed_within(budget)
    }
    fn enumerate(&mut self, budget: Duration) -> Option<Catalog<cpal::Device>> {
        native::enumerate_within(budget)
    }
    fn open(
        &mut self,
        target: OpenTarget<'_, cpal::Device>,
        budget: Duration,
    ) -> Result<Option<String>, OpenFailure> {
        let recorder = self
            .recorder
            .as_deref_mut()
            .ok_or_else(|| OpenFailure::Failed("probe only".into()))?;
        let device = match target {
            OpenTarget::Device(device) => Some(device.clone()),
            OpenTarget::SystemDefault => None,
        };
        match recorder.open_within(device, budget) {
            Ok(()) => Ok(recorder.device_name()),
            Err(error) => Err(open_failure(error.as_ref())),
        }
    }
}
fn open_failure(error: &(dyn std::error::Error + 'static)) -> OpenFailure {
    let timed_out = error
        .downcast_ref::<std::io::Error>()
        .is_some_and(|error| error.kind() == std::io::ErrorKind::TimedOut);
    match timed_out {
        true => OpenFailure::TimedOut,
        false => OpenFailure::Failed(error.to_string()),
    }
}
fn preferences(settings: &AppSettings) -> Preferences<'_> {
    Preferences {
        priority: &settings.microphone_priority,
        clamshell: settings.clamshell_microphone.as_deref(),
    }
}

// @spec FR-003: Ordered bounded opening, preferences untouched — .specs/features/001-microphone-priority/spec.md#fr-003
impl AudioRecordingManager {
    /// Open `recorder` on the input the committed priority resolves to and record it as the
    /// effective input. The settings are only read.
    pub(super) fn open_resolved(&self, recorder: &mut AudioRecorder) -> Result<(), anyhow::Error> {
        let settings = get_settings(&self.app_handle);
        let mut ports = NativePorts {
            recorder: Some(recorder),
            started: Instant::now(),
        };
        let resolution = policy::resolve(&preferences(&settings), &mut ports);
        log_resolution(&resolution, ports.started.elapsed());
        let Some(opened) = resolution.opened else {
            return Err(anyhow::anyhow!(resolution_error(&resolution.attempts)));
        };
        lock(&self.effective).stream_opened(EffectiveInput {
            name: opened.stream,
            choice: opened.choice,
            reason: opened.reason,
            capture_generation: self.capture_generation.load(Ordering::Acquire),
        });
        Ok(())
    }
    /// Whether the open stream is still what the priority prefers now (fresh catalog, no open).
    /// An unobservable catalog keeps the stream.
    fn stream_matches_preference(&self) -> bool {
        let settings = get_settings(&self.app_handle);
        let mut ports = NativePorts {
            recorder: None,
            started: Instant::now(),
        };
        let Some(preferred) = policy::preferred(&preferences(&settings), &mut ports) else {
            return true;
        };
        lock(&self.effective).current_choice() == Some(preferred.as_deref())
    }
    /// At a recording start on an already-open stream (always-on, or warm after a lazy close):
    /// close it when the priority now prefers another input, so the start reopens on that one
    /// before capture begins (AC-004: a reconnected higher choice wins at the next recording).
    pub(super) fn close_stream_if_not_preferred(&self) {
        let healthy = *lock(&self.is_open)
            && !lock(&self.recorder)
                .as_ref()
                .is_some_and(AudioRecorder::needs_reopen);
        if healthy && !self.stream_matches_preference() {
            info!("Microphone priority prefers another input; reopening before capture");
            self.close_generation.fetch_add(1, Ordering::SeqCst);
            self.stop_microphone_stream();
        }
    }
}

/// Most time a status read waits for the device enumeration: well inside the 5 s answer window
/// of a routed CLI call (008), so `list` answers even when the enumeration hangs.
const STATUS_ENUMERATION_BUDGET: Duration = Duration::from_secs(3);

// @spec FR-002: Availability and effective input reported — .specs/features/001-microphone-priority/spec.md#fr-002
impl AudioRecordingManager {
    /// The effective input to report: current while a recording runs or the always-on stream
    /// is open, otherwise the last recording's (AC-002).
    pub fn effective_input(&self) -> EffectiveView {
        let always_on_open =
            matches!(*lock(&self.mode), MicrophoneMode::AlwaysOn) && *lock(&self.is_open);
        lock(&self.effective).view(self.is_recording() || always_on_open)
    }
    /// Status of `configured`: one fresh bounded observation (no device is opened) and the
    /// effective input. Blocks up to `STATUS_ENUMERATION_BUDGET`: call it off the main loop.
    pub fn microphone_status(&self, configured: Vec<String>) -> MicrophoneStatus {
        let catalog = native::enumerate_within(STATUS_ENUMERATION_BUDGET);
        let observed_at = chrono::Utc::now().to_rfc3339();
        status::status(
            configured,
            catalog.as_ref(),
            observed_at,
            self.effective_input(),
        )
    }
}

/// Fixed log of one resolution: timings and attempt kinds only, never a device name or a
/// backend message (they can name devices).
fn log_resolution<S>(resolution: &Resolution<S>, elapsed: Duration) {
    let kinds = resolution
        .attempts
        .iter()
        .map(|attempt| attempt_kind(&attempt.result))
        .collect::<Vec<_>>();
    let reason = resolution.opened.as_ref().map(|opened| opened.reason);
    info!("Microphone resolved in {elapsed:?}: failed attempts {kinds:?}, opened {reason:?}");
}
fn attempt_kind(result: &AttemptResult) -> &'static str {
    match result {
        AttemptResult::Missing => "missing",
        AttemptResult::Ambiguous => "ambiguous",
        AttemptResult::Unresolved => "unresolved",
        AttemptResult::Deadline => "deadline",
        AttemptResult::Open(OpenFailure::TimedOut) => "timed_out",
        AttemptResult::Open(OpenFailure::Failed(_)) => "open_failed",
    }
}
/// Recording error of a resolution that opened nothing: the system default's failure (the last
/// attempt), so a denied permission or a missing device is still recognized.
fn resolution_error(attempts: &[Attempt]) -> String {
    match attempts.last().map(|attempt| &attempt.result) {
        Some(AttemptResult::Open(OpenFailure::Failed(message))) => message.clone(),
        Some(AttemptResult::Open(OpenFailure::TimedOut)) => {
            "Microphone did not open within its time limit".to_owned()
        }
        _ => "No input device found".to_owned(),
    }
}

/// `recording-error` payload of a cut-short recording (same shape as the start errors).
#[derive(Clone, serde::Serialize)]
struct CutShort {
    error_type: &'static str,
    detail: Option<String>,
}

// @spec FR-004: Changes at the next opening, loss ends the take — .specs/features/001-microphone-priority/spec.md#fr-004
impl AudioRecordingManager {
    /// Live effect of a committed priority change: queued on a worker so the commit never waits
    /// for a reopen; the worker defers it to the end of a recording in progress (the session
    /// keeps its device and audio).
    pub fn update_selected_device(&self) -> Result<(), anyhow::Error> {
        self.idle_recovery_armed.store(true, Ordering::Release);
        self.queue_reopen();
        Ok(())
    }
    /// Re-resolve the open stream on a worker; requests arriving before it starts are coalesced
    /// (it reads the latest committed settings).
    pub(super) fn queue_reopen(&self) {
        if self.reopen_queued.swap(true, Ordering::AcqRel) {
            return;
        }
        let manager = self.clone();
        std::thread::spawn(move || {
            manager.reopen_queued.store(false, Ordering::Release);
            manager.apply_device_change();
        });
    }
    /// Serialized against recording starts by the state lock (see `boundary::device_change`).
    fn apply_device_change(&self) {
        let state = lock(&self.state);
        let facts = StreamFacts {
            recording: !matches!(*state, RecordingState::Idle),
            ..self.stream_facts()
        };
        match boundary::device_change(facts, || self.stream_matches_preference()) {
            DeviceChange::Defer => self.pending_reopen.store(true, Ordering::Release),
            DeviceChange::Keep => {}
            DeviceChange::CloseWarm => self.stop_microphone_stream(),
            DeviceChange::Reopen | DeviceChange::Open => {
                self.close_generation.fetch_add(1, Ordering::SeqCst);
                self.stop_microphone_stream();
                if let Err(error) = self.start_microphone_stream() {
                    error!("Could not reopen the always-on microphone: {error}");
                }
            }
        }
        drop(state);
    }
    /// Mode, open flag and loss of the stream (the recording flag is the caller's).
    fn stream_facts(&self) -> StreamFacts {
        StreamFacts {
            recording: self.is_recording(),
            always_on: matches!(*lock(&self.mode), MicrophoneMode::AlwaysOn),
            open: *lock(&self.is_open),
            lost: lock(&self.recorder)
                .as_ref()
                .is_some_and(AudioRecorder::needs_reopen),
        }
    }
    /// End of a recording (stop or cancel): apply a priority change that waited for it, and
    /// reopen an always-on stream lost during it.
    pub(super) fn after_recording(&self) {
        let pending = self.pending_reopen.swap(false, Ordering::AcqRel);
        let facts = self.stream_facts();
        if boundary::reopen_after_recording(pending, facts.lost, facts.always_on) {
            self.queue_reopen();
        }
    }
    /// The open stream reported a loss (called on a worker, never the capture thread).
    pub(super) fn input_lost(&self) {
        lock(&self.effective).stream_closed();
        let recording = matches!(*lock(&self.state), RecordingState::Recording { .. });
        let always_on = matches!(*lock(&self.mode), MicrophoneMode::AlwaysOn);
        let armed = self.idle_recovery_armed.load(Ordering::Acquire);
        match boundary::on_loss(recording, always_on, armed) {
            LossAction::EndRecording => self.end_cut_short_recording(),
            LossAction::Recover => {
                self.idle_recovery_armed.store(false, Ordering::Release);
                debug!("Idle always-on microphone lost; one bounded reopen");
                self.queue_reopen();
            }
            LossAction::Ignore => {}
        }
    }
    /// Tell the user the recording was cut short and stop it through the coordinator, which
    /// runs the normal stop (the captured prefix is transcribed).
    fn end_cut_short_recording(&self) {
        warn!("Microphone lost during a recording; ending it with the captured audio");
        let event = CutShort {
            error_type: "input_lost",
            detail: None,
        };
        if let Err(error) = self.app_handle.emit("recording-error", event) {
            warn!("Could not report the cut-short recording: {error}");
        }
        if let Some(coordinator) = self
            .app_handle
            .try_state::<crate::TranscriptionCoordinator>()
        {
            coordinator.notify_input_lost();
        }
    }
}

/// Loss callback of the recorder: hand the event to a worker so the capture thread never waits
/// on a manager lock (a thread holding one may be joining that capture thread).
pub(super) fn report_input_lost(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        if let Some(manager) = app.try_state::<Arc<AudioRecordingManager>>() {
            manager.input_lost();
        }
    });
}
