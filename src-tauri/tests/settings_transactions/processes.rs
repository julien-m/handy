//! Subprocess barriers and an independent reader verify real atomic repository generations.
use super::driver::Driver;
use handy_app_lib::settings_service::platform_capability::PlatformCapabilities;
use handy_app_lib::settings_service::{
    ownership::LegacyPathProbe,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
const DEADLINE: Duration = Duration::from_secs(10);
#[test]
fn overlapping_subprocess_writers_serialize_under_the_lock_and_preserve_disjoint_fields(
) -> TestResult<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    std::fs::write(
        &path,
        serde_json::to_vec(
            &json!({"settings_revision":0,"settings":{"audio_feedback":true,"start_hidden":false}}),
        )?,
    )?;
    let mut first = Driver::start(&path, "pause_after_load", "audio_feedback", "false")?;
    let mut second = Driver::start(&path, "write", "start_hidden", "true")?;
    first.send("GO")?;
    // Forced overlap: the first writer holds the transaction lock after its fresh read.
    first.receive("PAUSED")?;
    second.send("GO")?;
    second.receive("COMMITTING")?;
    // Well under the 5 s lock deadline: the second writer must still be waiting for the lock.
    assert!(second
        .receive_within("RESULT ", Duration::from_millis(500))?
        .is_none());
    first.send("RESUME")?;
    let first_result = first.outcome()?;
    let second_result = second.outcome()?;
    assert_eq!(first_result["ok"], true);
    assert_eq!(second_result["ok"], true);
    assert_eq!(first_result["revision"], 1);
    assert_eq!(second_result["revision"], 2);
    first.finish()?;
    second.finish()?;
    let disk: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(disk["settings_revision"], 2);
    assert_eq!(disk["settings"]["audio_feedback"], false);
    assert_eq!(disk["settings"]["start_hidden"], true);
    Ok(())
}
#[test]
fn barrier_started_startup_and_writer_respect_real_lease_ownership() -> TestResult<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let original = serde_json::to_value(
        SettingsRepository::new(path.clone())
            .read_snapshot()?
            .snapshot
            .public_settings(),
    )?["audio_feedback"]
        .as_bool()
        .ok_or("default feedback")?;
    let mut startup = Driver::start(&path, "startup", "", "")?;
    let mut writer = Driver::start(
        &path,
        "write",
        "audio_feedback",
        if original { "false" } else { "true" },
    )?;
    startup.send("GO")?;
    writer.send("GO")?;
    let startup_result = startup.outcome()?;
    let writer_result = writer.outcome()?;
    assert_eq!(startup_result["ok"], true);
    let disk: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    if writer_result["ok"] == true {
        assert_eq!(disk["settings"]["audio_feedback"], !original);
        assert!(
            startup_result["revision"]
                .as_u64()
                .ok_or("startup revision")?
                >= writer_result["revision"]
                    .as_u64()
                    .ok_or("writer revision")?
        );
    } else {
        assert_eq!(writer_result["error"]["code"], "unavailable");
        assert_eq!(disk["settings_revision"], startup_result["revision"]);
    }
    startup.send("RELEASE")?;
    startup.finish()?;
    writer.finish()?;
    Ok(())
}
#[test]
fn independent_raw_reader_observes_only_complete_old_or_new_generations() -> TestResult<()> {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    };
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    std::fs::write(
        &path,
        serde_json::to_vec(&json!({"settings_revision":0,"settings":{"audio_feedback":true}}))?,
    )?;
    let barrier = Arc::new(Barrier::new(2));
    let stop = Arc::new(AtomicBool::new(false));
    let reader_path = path.clone();
    let reader_barrier = barrier.clone();
    let reader_stop = stop.clone();
    let reader =
        std::thread::spawn(move || read_generations(reader_path, reader_barrier, reader_stop));
    barrier.wait();
    let repository = SettingsRepository::new(path.clone());
    let probe = LegacyPathProbe::new(directory.path().join("absent-legacy.sock"));
    for revision in 1..=32 {
        let mutation = validate_cli(
            "audio_feedback",
            if revision % 2 == 0 { "true" } else { "false" },
            &ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
                platform: PlatformCapabilities::current(),
            },
        )?;
        assert_eq!(
            repository
                .commit_with_probe(mutation, None, &probe, &())?
                .snapshot
                .revision
                .0,
            revision
        );
    }
    stop.store(true, Ordering::Release);
    assert!(
        reader
            .join()
            .map_err(|_| "reader panic")?
            .map_err(|error| format!("reader: {error}"))?
            > 0
    );
    Ok(())
}

