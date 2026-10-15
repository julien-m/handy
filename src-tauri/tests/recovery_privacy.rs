//! Privacy of the recovery bubble at run time (010 QE P0, delivery review): a dictation with
//! a sentinel transcript goes through every bubble path — consume, first render, late read,
//! failed and successful Copy, refused drag, Close, a hung surface and its fallback toast, and
//! a pre-insertion classification — with a capturing logger at trace level and the recording
//! surface. The sentinel never reaches a log record, the outcome event or the toast; it only
//! appears in the projection sent to the recovery window and in the Copy and drag payloads.
#[path = "recovery/support.rs"]
mod support;

use handy_app_lib::destination::context::{
    ApplicationIdentity, DestinationContext, DestinationSource, DictationId, RecoveryPort,
    ResolvedTarget,
};
use handy_app_lib::destination::outcome::MethodOutcome;
use handy_app_lib::destination::recovery::{OutcomeConsumer, OutcomeRegistry};
use handy_app_lib::recovery::ports::CopyError;
use handy_app_lib::recovery::types::RecoveryPlatform;
use std::sync::{Mutex, OnceLock, PoisonError};
use support::{lock, Bench, SurfaceCall, TestResult};

const SENTINEL: &str = "SENTINEL-010-transcript-4b2e";

struct CapturingLogger;
fn records() -> &'static Mutex<Vec<String>> {
    static RECORDS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    RECORDS.get_or_init(|| Mutex::new(Vec::new()))
}
impl log::Log for CapturingLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let line = format!("{} {}", record.target(), record.args());
        records()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(line);
    }
    fn flush(&self) {}
}

/// Every bubble path of one dictation (and a hung second one), with the logger installed.
fn run_every_path(bench: &Bench) -> Result<(), Box<dyn std::error::Error>> {
    let text = format!("{SENTINEL} said in a recovery");
    let registry = OutcomeRegistry::default();
    registry.register(bench.service.clone());
    let outcome = bench.failed_with(1, &text, Some(MethodOutcome::NotRead));
    assert!(
        !serde_json::to_string(&outcome)?.contains(SENTINEL),
        "insertion-outcome event"
    );
    let _receipt = registry.publish(&outcome);
    let view = bench.surface.presented().pop().ok_or("presented")?;
    bench.service.rendered(view.bubble_id);
    bench.service.late_read(DictationId(1), 7);
    *lock(&bench.copier.answer) = Err(CopyError::Pasteboard);
    let _ = bench
        .service
        .copy(view.bubble_id, view.destination_generation);
    bench.service.begin_drag(view.bubble_id);
    bench.service.drag_ended(view.bubble_id, false);
    bench.service.close(view.bubble_id);
    let hung = bench.failed_with(2, &text, None);
    drop(bench.service.consume(&hung));
    bench.timers.fire_all();
    let second = bench.surface.presented().pop().ok_or("second")?;
    bench.service.window_destroyed(second.bubble_id);
    let context = DestinationContext {
        dictation_id: DictationId(3),
        destination_generation: 3,
        source: DestinationSource::Captured,
        target: Some(ResolvedTarget {
            pid: 42,
            identity: ApplicationIdentity::BundleId("com.apple.Notes".to_owned()),
        }),
    };
    bench.service.dictation_starting(true);
    bench.service.dictation_started(DictationId(3));
    let _ = bench.service.before_insertion(&context);
    Ok(())
}

// @spec AC-003: The transcript reaches only the bubble window and its payloads — .specs/features/010-recovery-bubble/spec.md#ac-003
#[test]
fn the_sentinel_reaches_only_the_bubble_window_and_the_copy_and_drag_payloads() -> TestResult {
    if log::set_boxed_logger(Box::new(CapturingLogger)).is_ok() {
        log::set_max_level(log::LevelFilter::Trace);
    }
    let bench = Bench::new(RecoveryPlatform::MacOs);
    run_every_path(&bench)?;
    let logs = records()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    assert!(
        logs.iter().any(|line| line.contains("Recovery")),
        "the paths logged"
    );
    assert!(logs.iter().all(|line| !line.contains(SENTINEL)), "{logs:?}");
    for call in bench.surface.calls() {
        match call {
            SurfaceCall::Present(view)
            | SurfaceCall::Update(view)
            | SurfaceCall::ShowAgain(view) => {
                assert!(
                    view.preview.contains(SENTINEL),
                    "the bubble window shows it"
                );
            }
            SurfaceCall::BeginDrag(_, payload) => assert!(payload.contains(SENTINEL)),
            SurfaceCall::Hide | SurfaceCall::Destroy(_) => {}
        }
    }
    assert!(lock(&bench.copier.written)
        .iter()
        .all(|copied| copied.contains(SENTINEL)));
    assert_eq!(
        bench.toasts().len(),
        1,
        "the hung bubble fell back to the toast, text-free"
    );
    Ok(())
}
