//! Scripted native ports for smart-paste tests. Fakes record every effect so
//! tests can assert what was (and was not) done.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::ax_port::{AxError, AxField, AxTextPort, Utf16Range};
use handy_app_lib::smart_paste::clock::{Clock, ManualClock};
use handy_app_lib::smart_paste::direct::expected_value;
use handy_app_lib::smart_paste::keyboard::{
    InputSourceInfo, InputSourceKind, InputSourcePort, KeyboardError,
};

/// How a scripted field reacts to `AXSelectedText`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBehavior {
    /// Applies the replacement after `delay` (zero: immediately).
    ApplyAfter(Duration),
    /// Ignores the write.
    Ignore,
    /// Applies only the first UTF-16 unit of the transcript.
    ApplyPartially,
}

/// Mutable state of a scripted AX field.
#[derive(Debug)]
pub struct FieldState {
    pub pid: i32,
    pub is_secure: bool,
    pub value: Vec<u16>,
    pub range: Utf16Range,
    pub behavior: WriteBehavior,
    pub setter_result: Result<(), AxError>,
    pub value_error: Option<AxError>,
    pub fail_reads_after_write: bool,
    /// Reads after the write that time out once each before reads succeed
    /// again, as a busy application would.
    pub transient_read_errors: usize,
    pub call_cost: Duration,
    pub writes: usize,
    pub timeouts: Vec<Duration>,
    pending: Option<(Instant, Vec<u16>)>,
}

/// Scripted focused field shared between the test and the fake port.
#[derive(Debug, Clone)]
pub struct ScriptedField {
    clock: ManualClock,
    state: Arc<Mutex<FieldState>>,
}

impl ScriptedField {
    /// Creates a field of process `pid` holding `value` with `range` selected.
    pub fn new(clock: &ManualClock, pid: i32, value: &str, range: Utf16Range) -> Self {
        let state = FieldState {
            pid,
            is_secure: false,
            value: value.encode_utf16().collect(),
            range,
            behavior: WriteBehavior::ApplyAfter(Duration::ZERO),
            setter_result: Ok(()),
            value_error: None,
            fail_reads_after_write: false,
            transient_read_errors: 0,
            call_cost: Duration::ZERO,
            writes: 0,
            timeouts: Vec::new(),
            pending: None,
        };
        Self {
            clock: clock.clone(),
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Locks the scripted state for setup or assertions.
    pub fn state(&self) -> MutexGuard<'_, FieldState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Current value after applying any due scheduled write.
    pub fn value_text(&self) -> String {
        let mut state = self.state();
        apply_due_write(&mut state, self.clock.now());
        String::from_utf16_lossy(&state.value)
    }

    fn begin_call(&self, timeout: Option<Duration>) -> MutexGuard<'_, FieldState> {
        let mut state = self.state();
        state.timeouts.extend(timeout);
        self.clock.advance(state.call_cost);
        apply_due_write(&mut state, self.clock.now());
        state
    }
}

fn apply_due_write(state: &mut FieldState, now: Instant) {
    if let Some((due, _)) = &state.pending {
        if now >= *due {
            if let Some((_, value)) = state.pending.take() {
                state.value = value;
            }
        }
    }
}

impl AxField for ScriptedField {
    fn pid(&mut self) -> Result<i32, AxError> {
        Ok(self.begin_call(None).pid)
    }

    fn is_secure(&mut self, timeout: Duration) -> Result<bool, AxError> {
        Ok(self.begin_call(Some(timeout)).is_secure)
    }

    fn value_utf16(&mut self, timeout: Duration, max_units: usize) -> Result<Vec<u16>, AxError> {
        let mut state = self.begin_call(Some(timeout));
        if let Some(error) = state.value_error {
            return Err(error);
        }
        if state.fail_reads_after_write && state.writes > 0 {
            return Err(AxError::Timeout);
        }
        if state.transient_read_errors > 0 && state.writes > 0 {
            state.transient_read_errors -= 1;
            return Err(AxError::Timeout);
        }
        if state.value.len() > max_units {
            return Err(AxError::TooLarge {
                utf16_units: state.value.len(),
            });
        }
        Ok(state.value.clone())
    }

