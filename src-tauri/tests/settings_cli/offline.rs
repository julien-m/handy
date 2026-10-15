//! Offline routing: with no instance, the CLI reads without creating the store and writes it
//! under the shared lock; an older Handy or a held startup lease refuses with no store change.
use super::harness::{Fallible, Portable, TestResult};
use handy_app_lib::settings_service::{
    live::LiveSettings, ownership::LegacyPathProbe, store::SettingsRepository,
};
use serde_json::json;
use std::os::unix::net::UnixListener;

#[test]
fn headless_read_never_creates_the_store_and_a_write_persists_one_revision() -> TestResult {
    let app = Portable::new()?;
    let read = app.run(&["settings", "get", "--json"])?;
    assert_eq!((read.code, app.store_bytes()?), (0, None));
    let set = app.run(&["settings", "set", "paste_delay_ms", "250", "--json"])?;
    assert_eq!(
        set.envelope()?,
        json!({"ok": true, "revision": 1, "data": {"paste_delay_ms": 250}})
    );
    let stored = SettingsRepository::new(app.store()?).read_snapshot()?;
    assert_eq!(stored.snapshot.revision.0, 1);
    let public = serde_json::to_value(stored.snapshot.public_settings())?;
    assert_eq!(public["paste_delay_ms"], json!(250));
    let same = app.run(&["settings", "set", "paste_delay_ms", "250", "--json"])?;
    assert_eq!(
        same.envelope()?["revision"],
        json!(1),
        "a no-op keeps the revision"
    );
    Ok(())
}
#[test]
fn rejected_write_changes_nothing() -> TestResult {
    let app = Portable::new()?;
    app.run(&["settings", "set", "paste_method", "direct"])?;
    let before = app.store_bytes()?;
    for (key, value) in [
        ("paste_method", "invalid_method"),
        ("hold_threshold_ms", "99"),
        ("audio_feedback", "yes"),
    ] {
        let run = app.run(&["settings", "set", key, value, "--json"])?;
        assert_eq!(run.code, 2, "{key}");
    }
    assert_eq!(app.store_bytes()?, before);
    Ok(())
}
#[test]
fn older_handy_holding_its_singleton_refuses_reads_and_writes_without_touching_the_store(
) -> TestResult {
    let app = Portable::new()?;
    app.run(&["settings", "set", "paste_method", "direct"])?;
    let before = app.store_bytes()?;
    let _older = UnixListener::bind(app.legacy_socket())?;
    for args in [
        &["settings", "get", "--json"][..],
        &["settings", "set", "paste_method", "none", "--json"],
    ] {
        let run = app.run(args)?;
        assert_eq!(
            (run.code, &run.envelope()?["error"]["code"]),
            (5, &json!("unavailable"))
        );
    }
    assert_eq!(app.store_bytes()?, before);
    Ok(())
}
fn corrupt(app: &Portable) -> Fallible<()> {
    std::fs::write(app.store()?, b"not json")?;
    Ok(())
}
#[test]
fn unreadable_store_reads_defaults_with_a_stderr_warning_and_refuses_writes() -> TestResult {
    let app = Portable::new()?;
    corrupt(&app)?;
    let read = app.run(&["settings", "get", "paste_method", "--json"])?;
    assert_eq!(
        (read.code, &read.envelope()?["data"]["paste_method"]),
        (0, &json!(super::contract::default_paste_method()))
    );
    assert!(read
        .stderr
        .contains("Warning: the settings store was recovered"));
    let write = app.run(&["settings", "set", "paste_method", "direct", "--json"])?;
    assert_eq!(
        (write.code, &write.envelope()?["error"]["code"]),
        (1, &json!("persistence_failed"))
    );
    assert_eq!(app.store_bytes()?, Some(b"not json".to_vec()));
    Ok(())
}
#[test]
fn concurrent_cli_writers_lose_no_field() -> TestResult {
    let app = std::sync::Arc::new(Portable::new()?);
    app.run(&["settings", "get"])?;
    let keys = [
        "audio_feedback",
        "start_hidden",
        "translate_to_english",
        "lazy_stream_close",
    ];
    let writers = keys.map(|key| {
        let app = std::sync::Arc::clone(&app);
        std::thread::spawn(move || {
            let run = app.run(&["settings", "set", key, "true", "--json"]);
            run.map(|run| run.code).map_err(|error| error.to_string())
        })
    });
    for writer in writers {
        assert_eq!(writer.join().map_err(|_| "writer panicked")??, 0);
    }
    let all = app.run(&["settings", "get", "--json"])?.envelope()?;
    assert_eq!(all["revision"], json!(keys.len()));
    assert!(keys.iter().all(|key| all["data"][key] == json!(true)));
    Ok(())
}
/// A running Handy holds the startup lease (stood in for by an owned live service).
pub fn running_instance(app: &Portable) -> Fallible<LiveSettings> {
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    Ok(LiveSettings::start_owned(
        SettingsRepository::new(app.store()?),
        &probe,
    )?)
}
#[test]
fn held_startup_lease_without_a_reachable_channel_refuses_and_never_writes_offline() -> TestResult {
    let app = Portable::new()?;
    let instance = running_instance(&app)?;
    let before = app.store_bytes()?;
    let run = app.run(&["settings", "set", "paste_method", "direct", "--json"])?;
    assert_eq!(
        (run.code, &run.envelope()?["error"]["code"]),
        (5, &json!("unavailable"))
    );
    assert_eq!(app.store_bytes()?, before);
    assert_eq!(instance.snapshot().revision.0, 1);
    Ok(())
}
