//! The microphone actually used, kept apart from the saved order (AC-002): the stream open now
//! and the device of the last recording since launch. Session-local: nothing here is persisted,
//! so after a restart the input is "not used yet". Idle stream openings never overwrite the last
//! recording.
use super::types::EffectiveReason;
use serde::Serialize;

/// One opened input.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct EffectiveInput {
    /// Device name read from the opened stream, `None` when the backend cannot name it.
    pub name: Option<String>,
    /// Configured choice that opened, `None` for the system default.
    #[serde(skip)]
    pub choice: Option<String>,
    pub reason: EffectiveReason,
    /// Capture generation of the recording that used it (the latest one for an idle stream).
    pub capture_generation: u64,
}

/// What the effective input describes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveScope {
    /// A recording is running, or the always-on stream is open, on this input.
    Current,
    /// No current stream: the input of the last recording since launch.
    LastRecording,
    /// No recording since launch.
    NotUsedYet,
}

/// The effective input as reported to the settings UI and `handy settings microphones list`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct EffectiveView {
    pub scope: EffectiveScope,
    pub input: Option<EffectiveInput>,
}

/// Current stream and last recording of this run.
#[derive(Debug, Default)]
pub struct EffectiveState {
    current: Option<EffectiveInput>,
    last_recording: Option<EffectiveInput>,
}
impl EffectiveState {
    /// A stream opened on `input`.
    pub fn stream_opened(&mut self, input: EffectiveInput) {
        self.current = Some(input);
    }
    /// The stream closed (or was lost).
    pub fn stream_closed(&mut self) {
        self.current = None;
    }
    /// A recording with `generation` started on the current stream.
    pub fn recording_started(&mut self, generation: u64) {
        if let Some(current) = &mut self.current {
            current.capture_generation = generation;
            self.last_recording = Some(current.clone());
        }
    }
    /// Configured choice of the current stream: `Some(None)` for the system default, `None`
    /// without a stream.
    pub fn current_choice(&self) -> Option<Option<&str>> {
        self.current.as_ref().map(|input| input.choice.as_deref())
    }
    /// What to report: the current input while `current_reported` (a recording runs or the
    /// always-on stream is open), otherwise the last recording's.
    pub fn view(&self, current_reported: bool) -> EffectiveView {
        let (scope, input) = match (&self.current, &self.last_recording) {
            (Some(current), _) if current_reported => (EffectiveScope::Current, Some(current)),
            (_, Some(last)) => (EffectiveScope::LastRecording, Some(last)),
            _ => (EffectiveScope::NotUsedYet, None),
        };
        EffectiveView {
            scope,
            input: input.cloned(),
        }
    }
}

#[cfg(test)]
#[path = "effective_tests.rs"]
mod tests;
