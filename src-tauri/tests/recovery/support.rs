//! Test doubles shared by the 010 integration tests: a recording surface, 006's real recovery
//! slot behind the transcript port, scripted session facts, Copy writer, manual timers, a
//! counting `paste-error` fallback and a scripted accessibility port. Only native boundaries
//! are faked; the recovery service, manager, decision and classifier are the production code.
#![allow(dead_code)]

use handy_app_lib::destination::context::DictationId;
use handy_app_lib::destination::outcome::{
    DestinationFailure, HistoryStatus, InsertionOutcome, InsertionStatus, MethodOutcome,
    TranscriptRef,
};
use handy_app_lib::destination::recovery::RecoverySlot;
use handy_app_lib::destination::workspace::SystemClock;
use handy_app_lib::recovery::classifier::{
    AxQueryError, EditAttribute, FocusedElement, FocusedElementPort,
};
use handy_app_lib::recovery::ports::{
    CopyError, CopyPort, FallbackPort, RecoverySurfacePort, SessionFacts, TimerPort,
    TranscriptSource,
};
use handy_app_lib::recovery::service::{RecoveryPorts, RecoveryService};
use handy_app_lib::recovery::types::{BubbleId, RecoveryBubbleView, RecoveryPlatform};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Result of a test.
pub type TestResult = Result<(), Box<dyn std::error::Error>>;
/// A safe phrase; long enough to be cut in the preview.
pub const PHRASE: &str = "safe test phrase kept by the recovery bubble while Handy could not \
insert it, long enough to need more than two lines of the bubble so the preview is cut while \
the full transcript stays complete for Copy and drag, with accents éèà and an emoji 🎙️ at the \
very end of the dictation that only the full payload carries ✅";

/// Lock a test mutex, ignoring poisoning.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the service asked the surface.
#[derive(Clone, Debug, PartialEq)]
pub enum SurfaceCall {
    Present(RecoveryBubbleView),
    Update(RecoveryBubbleView),
    Hide,
    Destroy(BubbleId),
    BeginDrag(BubbleId, String),
    ShowAgain(RecoveryBubbleView),
}
/// Surface double recording every call.
#[derive(Default)]
pub struct RecordingSurface {
    pub calls: Mutex<Vec<SurfaceCall>>,
}
impl RecordingSurface {
    /// Every call so far.
    pub fn calls(&self) -> Vec<SurfaceCall> {
        lock(&self.calls).clone()
    }
    /// The views presented so far.
    pub fn presented(&self) -> Vec<RecoveryBubbleView> {
        self.calls()
            .into_iter()
            .filter_map(|call| match call {
                SurfaceCall::Present(view) => Some(view),
                _ => None,
            })
            .collect()
    }
    fn push(&self, call: SurfaceCall) {
        lock(&self.calls).push(call);
    }
}
impl RecoverySurfacePort for RecordingSurface {
    fn present(&self, view: RecoveryBubbleView) {
        self.push(SurfaceCall::Present(view));
    }
    fn update(&self, view: RecoveryBubbleView) {
        self.push(SurfaceCall::Update(view));
    }
    fn hide(&self) {
        self.push(SurfaceCall::Hide);
    }
    fn destroy(&self, bubble_id: BubbleId) {
        self.push(SurfaceCall::Destroy(bubble_id));
    }
    fn begin_drag(&self, bubble_id: BubbleId, transcript: String) {
        self.push(SurfaceCall::BeginDrag(bubble_id, transcript));
    }
    fn show_again(&self, view: RecoveryBubbleView) {
        self.push(SurfaceCall::ShowAgain(view));
    }
}

/// 006's real slot behind the transcript port, with a release log.
#[derive(Default)]
pub struct Slot {
    pub slot: RecoverySlot,
    pub released: Mutex<Vec<TranscriptRef>>,
}
impl TranscriptSource for Slot {
    fn read(&self, reference: TranscriptRef) -> Option<String> {
        self.slot.read(reference).ok()
    }
    fn release(&self, reference: TranscriptRef) {
        lock(&self.released).push(reference);
        self.slot.release(reference);
    }
}

/// Session facts double: a fixed name, no decision instant.
pub struct Facts {
    pub name: Option<String>,
}
impl SessionFacts for Facts {
    fn app_name(&self, _outcome: &InsertionOutcome) -> Option<String> {
        self.name.clone()
    }
    fn decided_at(&self, _outcome: &InsertionOutcome) -> Option<Instant> {
        None
    }
}

/// Copy writer answering a scripted result and logging what it wrote; it can signal that a
/// write started and hold it, like a Copy queued behind an insertion in 007's actor.
pub struct Copier {
    pub answer: Mutex<Result<(), CopyError>>,
    pub written: Mutex<Vec<String>>,
    pub entered: Mutex<Option<std::sync::mpsc::Sender<()>>>,
    pub hold: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
}
impl CopyPort for Copier {
    fn write(&self, transcript: &str) -> Result<(), CopyError> {
        if let Some(entered) = lock(&self.entered).take() {
            let _ = entered.send(());
        }
        if let Some(hold) = lock(&self.hold).take() {
            let _ = hold.recv();
        }
        lock(&self.written).push(transcript.to_owned());
        *lock(&self.answer)
    }
}

/// One scheduled task and its delay.
pub type Scheduled = (Duration, Box<dyn FnOnce() + Send>);
/// Timers fired by the test.
#[derive(Default)]
pub struct ManualTimers {
    pub tasks: Mutex<Vec<Scheduled>>,
}
impl ManualTimers {
    /// Run every scheduled task; returns their delays.
    pub fn fire_all(&self) -> Vec<Duration> {
        let tasks: Vec<_> = lock(&self.tasks).drain(..).collect();
        tasks
            .into_iter()
            .map(|(delay, task)| {
                task();
                delay
            })
            .collect()
    }
}
impl TimerPort for ManualTimers {
    fn after(&self, delay: Duration, task: Box<dyn FnOnce() + Send>) {
        lock(&self.tasks).push((delay, task));
    }
}

