//! The actor thread with real time: serialized insertions, idle timers and
//! exit (AC-009, AC-012). Reads are served by a fake target through the
//! production read sink, as AppKit would on the main thread. Assertions wait
//! for conditions with generous timeouts, so a loaded machine only slows
//! the tests down.

use std::sync::Arc;
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::actor::{shutdown_allowance, SmartPasteActor};
use handy_app_lib::smart_paste::clock::{Clock, ManualClock, SystemClock};
use handy_app_lib::smart_paste::coordinator::SmartPastePorts;
use handy_app_lib::smart_paste::keyboard::InputSourceKind;
use handy_app_lib::smart_paste::outcome::PasteOutcomeKind;
use handy_app_lib::smart_paste::pasteboard_port::{ReadEvent, ReadSink};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::DontModify;
use handy_app_lib::smart_paste::timing::JUMP_DESKTOP_BUNDLE_ID;

use super::fakes::{FakeAx, FakeInputSources, ScriptedField};
use super::rig::{
    request, rich_items, Content, FakeDestination, FakeDiagnostics, FakeKeys, FakePasteboard,
};

// @spec FR-008: Serialized terminal outcomes — .specs/features/007-smart-paste/spec.md#fr-008

/// The fake target reads the clipboard this long after the chord.
const TARGET_READ_DELAY: Duration = Duration::from_millis(20);
/// A target that keeps reading: every 50 ms for 5 s, so quiet never
/// elapses while the test runs, however slow the machine.
const REPEATED_READS: u32 = 100;
const REPEATED_READ_INTERVAL: Duration = Duration::from_millis(50);
const OUTCOME_TIMEOUT: Duration = Duration::from_secs(10);

type Target = fn(&FakePasteboard, ReadSink) -> Arc<dyn Fn() + Send + Sync>;

/// Reads `count` times, `interval` apart, after the chord.
fn reads_after_chord(
    pasteboard: &FakePasteboard,
    sink: ReadSink,
    count: u32,
    interval: Duration,
) -> Arc<dyn Fn() + Send + Sync> {
    let pasteboard = pasteboard.clone();
    Arc::new(move || {
        let Some(transaction) = pasteboard.published() else {
            return;
        };
        let sink = Arc::clone(&sink);
        std::thread::spawn(move || {
            std::thread::sleep(TARGET_READ_DELAY);
            for _ in 0..count {
                let at = Instant::now();
                sink(ReadEvent { transaction, at });
                std::thread::sleep(interval);
            }
        });
    })
}

fn reading_once(pasteboard: &FakePasteboard, sink: ReadSink) -> Arc<dyn Fn() + Send + Sync> {
    reads_after_chord(pasteboard, sink, 1, Duration::ZERO)
}

fn never_reading(_: &FakePasteboard, _: ReadSink) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(|| {})
}

fn reading_repeatedly(pasteboard: &FakePasteboard, sink: ReadSink) -> Arc<dyn Fn() + Send + Sync> {
    reads_after_chord(pasteboard, sink, REPEATED_READS, REPEATED_READ_INTERVAL)
}

fn spawn_actor(pasteboard: &FakePasteboard, target: Target) -> std::io::Result<SmartPasteActor> {
    let pasteboard = pasteboard.clone();
    SmartPasteActor::spawn(move |sink| {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let manual = ManualClock::new();
        let keys = FakeKeys::new(Arc::clone(&clock));
        keys.state().on_chord = Some(target(&pasteboard, sink));
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

/// Waits until `condition` holds, or the timeout.
fn wait_until(condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + OUTCOME_TIMEOUT;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn wait_for_content(pasteboard: &FakePasteboard, content: &Content) -> bool {
    wait_until(|| pasteboard.state().content == *content)
}

fn shutdown(actor: &mut SmartPasteActor) -> bool {
    actor.shutdown(OUTCOME_TIMEOUT).is_ok()
}

#[test]
fn actor_reports_the_read_and_restores_after_the_quiet_period(
) -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard, reading_once)?;
    let outcome = actor
        .insert(request(1, "hello world", None, DontModify))
        .map_err(|_| "actor stopped")?
        .recv_timeout(OUTCOME_TIMEOUT)?;
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    assert!(wait_for_content(&pasteboard, &Content::Items(rich_items())));
    assert!(shutdown(&mut actor));
    Ok(())
}

#[test]
fn queued_insertions_are_serialized_without_intermediate_restoration(
) -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard, reading_once)?;
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
    assert!(wait_for_content(&pasteboard, &Content::Items(rich_items())));
    assert_eq!(
        pasteboard.state().writes,
        vec!["publish", "publish", "restore"]
    );
    assert!(shutdown(&mut actor));
    Ok(())
}

#[test]
fn shutdown_before_the_quiet_period_keeps_the_transcript() -> Result<(), Box<dyn std::error::Error>>
{
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard, reading_repeatedly)?;
    actor
        .insert(request(1, "hello world", None, DontModify))
        .map_err(|_| "actor stopped")?
        .recv_timeout(OUTCOME_TIMEOUT)?;
    assert!(shutdown(&mut actor));
    assert_eq!(
        pasteboard.state().content,
        Content::Private("hello world".into())
    );
    assert!(actor.insert(request(2, "late", None, DontModify)).is_err());
    Ok(())
}

#[test]
fn shutdown_during_an_insertion_lets_it_finish_and_drops_queued_ones(
) -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard, never_reading)?;
    let jump = Some(JUMP_DESKTOP_BUNDLE_ID);
    let running = actor
        .insert(request(1, "first", jump, DontModify))
        .map_err(|_| "actor stopped")?;
    // The first insertion is waiting for its read once its item is published.
    assert!(wait_until(|| pasteboard.published().is_some()));
    let queued = actor
        .insert(request(2, "second", jump, DontModify))
        .map_err(|_| "actor stopped")?;
    let started = Instant::now();
    let is_stopped = actor.shutdown(shutdown_allowance(Duration::from_millis(60)));
    assert!(is_stopped.is_ok());
    assert!(started.elapsed() < shutdown_allowance(Duration::from_millis(60)));
    assert_eq!(
        running.recv_timeout(OUTCOME_TIMEOUT)?.kind,
        PasteOutcomeKind::NotRead
    );
    assert!(queued.recv_timeout(OUTCOME_TIMEOUT).is_err());
    assert_eq!(pasteboard.state().writes, vec!["publish", "materialize"]);
    Ok(())
}

#[test]
fn dropping_the_actor_without_shutdown_still_settles() -> Result<(), Box<dyn std::error::Error>> {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let actor = spawn_actor(&pasteboard, reading_repeatedly)?;
    actor
        .insert(request(1, "hello world", None, DontModify))
        .map_err(|_| "actor stopped")?
        .recv_timeout(OUTCOME_TIMEOUT)?;
    drop(actor);
    let transcript = Content::Private("hello world".into());
    assert!(wait_for_content(&pasteboard, &transcript));
    Ok(())
}
