//! Deliberate Copy of the recovery bubble (010 AC-005) through 007's real coordinator and
//! actor with 007's scripted pasteboard: Copy writes unmarked plain text and advances the
//! `changeCount`, even for the identical transcript, so a pending read-confirmed restoration
//! abandons through its own guard (before the quiet period, during repeated reads, and after a
//! "not read" outcome, which never restores); through the actor, Copy waits behind an
//! insertion in progress and still wins. 007's own timer boundaries stay in 007's suite.
// The shared 007 fixtures carry helpers this crate does not use.
#![allow(dead_code)]

mod smart_paste {
    pub mod fake_pasteboard;
    pub mod fakes;
    pub mod rig;
}

use handy_app_lib::smart_paste::actor::SmartPasteActor;
use handy_app_lib::smart_paste::clock::{Clock, ManualClock, SystemClock};
use handy_app_lib::smart_paste::coordinator::SmartPastePorts;
use handy_app_lib::smart_paste::ids::Transcript;
use handy_app_lib::smart_paste::keyboard::InputSourceKind;
use handy_app_lib::smart_paste::outcome::PasteOutcomeKind;
use handy_app_lib::smart_paste::pasteboard_port::{ReadEvent, ReadSink};
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::DontModify;
use smart_paste::fakes::{FakeAx, FakeInputSources, ScriptedField};
use smart_paste::rig::{
    request, rich_items, settle_until, Content, FakeDestination, FakeDiagnostics, FakeKeys,
    FakePasteboard, Rig, ScriptedReads,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const TEXTEDIT: Option<&str> = Some("com.apple.TextEdit");
const LONG_WAIT: Duration = Duration::from_secs(10);

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}
fn key_down(rig: &Rig) -> Instant {
    let chords = rig.keys.state().chords.clone();
    chords.last().copied().unwrap_or_else(|| rig.clock.now())
}
fn restored(rig: &Rig) -> bool {
    rig.pasteboard.state().writes.contains(&"restore")
}

// @spec AC-005: Deliberate Copy dominates restoration — .specs/features/010-recovery-bubble/spec.md#ac-005
#[test]
fn copy_of_the_identical_transcript_before_the_quiet_period_wins_over_restoration() -> TestResult {
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[20]);
    let outcome = coordinator.insert(&request(1, "hello world", TEXTEDIT, DontModify), &mut reads);
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    let count = rig.pasteboard.state().change_count;
    coordinator.deliberate_copy(&Transcript::new("hello world"))?;
    assert_eq!(rig.pasteboard.state().change_count, count + 1);
    settle_until(
        &rig.clock,
        &mut coordinator,
        key_down(&rig) + Duration::from_secs(9),
    );
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
    assert!(
        !restored(&rig),
        "restoration abandoned through its changeCount guard"
    );
    Ok(())
}
#[test]
fn without_copy_the_same_scenario_restores_the_previous_clipboard() {
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(&request(1, "hello world", TEXTEDIT, DontModify), &mut reads);
    settle_until(
        &rig.clock,
        &mut coordinator,
        key_down(&rig) + Duration::from_secs(9),
    );
    assert!(
        restored(&rig),
        "control: the restoration runs when nothing wrote"
    );
    assert_eq!(rig.pasteboard.state().content, Content::Items(rich_items()));
}
#[test]
fn copy_during_repeated_reads_of_an_earlier_dictation_wins_over_the_quiet_and_the_cap() -> TestResult
{
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[20]);
    coordinator.insert(&request(1, "dictation A", TEXTEDIT, DontModify), &mut reads);
    let transaction = rig.pasteboard.published().ok_or("published")?;
    for offset in [150, 300] {
        let at = key_down(&rig) + ms(offset);
        settle_until(&rig.clock, &mut coordinator, at);
        coordinator.handle_read(ReadEvent { transaction, at });
    }
    // Dictation B's bubble: Copy before the quiet deadline and the 8 s cap.
    coordinator.deliberate_copy(&Transcript::new("dictation B kept by the bubble"))?;
    settle_until(
        &rig.clock,
        &mut coordinator,
        key_down(&rig) + Duration::from_secs(9),
    );
    let content = rig.pasteboard.state().content.clone();
    assert_eq!(
        content,
        Content::Plain("dictation B kept by the bubble".into())
    );
    assert!(!restored(&rig));
    Ok(())
}
#[test]
fn copy_after_a_not_read_outcome_stays_and_nothing_is_ever_restored() -> TestResult {
    let rig = Rig::new();
    let mut coordinator = rig.coordinator();
    let mut reads = ScriptedReads::new(&rig, &[]);
    let outcome = coordinator.insert(&request(1, "hello world", TEXTEDIT, DontModify), &mut reads);
    assert_eq!(outcome.kind, PasteOutcomeKind::NotRead);
    coordinator.deliberate_copy(&Transcript::new("hello world"))?;
    settle_until(
        &rig.clock,
        &mut coordinator,
        key_down(&rig) + Duration::from_secs(9),
    );
    assert_eq!(
        rig.pasteboard.state().content,
        Content::Plain("hello world".into())
    );
    assert!(!restored(&rig));
    Ok(())
}

/// The real actor; the fake target reads the published item 20 ms after the chord.
fn spawn_actor(pasteboard: &FakePasteboard) -> std::io::Result<SmartPasteActor> {
    let pasteboard = pasteboard.clone();
    SmartPasteActor::spawn(move |sink: ReadSink| {
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let manual = ManualClock::new();
        let keys = FakeKeys::new(Arc::clone(&clock));
        let target = pasteboard.clone();
        keys.state().on_chord = Some(Arc::new(move || {
            let (Some(transaction), sink) = (target.published(), Arc::clone(&sink)) else {
                return;
            };
            std::thread::spawn(move || {
                std::thread::sleep(ms(20));
                sink(ReadEvent {
                    transaction,
                    at: Instant::now(),
                });
            });
        }));
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
fn through_the_actor_copy_waits_behind_the_insertion_in_progress_and_still_wins() -> TestResult {
    let pasteboard = FakePasteboard::new(&ManualClock::new(), rich_items());
    let mut actor = spawn_actor(&pasteboard)?;
    let started = Instant::now();
    let inserted = actor
        .insert(request(1, "dictation A", None, DontModify))
        .map_err(|_| "actor stopped")?;
    let copied = actor
        .deliberate_copy(Transcript::new("dictation B kept by the bubble"))
        .map_err(|_| "actor stopped")?;
    let outcome = inserted.recv_timeout(LONG_WAIT)?;
    copied.recv_timeout(LONG_WAIT)??;
    println!("copy answered {:?} after the request", started.elapsed());
    assert_eq!(outcome.kind, PasteOutcomeKind::ReadObserved);
    std::thread::sleep(ms(700));
    let content = pasteboard.state().content.clone();
    assert_eq!(
        content,
        Content::Plain("dictation B kept by the bubble".into())
    );
    assert!(!pasteboard.state().writes.contains(&"restore"));
    assert!(actor.shutdown(LONG_WAIT).is_ok());
    Ok(())
}
