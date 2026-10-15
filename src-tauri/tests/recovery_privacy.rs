//! Privacy of the recovery bubble at run time (010 QE P0, delivery review): a dictation with
//! a sentinel transcript goes through every bubble path of the service — consume through 006's
//! registry, first render, late read, failed and successful Copy, refused drag and accepted
//! drop, Close, a hung surface and its fallback toast, and a pre-insertion classification —
//! with a capturing logger at trace level and the recording surface (the Tauri adapters are
//! covered by the native checklist). The sentinel never reaches a log record, the outcome event or the toast; it only
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
use handy_app_lib::recovery::types::{RecoveryBubbleView, RecoveryPlatform};
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

/// The sentinel phrase of every dictation.
fn sentinel_text() -> String {
    format!("{SENTINEL} said in a recovery")
}
/// Publish a failed dictation through 006's registry; returns its rendered view.
fn shown(
    bench: &Bench,
    registry: &OutcomeRegistry,
    dictation: u64,
) -> Result<RecoveryBubbleView, Box<dyn std::error::Error>> {
    let outcome = bench.failed_with(dictation, &sentinel_text(), Some(MethodOutcome::NotRead));
    assert!(
        !serde_json::to_string(&outcome)?.contains(SENTINEL),
        "insertion-outcome event"
    );
    let _receipt = registry.publish(&outcome);
    let view = bench.surface.presented().pop().ok_or("presented")?;
    bench.service.rendered(view.bubble_id);
    Ok(view)
}
/// Late read, failed Copy, refused drag, then Close of the bubble of dictation 1.
fn refused_paths(bench: &Bench, view: &RecoveryBubbleView) {
    bench.service.late_read(DictationId(1), 7);
    *lock(&bench.copier.answer) = Err(CopyError::Pasteboard);
    let _ = bench
        .service
        .copy(view.bubble_id, view.destination_generation);
    *lock(&bench.copier.answer) = Ok(());
    bench.service.begin_drag(view.bubble_id);
    bench.service.drag_ended(view.bubble_id, false);
    bench.service.close(view.bubble_id);
}
/// A bubble whose surface never answers: destroyed at the hang limit, toast fallback.
fn hung_path(bench: &Bench, dictation: u64) -> Result<(), Box<dyn std::error::Error>> {
    let hung = bench.failed_with(dictation, &sentinel_text(), None);
    drop(bench.service.consume(&hung));
    bench.timers.fire_all();
    let view = bench.surface.presented().pop().ok_or("hung")?;
    bench.service.window_destroyed(view.bubble_id);
    Ok(())
}
/// A pre-insertion classification of a new dictation (closes nothing left).
fn classification_path(bench: &Bench, dictation: u64) {
    let context = DestinationContext {
        dictation_id: DictationId(dictation),
        destination_generation: dictation,
        source: DestinationSource::Captured,
        target: Some(ResolvedTarget {
            pid: 42,
            identity: ApplicationIdentity::BundleId("com.apple.Notes".to_owned()),
        }),
    };
    bench.service.dictation_starting(true);
    bench.service.dictation_started(DictationId(dictation));
    let _ = bench.service.before_insertion(&context);
}
/// Every bubble path, one dictation each: refused actions and Close, a successful Copy, an
/// accepted drop, a hung surface, then a classification.
fn run_every_path(bench: &Bench) -> Result<(), Box<dyn std::error::Error>> {
    let registry = OutcomeRegistry::default();
    registry.register(bench.service.clone());
    refused_paths(bench, &shown(bench, &registry, 1)?);
    let copied = shown(bench, &registry, 2)?;
    bench
        .service
        .copy(copied.bubble_id, copied.destination_generation)
        .map_err(|error| format!("{error:?}"))?;
    let dropped = shown(bench, &registry, 3)?;
    bench.service.begin_drag(dropped.bubble_id);
    bench.service.drag_ended(dropped.bubble_id, true);
    hung_path(bench, 4)?;
    classification_path(bench, 5);
    Ok(())
}
/// The surface calls carry the sentinel only in the projection and the drag payload.
fn assert_surface_calls(bench: &Bench) {
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
        logs.iter().any(|line| line.contains("drop accepted")),
        "the paths logged, the accepted drop included"
    );
    assert!(logs.iter().all(|line| !line.contains(SENTINEL)), "{logs:?}");
    assert_surface_calls(&bench);
    let written = lock(&bench.copier.written).clone();
    assert!(!written.is_empty() && written.iter().all(|copied| copied.contains(SENTINEL)));
    assert_eq!(
        bench.toasts().len(),
        1,
        "the hung bubble fell back to the toast, text-free"
    );
    Ok(())
}
