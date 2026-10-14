//! Operations routed to a running instance, and refusal next to an older Handy.
use super::harness::{Fallible, Portable, TestResult};
use super::support::{pairs, path, replacements, texts, write};
use handy_app_lib::settings_service::{
    ipc, live::LiveSettings, ownership::LegacyPathProbe, store::SettingsRepository,
};
use serde_json::json;
use std::sync::Arc;

fn running_instance(app: &Portable) -> Fallible<LiveSettings> {
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    Ok(LiveSettings::start_owned(
        SettingsRepository::new(app.store()?),
        &probe,
    )?)
}

/// Add, merge-import and export through the instance; returns the export's exit code.
fn add_import_export(app: &Portable, start: u64) -> Fallible<i32> {
    let (code, added) = replacements(app, &["add", "cloud code", "Claude Code"])?;
    assert_eq!((code, &added["revision"]), (0, &json!(start + 1)));
    let file = write(app, "team.txt", b"get hub;GitHub\n")?;
    let (code, imported) = replacements(app, &["import", &file, "--mode", "merge"])?;
    assert_eq!((code, &imported["revision"]), (0, &json!(start + 2)));
    let exported = path(app, "from-instance.txt");
    Ok(replacements(app, &["export", &exported])?.0)
}

#[test]
fn operations_routed_to_a_running_instance_are_committed_and_published_by_it() -> TestResult {
    let app = Portable::new()?;
    let instance = Arc::new(running_instance(&app)?);
    let endpoint = ipc::serve(&app.store()?, instance.clone())?;
    let start = instance.snapshot().revision.0;
    assert_eq!(add_import_export(&app, start)?, 0);
    let published = serde_json::to_value(instance.snapshot().public_settings())?;
    let expected = pairs(&[("cloud code", "Claude Code"), ("get hub", "GitHub")]);
    assert_eq!(texts(&published["word_replacements"]), expected);
    let written = std::fs::read_to_string(path(&app, "from-instance.txt"))?;
    assert_eq!(
        written,
        "recognized;replacement\ncloud code;Claude Code\nget hub;GitHub\n"
    );
    endpoint.close();
    Ok(())
}

#[test]
fn with_an_older_handy_running_changes_are_unavailable_and_write_nothing() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let (code, envelope) = replacements(&app, &["add", "a", "b"])?;
    assert_eq!(
        (code, &envelope["error"]["code"]),
        (5, &json!("unavailable"))
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
