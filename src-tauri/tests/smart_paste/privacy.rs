//! No transcript, clipboard content or field value reaches any log record
//! (QE privacy gate), whatever the insertion path.

use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::Duration;

use handy_app_lib::smart_paste::ax_port::Utf16Range;
use handy_app_lib::smart_paste::settlement::ClipboardPolicy::{CopyToClipboard, DontModify};
use handy_app_lib::smart_paste::strategy::SCREEN_SHARING_BUNDLE_ID;

use super::fakes::{ScriptedField, WriteBehavior};
use super::rig::{request, settle_until, Content, Rig, ScriptedReads};

const SENTINEL: &str = "SENTINEL-transcript-7f3a";
const FIELD_SENTINEL: &str = "SENTINEL-field-91c2";
const CLIPBOARD_SENTINEL: &str = "SENTINEL-clipboard-55d0";

struct CapturingLogger;

/// Every record captured by this process's logger (shared by the tests).
pub fn records() -> &'static Mutex<Vec<String>> {
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

/// Installs the capturing logger once for the whole test process.
pub fn install_logger() {
    // Another test may have installed it first; one capturing logger is enough.
    if log::set_boxed_logger(Box::new(CapturingLogger)).is_ok() {
        log::set_max_level(log::LevelFilter::Trace);
    }
}

fn run_every_path() {
    let rig = Rig::new();
    rig.pasteboard
        .external_write(Content::Plain(CLIPBOARD_SENTINEL.into()));
    let mut coordinator = rig.coordinator();
    for (id, offsets) in [(1, vec![20]), (2, vec![])] {
        let mut reads = ScriptedReads::new(&rig, &offsets);
        coordinator.insert(&request(id, SENTINEL, None, DontModify), &mut reads);
        settle_until(
            &rig.clock,
            &mut coordinator,
            rig.clock.at(Duration::from_secs(60 * id)),
        );
    }
    coordinator.shutdown();
    let mut direct = Rig::new();
    let field = ScriptedField::new(&direct.clock, 7, FIELD_SENTINEL, Utf16Range::default());
    field.state().behavior = WriteBehavior::ApplyPartially;
    direct.ax.field = Some(field);
    let mut reads = ScriptedReads::new(&direct, &[]);
    let mut coordinator = direct.coordinator();
    coordinator.insert(&request(3, SENTINEL, None, CopyToClipboard), &mut reads);
    coordinator.insert(
        &request(4, SENTINEL, Some(SCREEN_SHARING_BUNDLE_ID), DontModify),
        &mut reads,
    );
}

#[test]
fn skipped_types_are_logged_by_name_without_their_content() {
    install_logger();
    let rig = Rig::new();
    let promise = "com.example.unreadable-promise";
    let content = CLIPBOARD_SENTINEL.as_bytes().to_vec();
    let items = vec![vec![
        (promise.into(), content.clone()),
        ("public.rtf".into(), content),
    ]];
    rig.pasteboard.state().content = Content::Items(items);
    rig.pasteboard.state().unreadable_types = vec![promise.into()];
    let mut reads = ScriptedReads::new(&rig, &[20]);
    let request = request(9, SENTINEL, None, DontModify);
    rig.coordinator().insert(&request, &mut reads);
    let captured = records().lock().map(|lines| lines.clone());
    let captured = captured.unwrap_or_else(|poisoned| poisoned.into_inner().clone());
    let is_named = |line: &String| line.contains(promise) && line.contains("Unreadable");
    assert!(captured.iter().any(is_named));
    assert!(captured
        .iter()
        .all(|line| !line.contains(CLIPBOARD_SENTINEL)));
}

#[test]
fn no_log_record_contains_user_content() {
    install_logger();
    run_every_path();
    let captured = records()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    assert!(captured.iter().any(|line| line.contains("[smart-paste]")));
    for sentinel in [SENTINEL, FIELD_SENTINEL, CLIPBOARD_SENTINEL] {
        assert!(
            captured.iter().all(|line| !line.contains(sentinel)),
            "{sentinel} logged"
        );
    }
}
