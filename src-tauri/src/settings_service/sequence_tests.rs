//! P0 tests of the ordered write sequence against real stores and a spy native port: effects run
//! before the UI event and both before the result; a failed effect after the commit is reported
//! as committed and retried before the next write; preparation is compensated on refusal and kept
//! when an unknown outcome landed; concurrent writes and repairs leave the native state equal to
//! the disk and lose no update.
use super::{
    commit_prepared, ordered, repair, write_edit, write_prepared, write_value, LiveEffectsPort,
    PreparedChange,
};
use crate::settings::AppSettings;
use crate::settings_service::{
    edit::SettingsEdit,
    errors::{Outcome, SettingsError, SettingsErrorCode},
    live::{LiveCommit, LiveSettings},
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::SettingsRepository,
    transaction::{PersistObserver, PersistStage},
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Records effects and events in order and mirrors applied values as the "native" state.
#[derive(Default)]
struct SpyPort {
    events: Mutex<Vec<String>>,
    failing: Mutex<Vec<&'static str>>,
    native: Mutex<BTreeMap<&'static str, Value>>,
}
impl SpyPort {
    fn events(&self) -> Vec<String> {
        self.events.lock().map(|e| e.clone()).unwrap_or_default()
    }
    fn native(&self, key: &str) -> Option<Value> {
        self.native.lock().ok()?.get(key).cloned()
    }
    fn set_native(&self, key: &'static str, value: Value) {
        if let Ok(mut native) = self.native.lock() {
            native.insert(key, value);
        }
    }
    fn fail(&self, keys: &[&'static str]) {
        if let Ok(mut failing) = self.failing.lock() {
            *failing = keys.to_vec();
        }
    }
    fn record(&self, event: String) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}
impl LiveEffectsPort for SpyPort {
    fn apply(&self, commit: &LiveCommit, skip: &[&str]) -> Result<(), Vec<&'static str>> {
        let after = serde_json::to_value(&commit.after).unwrap_or_default();
        let failing = self.failing.lock().map(|f| f.clone()).unwrap_or_default();
        let mut failed = Vec::new();
        for key in commit
            .changed
            .iter()
            .copied()
            .filter(|key| !skip.contains(key))
        {
            self.record(format!("effect:{key}"));
            if failing.contains(&key) {
                failed.push(key);
            } else {
                self.set_native(key, after[key].clone());
            }
        }
        if failed.is_empty() {
            Ok(())
        } else {
            Err(failed)
        }
    }
    fn notify(&self, revision: Revision, keys: &[&'static str]) {
        self.record(format!("notify:{}:{}", revision.0, keys.join(",")));
    }
}
fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn owned(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    LiveSettings::start_owned(
        SettingsRepository::new(store(directory)),
        &LegacyPathProbe::new(directory.path().join("absent.sock")),
    )
}
fn stored(directory: &tempfile::TempDir, key: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let read = SettingsRepository::new(store(directory)).read_snapshot()?;
    Ok(serde_json::to_value(&read.snapshot.settings)?[key].clone())
}

#[test]
fn live_effects_run_before_the_ui_event_and_both_before_the_result() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    let commit = write_value(&live, &port, "always_on_microphone", json!(true))?;
    port.record("returned".into());
    let expected = vec![
        "effect:always_on_microphone".to_string(),
        format!("notify:{}:always_on_microphone", commit.revision.0),
        "returned".to_string(),
    ];
    assert_eq!(port.events(), expected);
    Ok(())
}
#[test]
fn failed_effect_after_commit_is_committed_with_revision_field_and_event() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    port.fail(&["always_on_microphone"]);
    let error = write_value(&live, &port, "always_on_microphone", json!(true))
        .err()
        .ok_or("a failed live effect was reported as a success")?;
    assert_eq!(error.code(), SettingsErrorCode::InternalError);
    assert_eq!(error.outcome(), Some(Outcome::Committed));
    assert_eq!(error.field(), Some("always_on_microphone"));
    assert_eq!(error.current_revision(), Some(live.snapshot().revision));
    assert_eq!(stored(&directory, "always_on_microphone")?, json!(true));
    let notified = format!("notify:{}:always_on_microphone", live.snapshot().revision.0);
    assert!(port.events().contains(&notified));
    assert_eq!(
        live.sequence().unapplied_keys(),
        vec!["always_on_microphone"]
    );
    Ok(())
}
#[test]
fn unapplied_effect_is_retried_before_the_next_write_and_on_resubmission() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    port.fail(&["always_on_microphone"]);
    let _ = write_value(&live, &port, "always_on_microphone", json!(true));
    let _ = write_value(&live, &port, "always_on_microphone", json!(true));
    assert_eq!(port.native("always_on_microphone"), None, "still failing");
    port.fail(&[]);
    // Resubmitting the same value is a no-op commit, but the ordered write retries first.
    write_value(&live, &port, "always_on_microphone", json!(true))?;
    assert_eq!(port.native("always_on_microphone"), Some(json!(true)));
    assert!(live.sequence().unapplied_keys().is_empty());
    Ok(())
}
/// Prepared change that mirrors `key` into the spy's native state and restores it on refusal.
fn mirrored<'a>(port: &'a SpyPort, key: &'static str, value: Value) -> PreparedChange<'a> {
    PreparedChange {
        prepare: Box::new(move |_| {
            port.set_native(key, value);
            Ok(())
        }),
        compensate: Box::new(move |before: &AppSettings| {
            let before = serde_json::to_value(before).unwrap_or_default();
            port.set_native(key, before[key].clone());
        }),
    }
}
#[test]
fn refused_preparation_leaves_disk_revision_native_and_ui_unchanged() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    let bytes = std::fs::read(store(&directory))?;
    let revision = live.snapshot().revision;
    let refusal = PreparedChange {
        prepare: Box::new(|_| Err(SettingsError::live_state_refused("vad_backend"))),
        compensate: Box::new(|_| port.record("compensated".into())),
    };
    let error = write_prepared(&live, &port, "vad_backend", json!("earshot"), refusal)
        .err()
        .ok_or("refused preparation committed")?;
    assert_eq!(
        (error.code(), error.field()),
        (SettingsErrorCode::Unavailable, Some("vad_backend"))
    );
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    assert_eq!(live.snapshot().revision, revision);
    assert!(port.events().is_empty());
    Ok(())
}
#[test]
fn refused_commit_compensates_the_prepared_native_change() -> TestResult {
    let directory = tempfile::tempdir()?;
    drop(owned(&directory)?);
    let bytes = std::fs::read(store(&directory))?;
    let live = LiveSettings::start_read_only(store(&directory), SettingsErrorCode::Unavailable);
    let port = SpyPort::default();
    port.set_native("vad_backend", json!("silero"));
    let change = mirrored(&port, "vad_backend", json!("earshot"));
    let error = write_prepared(&live, &port, "vad_backend", json!("earshot"), change)
        .err()
        .ok_or("read-only commit accepted")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert_eq!(port.native("vad_backend"), Some(json!("silero")));
    assert_eq!(std::fs::read(store(&directory))?, bytes);
    assert!(port.events().is_empty());
    Ok(())
}
#[test]
fn refused_prepared_edit_compensates_and_notifies_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    let refused = |_: &mut AppSettings| {
        Err(SettingsError::validation(
            "bindings",
            crate::settings_service::errors::ValidationReason::CanonicalValue,
        ))
    };
    let compensated = AtomicBool::new(false);
    let result = ordered(&live, &port, || {
        let before = live.settings();
        let edit = SettingsEdit::Apply(&refused);
        commit_prepared(&live, &port, "bindings", &edit, &before, || {
            compensated.store(true, Ordering::SeqCst)
        })
    });
    assert_eq!(
        result.err().map(|error| error.code()),
        Some(SettingsErrorCode::ValidationFailed)
    );
    assert!(compensated.load(Ordering::SeqCst));
    assert!(port.events().is_empty());
    Ok(())
}
/// Fails once just after the new generation replaced the store (durability unconfirmed).
#[derive(Default)]
struct UnconfirmedOnce(AtomicBool);
impl PersistObserver for UnconfirmedOnce {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == PersistStage::AfterReplace && !self.0.swap(true, Ordering::SeqCst) {
            return Err(std::io::Error::other("injected unconfirmed durability"));
        }
        Ok(())
    }
}
#[test]
fn unknown_outcome_that_landed_keeps_the_preparation_and_notifies_the_reread() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?.with_observer(Box::<UnconfirmedOnce>::default());
    let port = SpyPort::default();
    port.set_native("vad_backend", json!("silero"));
    let change = mirrored(&port, "vad_backend", json!("earshot"));
    let error = write_prepared(&live, &port, "vad_backend", json!("earshot"), change)
        .err()
        .ok_or("unconfirmed durability reported as a success")?;
    assert_eq!(error.outcome(), Some(Outcome::Unknown));
    assert_eq!(stored(&directory, "vad_backend")?, json!("earshot"));
    assert_eq!(port.native("vad_backend"), Some(json!("earshot")));
    let event = format!("notify:{}:vad_backend", live.snapshot().revision.0);
    assert_eq!(port.events(), vec![event]);
    Ok(())
}
/// A manager repair clears a missing selection only while it is still that selection; a user
/// selection racing with it is never lost, whichever commits first.
#[test]
fn concurrent_manager_repair_and_user_selection_lose_no_update() -> TestResult {
    for _ in 0..10 {
        let directory = tempfile::tempdir()?;
        let live = Arc::new(owned(&directory)?);
        let port = Arc::new(SpyPort::default());
        let start = live.snapshot().revision.0;
        select(&live, &port, "missing", "unplugged")?;
        let repairs = {
            let (live, port) = (Arc::clone(&live), Arc::clone(&port));
            std::thread::spawn(move || clear_missing(&live, &port))
        };
        select(&live, &port, "whisper-small", "USB")?;
        repairs.join().map_err(|_| "repair panicked")??;
        assert_eq!(
            stored(&directory, "selected_model")?,
            json!("whisper-small")
        );
        assert_eq!(stored(&directory, "selected_microphone")?, json!("USB"));
        let revision = live.snapshot().revision.0;
        let notified = port
            .events()
            .iter()
            .filter(|e| e.starts_with("notify:"))
            .count();
        assert_eq!(
            notified as u64,
            revision - start,
            "one event per committed change"
        );
    }
    Ok(())
}
/// User selection of a model and a microphone (ordered write).
fn select(live: &LiveSettings, port: &SpyPort, model: &str, microphone: &str) -> TestResult {
    write_edit(live, port, &|settings| {
        settings.selected_model = model.into();
        settings.selected_microphone = Some(microphone.into());
        Ok(())
    })?;
    Ok(())
}
/// The model and microphone managers' repairs (unordered), each conditional on the locked state.
fn clear_missing(live: &LiveSettings, port: &SpyPort) -> Result<(), SettingsError> {
    repair(live, port, &|settings| {
        if settings.selected_model == "missing" {
            settings.selected_model.clear();
        }
        Ok(())
    })?;
    repair(live, port, &|settings| {
        if settings.selected_microphone.as_deref() == Some("unplugged") {
            settings.selected_microphone = None;
        }
        Ok(())
    })?;
    Ok(())
}
#[path = "sequence_race_tests.rs"]
mod races;
