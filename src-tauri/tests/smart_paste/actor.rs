//! The actor thread with real time: serialized insertions, idle timers and
//! exit (AC-009, AC-012). Reads are served by a fake target through the
//! production read sink, as AppKit would on the main thread.

use std::sync::Arc;
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::actor::SmartPasteActor;
use handy_app_lib::smart_paste::clock::{Clock, ManualClock, SystemClock};
use handy_app_lib::smart_paste::coordinator::SmartPastePorts;
use handy_app_lib::smart_paste::keyboard::InputSourceKind;
use handy_app_lib::smart_paste::outcome::PasteOutcomeKind;
use handy_app_lib::smart_paste::pasteboard_port::{ReadEvent, ReadSink};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::DontModify;

use super::fakes::{FakeAx, FakeInputSources, ScriptedField};
use super::rig::{
    request, rich_items, Content, FakeDestination, FakeDiagnostics, FakeKeys, FakePasteboard,
};

// @spec FR-008: Serialized terminal outcomes — .specs/features/007-smart-paste/spec.md#fr-008

/// The fake target reads the clipboard this long after the chord.
const TARGET_READ_DELAY: Duration = Duration::from_millis(20);
const OUTCOME_TIMEOUT: Duration = Duration::from_secs(5);

fn reading_target(pasteboard: &FakePasteboard, sink: ReadSink) -> Arc<dyn Fn() + Send + Sync> {
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

fn spawn_actor(pasteboard: &FakePasteboard) -> std::io::Result<SmartPasteActor> {
    let pasteboard = pasteboard.clone();
    SmartPasteActor::spawn(move |sink| {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let manual = ManualClock::new();
        let keys = FakeKeys::new(Arc::clone(&clock));
        keys.state().on_chord = Some(reading_target(&pasteboard, sink));
        let mut ax = FakeAx::with_field(&ScriptedField::new(&manual, 7, "", Default::default()));
        ax.field = None;
        SmartPastePorts {
            clock,
            ax: Box::new(ax),
            pasteboard: Box::new(pasteboard),
            input_sources: Box::new(FakeInputSources::new(
                &manual,
                "com.apple.keylayout.US",
                InputSourceKind::KeyboardLayout,
                &[],
            )),
            keys: Box::new(keys),
            destination: Box::new(FakeDestination::default()),
            diagnostics: Box::new(FakeDiagnostics::default()),
        }
    })
}

#[test]
fn actor_reports_the_read_and_restores_after_the_quiet_period(
) -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard)?;
    let outcome = actor
        .insert(request(1, "hello world", None, DontModify))
        .map_err(|_| "actor stopped")?
        .recv_timeout(OUTCOME_TIMEOUT)?;
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert!(matches!(pasteboard.state().content, Content::Lazy(_)));
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(pasteboard.state().content, Content::Items(rich_items()));
    assert!(actor.shutdown(OUTCOME_TIMEOUT));
    Ok(())
}

#[test]
fn queued_insertions_are_serialized_without_intermediate_restoration(
) -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard)?;
    let first = actor
        .insert(request(1, "first", None, DontModify))
        .map_err(|_| "actor stopped")?;
    let second = actor
        .insert(request(2, "second", None, DontModify))
        .map_err(|_| "actor stopped")?;
    assert_eq!(
        first.recv_timeout(OUTCOME_TIMEOUT)?.kind,
        PasteOutcomeKind::ReadObserved
    );
    assert_eq!(
        second.recv_timeout(OUTCOME_TIMEOUT)?.kind,
        PasteOutcomeKind::ReadObserved
    );
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(
        pasteboard.state().writes,
        vec!["publish", "publish", "restore"]
    );
    assert_eq!(pasteboard.state().content, Content::Items(rich_items()));
    assert!(actor.shutdown(OUTCOME_TIMEOUT));
    Ok(())
}

#[test]
fn shutdown_before_the_quiet_period_keeps_the_transcript() -> Result<(), Box<dyn std::error::Error>>
{
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard)?;
    actor
        .insert(request(1, "hello world", None, DontModify))
        .map_err(|_| "actor stopped")?
        .recv_timeout(OUTCOME_TIMEOUT)?;
    assert!(actor.shutdown(OUTCOME_TIMEOUT));
    assert_eq!(
        pasteboard.state().content,
        Content::Plain("hello world".into())
    );
    assert!(actor.insert(request(2, "late", None, DontModify)).is_err());
    Ok(())
}