    fn selected_range(&mut self, timeout: Duration) -> Result<Utf16Range, AxError> {
        Ok(self.begin_call(Some(timeout)).range)
    }

    fn set_selected_text(&mut self, text: &[u16], timeout: Duration) -> Result<(), AxError> {
        let now = self.clock.now();
        let mut state = self.begin_call(Some(timeout));
        state.writes += 1;
        let insertion = match state.behavior {
            WriteBehavior::Ignore => return state.setter_result,
            WriteBehavior::ApplyPartially => text.get(..1).unwrap_or_default().to_vec(),
            WriteBehavior::ApplyAfter(_) => text.to_vec(),
        };
        let delay = match state.behavior {
            WriteBehavior::ApplyAfter(delay) => delay,
            _ => Duration::ZERO,
        };
        if let Some(applied) = expected_value(&state.value, state.range, &insertion) {
            state.pending = Some((now + delay, applied));
        }
        state.setter_result
    }
}

/// Scripted accessibility port returning one shared field.
#[derive(Debug, Clone)]
pub struct FakeAx {
    pub is_trusted: bool,
    pub field: Option<ScriptedField>,
    pub focus_requests: Arc<Mutex<usize>>,
}

impl FakeAx {
    /// Creates a trusted port whose focused element is `field`.
    pub fn with_field(field: &ScriptedField) -> Self {
        Self {
            is_trusted: true,
            field: Some(field.clone()),
            focus_requests: Arc::new(Mutex::new(0)),
        }
    }

    /// Number of focused-element requests received.
    pub fn focus_request_count(&self) -> usize {
        *self
            .focus_requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl AxTextPort for FakeAx {
    fn is_trusted(&mut self) -> bool {
        self.is_trusted
    }

    fn focused_field(
        &mut self,
        _pid: i32,
        _timeout: Duration,
    ) -> Result<Box<dyn AxField>, AxError> {
        *self
            .focus_requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner) += 1;
        match &self.field {
            Some(field) => Ok(Box::new(field.clone())),
            None => Err(AxError::NoValue),
        }
    }
}

/// Converts text to UTF-16 code units.
pub fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// Elapsed virtual time since `start`.
pub fn elapsed_since(clock: &ManualClock, start: Instant) -> Duration {
    clock.now() - start
}

/// Scripted Text Input Sources: a current source and the identifiers that
/// can be selected; every selection is recorded with its virtual instant.
#[derive(Debug, Clone)]
pub struct FakeInputSources {
    clock: ManualClock,
    state: Arc<Mutex<InputSourcesState>>,
}

/// Mutable state of [`FakeInputSources`].
#[derive(Debug)]
pub struct InputSourcesState {
    pub current: Option<InputSourceInfo>,
    pub selectable: Vec<String>,
    pub selections: Vec<(String, Instant)>,
}

impl FakeInputSources {
    /// Creates sources whose current one is `id` of `kind`.
    pub fn new(clock: &ManualClock, id: &str, kind: InputSourceKind, selectable: &[&str]) -> Self {
        let state = InputSourcesState {
            current: Some(InputSourceInfo {
                id: id.to_string(),
                kind,
            }),
            selectable: selectable.iter().map(|id| id.to_string()).collect(),
            selections: Vec::new(),
        };
        Self {
            clock: clock.clone(),
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Locks the scripted state for setup or assertions.
    pub fn state(&self) -> MutexGuard<'_, InputSourcesState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl InputSourcePort for FakeInputSources {
    fn current_source(&mut self) -> Result<InputSourceInfo, KeyboardError> {
        self.state()
            .current
            .clone()
            .ok_or(KeyboardError::SourceUnavailable)
    }

    fn select_source(&mut self, id: &str) -> Result<(), KeyboardError> {
        let now = self.clock.now();
        let mut state = self.state();
        if !state.selectable.iter().any(|selectable| selectable == id) {
            return Err(KeyboardError::SelectFailed);
        }
        state.selections.push((id.to_string(), now));
        let kind = InputSourceKind::KeyboardLayout;
        state.current = Some(InputSourceInfo {
            id: id.to_string(),
            kind,
        });
        Ok(())
    }
}
