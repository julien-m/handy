//! Scripted pasteboard, keys, destination and read source wired into the
//! production coordinator (no mock coordinator).

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::clock::{Clock, ManualClock};
use handy_app_lib::smart_paste::coordinator::{SmartPasteCoordinator, SmartPastePorts};
use handy_app_lib::smart_paste::ids::{
    DestinationGeneration, DictationId, TransactionId, Transcript,
};
use handy_app_lib::smart_paste::key_events::{KeyEventPort, PasteKey};
use handy_app_lib::smart_paste::keyboard::{InputSourceKind, KeyboardError};
use handy_app_lib::smart_paste::outcome::PasteDiagnostic;
use handy_app_lib::smart_paste::pasteboard_port::ReadEvent;
use handy_app_lib::smart_paste::request::{
    DestinationContext, DestinationGuard, DiagnosticSink, InsertionRequest, ReadEvents,
};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy;

use super::fake_pasteboard::Items;
pub use super::fake_pasteboard::{Content, FakePasteboard};
use super::fakes::{FakeAx, FakeInputSources};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Scripted key events; the chord hold advances virtual time.
#[derive(Clone)]
pub struct FakeKeys {
    clock: Arc<dyn Clock>,
    pub state: Arc<Mutex<KeysState>>,
}

/// Recorded key activity and scripted failures.
#[derive(Default)]
pub struct KeysState {
    pub resolve_fails: bool,
    pub chord_fails: bool,
    pub typing_fails: bool,
    pub chords: Vec<Instant>,
    pub resolved: Vec<Instant>,
    pub typed: Vec<String>,
    pub on_chord: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Makes the chord unwind, as a panicking native port would.
    pub chord_unwinds: bool,
}

impl FakeKeys {
    /// Creates keys driven by `clock`.
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            state: Arc::new(Mutex::new(KeysState::default())),
        }
    }

    /// Locks the recorded state.
    pub fn state(&self) -> MutexGuard<'_, KeysState> {
        lock(&self.state)
    }
}

impl KeyEventPort for FakeKeys {
    fn resolve_paste_key(&mut self) -> Result<PasteKey, KeyboardError> {
        let mut state = self.state();
        state.resolved.push(self.clock.now());
        if state.resolve_fails {
            return Err(KeyboardError::MainThreadUnavailable);
        }
        Ok(PasteKey(9))
    }

    fn post_paste_chord(&mut self, _key: PasteKey, hold: Duration) -> Result<(), KeyboardError> {
        let hook = {
            let mut state = self.state();
            state.chords.push(self.clock.now());
            if state.chord_fails {
                return Err(KeyboardError::PostFailed);
            }
            if state.chord_unwinds {
                drop(state);
                std::panic::resume_unwind(Box::new("scripted native port failure"));
            }
            state.on_chord.clone()
        };
        self.clock.sleep(hold);
        hook.iter().for_each(|on_chord| on_chord());
        Ok(())
    }

    fn type_unicode(&mut self, text: &str) -> Result<(), KeyboardError> {
        let mut state = self.state();
        if state.typing_fails {
            return Err(KeyboardError::PostFailed);
        }
        state.typed.push(text.to_string());
        Ok(())
    }
}

/// Destination guard answering from a script (default: still current).
#[derive(Debug, Clone, Default)]
pub struct FakeDestination {
    pub answers: Arc<Mutex<VecDeque<bool>>>,
}

impl DestinationGuard for FakeDestination {
    fn is_current(&mut self, _destination: &DestinationContext) -> bool {
        lock(&self.answers).pop_front().unwrap_or(true)
    }
}

/// Records late-read diagnostics.
#[derive(Debug, Clone, Default)]
pub struct FakeDiagnostics {
    pub received: Arc<Mutex<Vec<PasteDiagnostic>>>,
}

impl DiagnosticSink for FakeDiagnostics {
    fn late_read(&mut self, diagnostic: PasteDiagnostic) {
        lock(&self.received).push(diagnostic);
    }
}

/// Reads scripted relative to the latest chord's key-down, in ms (negative:
/// served before the keystroke). By default every read is reported, which
/// tests the reducer; [`ScriptedReads::with_appkit_cache`] reports only the
/// first read of each item, as AppKit does once it cached the data.
pub struct ScriptedReads {
    clock: ManualClock,
    keys: FakeKeys,
    pasteboard: FakePasteboard,
    pending: VecDeque<i64>,
    reports_only_first_read: bool,
    reported: HashSet<TransactionId>,
}

impl ScriptedReads {
    /// Reads at `offsets` (ms, ascending) of the item published at that time.
    pub fn new(rig: &Rig, offsets: &[i64]) -> Self {
        Self {
            clock: rig.clock.clone(),
            keys: rig.keys.clone(),
            pasteboard: rig.pasteboard.clone(),
            pending: offsets.iter().copied().collect(),
            reports_only_first_read: false,
            reported: HashSet::new(),
        }
    }

    /// Reports only the first read of each item, like the native provider.
    pub fn with_appkit_cache(mut self) -> Self {
        self.reports_only_first_read = true;
        self
    }

