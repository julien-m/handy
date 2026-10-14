//! Full dictation chain of step 7 with real time: 006's orchestrator,
//! manager and pre-dispatch guard, smart paste as its insertion method, the
//! production destination port over the registered guard, and the real
//! actor thread over scripted native ports (AC-012, AC-018).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use handy_app_lib::destination::context::{
    ApplicationIdentity, NoRecoveryConsumer, RecoveryDisposition,
};
use handy_app_lib::destination::insertion::{
    InsertionReport, InsertionRequest as Delivery, Orchestrator,
};
use handy_app_lib::destination::manager::DestinationManager;
use handy_app_lib::destination::outcome::{InsertionStatus, MethodOutcome};
use handy_app_lib::destination::workspace::{
    ActivationRequest, Foreground, RunningApp, SystemClock as WallClock, WorkspacePort,
};
use handy_app_lib::settings_service::platform_capability::{
    PlatformCapabilities, PlatformCapability,
};
use handy_app_lib::smart_paste::actor::SmartPasteActor;
use handy_app_lib::smart_paste::clock::{Clock, ManualClock, SystemClock};
use handy_app_lib::smart_paste::coordinator::SmartPastePorts;
use handy_app_lib::smart_paste::destination_bridge::{DestinationChecks, RegisteredDestinations};
use handy_app_lib::smart_paste::dispatch::{ActorSlot, InsertionInputs, SmartPasteMethod};
use handy_app_lib::smart_paste::ids::Transcript;
use handy_app_lib::smart_paste::keyboard::InputSourceKind;
use handy_app_lib::smart_paste::pasteboard_port::{ReadEvent, ReadSink};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy;

use super::fakes::{FakeAx, FakeInputSources, ScriptedField};
use super::rig::{rich_items, FakeDiagnostics, FakeKeys, FakePasteboard};

// @spec FR-008: Pipeline Enter recovery handover contract — .specs/features/007-smart-paste/spec.md#fr-008
// @spec FR-011: AX clipboard recovery ordering — .specs/features/007-smart-paste/spec.md#fr-011

const TARGET_READ_DELAY: Duration = Duration::from_millis(20);

/// Foreground answering a script, then a fixed application.
struct ScriptedWorkspace {
    script: Mutex<VecDeque<RunningApp>>,
    steady: RunningApp,
}
impl WorkspacePort for ScriptedWorkspace {
    fn frontmost(&self) -> Foreground {
        let mut script = self.script.lock().unwrap_or_else(PoisonError::into_inner);
        Foreground::Application(script.pop_front().unwrap_or_else(|| self.steady.clone()))
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        vec![self.steady.clone()]
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        let steady = self.steady.clone();
        (steady.identity == *identity)
            .then_some(steady)
            .into_iter()
            .collect()
    }
    fn activate_once(&self, _target: &RunningApp) -> ActivationRequest {
        ActivationRequest::Sent
    }
}

fn running(pid: u32, bundle: &str) -> RunningApp {
    RunningApp {
        pid,
        identity: ApplicationIdentity::BundleId(bundle.into()),
        display_name: bundle.into(),
        icon_ref: None,
    }
}

/// Reads the published item once, shortly after the chord, when `reads`.
fn spawn_actor(
    pasteboard: &FakePasteboard,
    checks: &Arc<DestinationChecks>,
    reads: bool,
) -> std::io::Result<(SmartPasteActor, FakeKeys)> {
    let (pasteboard, checks) = (pasteboard.clone(), Arc::clone(checks));
    let keys = FakeKeys::new(Arc::new(SystemClock));
    let shared_keys = keys.clone();
    let actor = SmartPasteActor::spawn(move |sink: ReadSink| {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let manual = ManualClock::new();
        if reads {
            keys.state().on_chord = Some(read_once(&pasteboard, sink));
        }
        let mut ax = FakeAx::with_field(&ScriptedField::new(&manual, 7, "", Default::default()));
        ax.field = None;
        let layout = "com.apple.keylayout.US";
        let sources = FakeInputSources::new(&manual, layout, InputSourceKind::KeyboardLayout, &[]);
        SmartPastePorts {
            clock,
            ax: Box::new(ax),
            pasteboard: Box::new(pasteboard),
            input_sources: Box::new(sources),
            keys: Box::new(keys),
            destination: Box::new(RegisteredDestinations(checks)),
            diagnostics: Box::new(FakeDiagnostics::default()),
        }
    })?;
    Ok((actor, shared_keys))
}

fn read_once(pasteboard: &FakePasteboard, sink: ReadSink) -> Arc<dyn Fn() + Send + Sync> {
    let pasteboard = pasteboard.clone();
    Arc::new(move || {
        let Some(transaction) = pasteboard.published() else {
            return;
        };
        let sink = Arc::clone(&sink);
        std::thread::spawn(move || {
            std::thread::sleep(TARGET_READ_DELAY);
            sink(ReadEvent {
                transaction,
                at: Instant::now(),
            });
        });
    })
}

