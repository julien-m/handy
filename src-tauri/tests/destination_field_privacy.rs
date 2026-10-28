//! Privacy of 011 at run time (spec AC-008, SC-004): a remembered selection with distinctive
//! sentinel position and length goes through the start read, a verified return, a diverted one,
//! an unchanged one and a release by a badge change, with a capturing logger at trace level.
//! Neither sentinel value reaches a log record, the context's serialization or the outcome's
//! reason; the log records the stages and times of `field_return` only. The port has no way to
//! read an element's value, selected text or title, so no content can be read at all.
#[path = "pin_enter/support.rs"]
mod destination;
#[path = "destination_field/fake.rs"]
mod fake;
#[path = "destination_field/rig.rs"]
mod rig;

use destination::notes;
use fake::{Element, Write};
use handy_app_lib::destination::outcome::DestinationFailure;
use rig::{FieldRig, PasteSpy, INPUT, OTHER, WINDOW};
use std::sync::{Mutex, OnceLock, PoisonError};

/// Distinctive selection position and length (spec SC-004).
const SENTINEL_LOCATION: usize = 987_654;
const SENTINEL_LENGTH: usize = 123_457;
const PID: u32 = 22;

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
        let line = format!("{} {} {}", record.level(), record.target(), record.args());
        records()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(line);
    }
    fn flush(&self) {}
}

fn sentinel_rig() -> FieldRig {
    let field = FieldRig::comet();
    let range = (SENTINEL_LOCATION, SENTINEL_LENGTH);
    field
        .world
        .element(INPUT, Element::input(PID, WINDOW, range));
    field
}
/// One dictation with the sentinel range, `script` applied before the insertion.
fn dictate(script: impl FnOnce(&FieldRig)) -> Option<DestinationFailure> {
    let field = sentinel_rig();
    let started = field.start(true);
    field.rig.workspace.bring(notes());
    script(&field);
    let spy = PasteSpy::new(&field.world);
    let report = field.deliver_plain(&started, &spy);
    for context in spy.contexts() {
        let json = serde_json::to_string(&context).unwrap_or_default();
        assert!(!json.contains(&SENTINEL_LOCATION.to_string()), "{json}");
    }
    report.reason
}

/// A verified return, a diverted one, an unchanged one, then a release by a cancellation.
fn run_every_path() {
    assert_eq!(dictate(|_| {}), None, "verified");
    let refused = dictate(|field| {
        field
            .world
            .edit(INPUT, |input| input.select = Write::Refuse)
    });
    assert_eq!(refused, Some(DestinationFailure::FieldReturnFailed));
    dictate(|field| {
        field.world.edit(INPUT, |input| input.focus = Write::Refuse);
        field.world.focus(PID, Ok(OTHER));
    });
    let field = sentinel_rig();
    field.start(true);
    field.rig.manager.cancel();
}

// @spec AC-008: No content or range in logs or events — .specs/features/011-dictation-field-return/spec.md#ac-008
// @spec SC-004: Sentinel range absent from logs and events — .specs/features/011-dictation-field-return/spec.md#sc-004
#[test]
fn the_sentinel_range_never_reaches_a_log_or_an_event() {
    let _ = log::set_logger(&CapturingLogger);
    log::set_max_level(log::LevelFilter::Trace);
    run_every_path();
    let lines = records()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let field_lines: Vec<_> = lines
        .iter()
        .filter(|line| line.contains("field_return"))
        .collect();
    assert!(field_lines.len() >= 4, "stages are logged: {field_lines:?}");
    for line in &lines {
        assert!(!line.contains(&SENTINEL_LOCATION.to_string()), "{line}");
        assert!(!line.contains(&SENTINEL_LENGTH.to_string()), "{line}");
    }
    let reason = serde_json::to_string(&DestinationFailure::FieldReturnFailed).unwrap_or_default();
    assert_eq!(
        reason, "\"field_return_failed\"",
        "the outcome carries a reason only"
    );
}