fn read_generations(
    path: std::path::PathBuf,
    barrier: std::sync::Arc<std::sync::Barrier>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<usize, String> {
    use std::sync::atomic::Ordering;
    barrier.wait();
    let deadline = Instant::now() + DEADLINE;
    let mut count = 0;
    loop {
        let bytes = std::fs::read(&path).map_err(|_| "raw read")?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| "partial JSON")?;
        let revision = value["settings_revision"].as_u64().ok_or("revision")?;
        if value["settings"]["audio_feedback"] != json!(revision % 2 == 0) {
            return Err("mixed generation".into());
        }
        count += 1;
        if stop.load(Ordering::Acquire) {
            return Ok(count);
        }
        if Instant::now() > deadline {
            return Err("reader deadline".into());
        }
    }
}

#[test]
fn forced_process_termination_before_and_after_replace_preserves_complete_generations_and_releases_locks(
) -> TestResult<()> {
    for mode in ["crash_before", "crash_after"] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("settings_store.json");
        let seed = json!({"settings_revision":7,"settings":{"audio_feedback":true,"unknown":"CANARY_KEEP"}});
        std::fs::write(&path, serde_json::to_vec(&seed)?)?;
        // Normalize first (revision 8) so the crashed write changes exactly one visible field.
        let probe = LegacyPathProbe::new(directory.path().join("absent-legacy.sock"));
        drop(SettingsRepository::new(path.clone()).initialize(&probe)?);
        let old: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        let mut driver = Driver::start(&path, mode, "audio_feedback", "false")?;
        driver.send("GO")?;
        driver.wait_crash_barrier()?;
        driver.kill_at_barrier()?;
        let revision = assert_crash_generation(&path, mode, &old)?;
        let mutation = validate_cli(
            "audio_feedback",
            if mode == "crash_before" {
                "false"
            } else {
                "true"
            },
            &ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
                platform: PlatformCapabilities::current(),
            },
        )?;
        assert_eq!(
            SettingsRepository::new(path)
                .commit_with_probe(mutation, None, &probe, &())?
                .snapshot
                .revision
                .0,
            revision + 1
        );
    }
    Ok(())
}

fn assert_crash_generation(path: &std::path::Path, mode: &str, old: &Value) -> TestResult<u64> {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    let mut expected = old.clone();
    let revision = if mode == "crash_before" {
        8
    } else {
        expected["settings_revision"] = json!(9);
        expected["settings"]["audio_feedback"] = json!(false);
        9
    };
    assert_eq!(expected["settings"]["unknown"], "CANARY_KEEP");
    let bytes = std::fs::read(path)?;
    let disk: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(disk, expected);
    assert_eq!(bytes, serde_json::to_vec(&expected)?);
    let mut observed_hash = DefaultHasher::new();
    bytes.hash(&mut observed_hash);
    let mut expected_hash = DefaultHasher::new();
    serde_json::to_vec(&expected)?.hash(&mut expected_hash);
    assert_eq!(observed_hash.finish(), expected_hash.finish());
    eprintln!(
        "CRASH_GENERATION {mode} revision={revision} fingerprint={:016x}",
        observed_hash.finish()
    );
    Ok(revision)
}