/// A dictation with destination return off: whatever is in front at the end.
fn dictation(workspace: Arc<ScriptedWorkspace>) -> (Orchestrator, Delivery) {
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let manager = Arc::new(DestinationManager::new(
        workspace,
        Arc::new(WallClock),
        platform,
    ));
    let started = manager.commit_start(manager.capture(false));
    manager.mark_processing();
    let delivery = Delivery {
        dictation_id: started.dictation_id,
        destination_generation: started.destination_generation,
        paste_method_none: false,
        cancelled: Box::new(|| false),
    };
    let (clock, recovery) = (Arc::new(WallClock), Arc::new(NoRecoveryConsumer));
    let orchestrator = Orchestrator {
        manager,
        clock: clock.clone(),
        pause: clock,
        recovery,
    };
    (orchestrator, delivery)
}

fn method(slot: &ActorSlot, checks: &Arc<DestinationChecks>) -> SmartPasteMethod {
    SmartPasteMethod {
        submitter: Arc::new(slot.clone()),
        checks: Arc::clone(checks),
        inputs: InsertionInputs {
            transcript: Transcript::new("hello world"),
            clipboard_policy: ClipboardPolicy::DontModify,
            paste_delay: Duration::from_millis(20),
        },
        follow_up: Box::new(|_, _| {}),
    }
}

fn disposition(orchestrator: &Orchestrator) -> Option<RecoveryDisposition> {
    let snapshot = orchestrator.manager.snapshot();
    snapshot.map(|snapshot| snapshot.disposition)
}

/// One dictation wired end to end, with `steady` in front unless scripted.
struct Chain {
    workspace: Arc<ScriptedWorkspace>,
    orchestrator: Orchestrator,
    slot: ActorSlot,
    checks: Arc<DestinationChecks>,
    keys: FakeKeys,
}

type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

fn chain(steady: RunningApp, reads: bool) -> Fallible<(Chain, Delivery)> {
    let script = Mutex::new(VecDeque::new());
    let workspace = Arc::new(ScriptedWorkspace { script, steady });
    let (orchestrator, delivery) = dictation(Arc::clone(&workspace));
    let checks = Arc::new(DestinationChecks::default());
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let (actor, keys) = spawn_actor(&pasteboard, &checks, reads)?;
    let slot = ActorSlot::new(actor);
    let chain = Chain {
        workspace,
        orchestrator,
        slot,
        checks,
        keys,
    };
    Ok((chain, delivery))
}

impl Chain {
    fn deliver(&self, delivery: Delivery) -> InsertionReport {
        let method = method(&self.slot, &self.checks);
        self.orchestrator.deliver(delivery, &method)
    }
    fn script(&self) -> std::sync::MutexGuard<'_, VecDeque<RunningApp>> {
        self.workspace
            .script
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
    fn stop(self) -> Fallible<()> {
        let mut actor = self.slot.take().ok_or("actor gone")?;
        actor
            .shutdown(Duration::from_secs(10))
            .map_err(|error| format!("{error:?}"))?;
        Ok(())
    }
}

#[test]
fn a_read_after_the_keystroke_is_an_attempt_006_keeps_for_enter() -> Fallible<()> {
    let (chain, delivery) = chain(running(31, "com.apple.Notes"), true)?;
    let (dictation_id, generation) = (delivery.dictation_id, delivery.destination_generation);
    let report = chain.deliver(delivery);
    let result = report.method_result.ok_or("no method result")?;
    assert_eq!(result.method_outcome(), Some(MethodOutcome::ReadObserved));
    assert!(result.transaction_id().is_some());
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(chain.keys.state().chords.len(), 1);
    let orchestrator = &chain.orchestrator;
    assert_eq!(disposition(orchestrator), Some(RecoveryDisposition::Insert));
    assert!(orchestrator
        .manager
        .claim_enter(dictation_id, generation)
        .is_ok());
    chain.stop()
}

#[test]
fn another_application_in_front_at_the_keystroke_stops_the_paste_for_recovery() -> Fallible<()> {
    let notes = running(31, "com.apple.Notes");
    let (chain, delivery) = chain(running(22, "com.apple.Safari"), false)?;
    // Guard first pass and the coordinator's preflight see Notes; the check
    // right before the keystroke sees Safari.
    chain.script().extend([notes.clone(), notes]);
    let report = chain.deliver(delivery);
    let result = report.method_result.ok_or("no method result")?;
    assert_eq!(
        result.method_outcome(),
        Some(MethodOutcome::InsertionFailed)
    );
    assert_eq!(report.status, InsertionStatus::Failed);
    assert!(chain.keys.state().chords.is_empty());
    assert_eq!(chain.script().len(), 0, "both Notes reads came first");
    let orchestrator = &chain.orchestrator;
    assert_eq!(
        disposition(orchestrator),
        Some(RecoveryDisposition::Recover)
    );
    chain.stop()
}