    fn next_at(&self) -> Option<Instant> {
        let key_down = *self.keys.state().chords.last()?;
        let offset = *self.pending.front()?;
        let magnitude = Duration::from_millis(offset.unsigned_abs());
        Some(if offset < 0 {
            key_down.checked_sub(magnitude).unwrap_or(key_down)
        } else {
            key_down + magnitude
        })
    }

    /// Delivers every remaining read to an idle coordinator, processing
    /// deadlines in between, as the actor would.
    pub fn drain_into(&mut self, coordinator: &mut SmartPasteCoordinator) {
        while let Some(event) = self.next_read(far_future(&self.clock)) {
            settle_until(&self.clock, coordinator, event.at);
            coordinator.handle_read(event);
        }
    }
}

impl ReadEvents for ScriptedReads {
    fn next_read(&mut self, deadline: Instant) -> Option<ReadEvent> {
        // Each pass serves one scripted read; a read AppKit would answer from
        // its cache produces no event and the next one is tried.
        loop {
            match (self.next_at(), self.pasteboard.published()) {
                (Some(at), Some(transaction)) if at <= deadline => {
                    self.pending.pop_front();
                    self.clock.advance_to(at);
                    if self.reports_only_first_read && !self.reported.insert(transaction) {
                        continue;
                    }
                    return Some(ReadEvent { transaction, at });
                }
                _ => {
                    self.clock.advance_to(deadline);
                    return None;
                }
            }
        }
    }
}

fn far_future(clock: &ManualClock) -> Instant {
    clock.now() + Duration::from_secs(3600)
}

/// Advances virtual time to `until`, firing every coordinator deadline on
/// the way, as the actor's timers would.
pub fn settle_until(clock: &ManualClock, coordinator: &mut SmartPasteCoordinator, until: Instant) {
    while let Some(deadline) = coordinator
        .next_deadline()
        .filter(|deadline| *deadline <= until)
    {
        clock.advance_to(deadline);
        coordinator.handle_deadline();
    }
    clock.advance_to(until);
}

/// All fakes of one scenario.
pub struct Rig {
    pub clock: ManualClock,
    pub pasteboard: FakePasteboard,
    pub keys: FakeKeys,
    pub sources: FakeInputSources,
    pub ax: FakeAx,
    pub destination: FakeDestination,
    pub diagnostics: FakeDiagnostics,
}

/// Rich previous clipboard: text with formatting and an image.
pub fn rich_items() -> Items {
    vec![
        vec![
            ("public.utf8-plain-text".into(), b"previous".to_vec()),
            ("public.rtf".into(), b"{\\rtf1 previous}".to_vec()),
        ],
        vec![("public.png".into(), vec![0x89, b'P', b'N', b'G'])],
    ]
}

impl Rig {
    /// A US layout, no focused AX field (direct insertion skipped) and a
    /// rich previous clipboard.
    pub fn new() -> Self {
        let clock = ManualClock::new();
        let shared: Arc<dyn Clock> = Arc::new(clock.clone());
        let mut ax = FakeAx::with_field(&super::fakes::ScriptedField::new(
            &clock,
            7,
            "",
            handy_app_lib::smart_paste::ax_port::Utf16Range {
                location: 0,
                length: 0,
            },
        ));
        ax.field = None;
        Self {
            pasteboard: FakePasteboard::new(&clock, rich_items()),
            keys: FakeKeys::new(shared),
            sources: FakeInputSources::new(
                &clock,
                "com.apple.keylayout.US",
                InputSourceKind::KeyboardLayout,
                &[],
            ),
            ax,
            destination: FakeDestination::default(),
            diagnostics: FakeDiagnostics::default(),
            clock,
        }
    }

    /// Production coordinator over this rig's fakes.
    pub fn coordinator(&self) -> SmartPasteCoordinator {
        SmartPasteCoordinator::new(self.ports())
    }

    /// Ports over this rig's fakes (shared state with the test).
    pub fn ports(&self) -> SmartPastePorts {
        SmartPastePorts {
            clock: Arc::new(self.clock.clone()),
            ax: Box::new(self.ax.clone()),
            pasteboard: Box::new(self.pasteboard.clone()),
            input_sources: Box::new(self.sources.clone()),
            keys: Box::new(self.keys.clone()),
            destination: Box::new(self.destination.clone()),
            diagnostics: Box::new(self.diagnostics.clone()),
        }
    }

    /// Late-read diagnostics received so far.
    pub fn diagnostics(&self) -> Vec<PasteDiagnostic> {
        lock(&self.diagnostics.received).clone()
    }
}

/// Request for dictation `id` into `bundle` (pid 7, generation 1).
pub fn request(
    id: u64,
    text: &str,
    bundle: Option<&str>,
    policy: ClipboardPolicy,
) -> InsertionRequest {
    InsertionRequest {
        dictation_id: DictationId(id),
        transcript: Transcript::new(text),
        destination: DestinationContext {
            pid: Some(7),
            bundle_id: bundle.map(str::to_string),
            generation: DestinationGeneration(1),
        },
        clipboard_policy: policy,
        paste_delay: Duration::from_millis(60),
        verifies_paste: false,
    }
}