/// `paste-error` double counting the toasts.
#[derive(Default)]
pub struct Fallback {
    pub toasts: Mutex<Vec<HistoryStatus>>,
}
impl FallbackPort for Fallback {
    fn paste_error(&self, history: HistoryStatus) {
        lock(&self.toasts).push(history);
    }
}

/// Scripted accessibility: trusted or not, one focused element, and a query counter.
#[derive(Clone)]
pub struct Focus {
    pub trusted: bool,
    pub pid: u32,
    pub role: &'static str,
    pub queries: Arc<AtomicUsize>,
    /// Runs during the classification (to change the world under the guard).
    pub during: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl Focus {
    /// A trusted focus of `role` in process `pid`.
    pub fn role(role: &'static str, pid: u32) -> Self {
        Self {
            trusted: true,
            pid,
            role,
            queries: Arc::new(AtomicUsize::new(0)),
            during: None,
        }
    }
    /// Accessibility queries made so far.
    pub fn queries(&self) -> usize {
        self.queries.load(Ordering::SeqCst)
    }
}
struct Element(Focus);
impl FocusedElementPort for Focus {
    fn is_trusted(&mut self) -> bool {
        self.trusted
    }
    fn focused_element(
        &mut self,
        _pid: u32,
        _timeout: Duration,
    ) -> Result<Box<dyn FocusedElement>, AxQueryError> {
        self.queries.fetch_add(1, Ordering::SeqCst);
        if let Some(during) = &self.during {
            during();
        }
        Ok(Box::new(Element(self.clone())))
    }
}
impl FocusedElement for Element {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.0.pid)
    }
    fn role(&mut self, _timeout: Duration) -> Result<String, AxQueryError> {
        self.0.queries.fetch_add(1, Ordering::SeqCst);
        Ok(self.0.role.to_owned())
    }
    fn subrole(&mut self, _timeout: Duration) -> Result<Option<String>, AxQueryError> {
        Ok(None)
    }
    fn is_settable(
        &mut self,
        _attribute: EditAttribute,
        _timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        Ok(self.0.role == "AXTextArea")
    }
}

/// A service over recording fakes, and the fakes.
pub struct Bench {
    pub service: Arc<RecoveryService>,
    pub surface: Arc<RecordingSurface>,
    pub slot: Arc<Slot>,
    pub copier: Arc<Copier>,
    pub timers: Arc<ManualTimers>,
    pub fallback: Arc<Fallback>,
}
impl Bench {
    /// A bench on `platform` whose focus is a not-editable list of pid 42.
    pub fn new(platform: RecoveryPlatform) -> Self {
        Self::with_focus(platform, Focus::role("AXList", 42))
    }
    /// A bench on `platform` with the scripted `focus`.
    pub fn with_focus(platform: RecoveryPlatform, focus: Focus) -> Self {
        let surface = Arc::new(RecordingSurface::default());
        let slot = Arc::new(Slot::default());
        let copier = Arc::new(Copier {
            answer: Mutex::new(Ok(())),
            written: Mutex::new(Vec::new()),
            entered: Mutex::new(None),
            hold: Mutex::new(None),
        });
        let timers = Arc::new(ManualTimers::default());
        let fallback = Arc::new(Fallback::default());
        let ports = RecoveryPorts {
            surface: surface.clone(),
            transcripts: slot.clone(),
            facts: Arc::new(Facts {
                name: Some("Notes".to_owned()),
            }),
            copy: copier.clone(),
            timers: timers.clone(),
            fallback: fallback.clone(),
            focus: Mutex::new(Box::new(focus)),
            clock: Arc::new(SystemClock),
        };
        Self {
            service: RecoveryService::new(ports, platform),
            surface,
            slot,
            copier,
            timers,
            fallback,
        }
    }
    /// A failed outcome of dictation `dictation`, its transcript retained in the real slot.
    pub fn failed(
        &self,
        dictation: u64,
        reason: Option<DestinationFailure>,
        method_outcome: Option<MethodOutcome>,
    ) -> InsertionOutcome {
        let reference = self
            .slot
            .slot
            .retain(DictationId(dictation), PHRASE.to_owned());
        outcome(dictation, reason, method_outcome, Some(reference))
    }
    /// Toasts emitted by 010 so far.
    pub fn toasts(&self) -> Vec<HistoryStatus> {
        lock(&self.fallback.toasts).clone()
    }
    /// Slot references released so far.
    pub fn released(&self) -> Vec<TranscriptRef> {
        lock(&self.slot.released).clone()
    }
}

/// A failed outcome of `dictation` with `reason`, `method_outcome` and its slot reference.
pub fn outcome(
    dictation: u64,
    reason: Option<DestinationFailure>,
    method_outcome: Option<MethodOutcome>,
    transcript_ref: Option<TranscriptRef>,
) -> InsertionOutcome {
    InsertionOutcome {
        dictation_id: DictationId(dictation),
        destination_generation: dictation + 100,
        outcome_id: dictation,
        target: None,
        destination_source: handy_app_lib::destination::context::DestinationSource::Captured,
        status: InsertionStatus::Failed,
        method_outcome,
        transaction_id: Some("7".to_owned()),
        reason,
        history_status: HistoryStatus::Saved,
        history_entry_id: Some(9),
        transcript_ref,
        completed_at: 0,
    }
}
