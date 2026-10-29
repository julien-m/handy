//! Routing to a running instance, simulated by a live settings service that owns the store and
//! serves the production private endpoint: operations are answered by the instance, a missing
//! answer is an unknown outcome, and the endpoint name follows the lease (reclaimed when stale,
//! removed at close, never taken from a live listener).
use super::harness::{Fallible, Portable, TestResult};
use super::offline::running_instance;
use handy_app_lib::settings_service::{
    errors::SettingsError, ipc, live::LiveSettings, operations::SettingsBackend,
    protocol::Revision, store::SettingsRead, validation::SettingsMutation,
};
use serde_json::json;
use std::{sync::Arc, time::Duration, time::Instant};

fn serve(app: &Portable) -> Fallible<(Arc<LiveSettings>, ipc::EndpointHandle)> {
    let instance = Arc::new(running_instance(app)?);
    let handle = ipc::serve(&app.store()?, instance.clone())?;
    Ok((instance, handle))
}
#[test]
fn the_running_instance_answers_reads_and_commits_writes_with_its_live_state() -> TestResult {
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    let start = instance.snapshot().revision.0;
    let method = super::contract::other_paste_method()?;
    let set = app.run(&["settings", "set", "paste_method", method, "--json"])?;
    let expected = json!({"ok": true, "revision": start + 1, "data": {"paste_method": method}});
    assert_eq!(set.envelope()?, expected);
    let public = serde_json::to_value(instance.snapshot().public_settings())?;
    assert_eq!(
        public["paste_method"],
        json!(method),
        "published by the instance"
    );
    let get = app.run(&["settings", "get", "paste_method"])?;
    assert_eq!(
        (get.code, get.stdout.as_str()),
        (0, format!("{method}\n").as_str())
    );
    endpoint.close();
    Ok(())
}
#[test]
fn the_running_instance_refuses_stale_and_invalid_writes_without_committing() -> TestResult {
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    let start = instance.snapshot().revision.0;
    let args = [
        "settings",
        "set",
        "paste_method",
        "none",
        "--expected-revision",
        "999",
        "--json",
    ];
    let stale = app.run(&args)?;
    let details = stale.envelope()?["error"]["details"].clone();
    assert_eq!(
        (stale.code, details),
        (3, json!({"current_revision": start}))
    );
    let invalid = app.run(&[
        "settings",
        "set",
        "paste_method",
        "invalid_method",
        "--json",
    ])?;
    assert_eq!((invalid.code, instance.snapshot().revision.0), (2, start));
    endpoint.close();
    Ok(())
}
/// An instance that commits only after the CLI deadline.
struct Late(Arc<LiveSettings>);
impl SettingsBackend for Late {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        self.0.read()
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        std::thread::sleep(ipc::DEADLINE + Duration::from_millis(500));
        self.0.commit(mutation, expected)
    }
}
#[test]
fn no_answer_within_five_seconds_is_an_unknown_outcome_resolved_by_a_new_read() -> TestResult {
    let app = Portable::new()?;
    let instance = Arc::new(running_instance(&app)?);
    let endpoint = ipc::serve(&app.store()?, Arc::new(Late(instance.clone())))?;
    let before = app.run(&["settings", "get", "paste_method"])?;
    let started = Instant::now();
    let method = super::contract::other_paste_method()?;
    let run = app.run(&["settings", "set", "paste_method", method, "--json"])?;
    let elapsed = started.elapsed();
    assert_eq!(before.code, 0);
    let error = run.envelope()?["error"].clone();
    assert_eq!(
        (run.code, &error["code"], &error["details"]["outcome"]),
        (5, &json!("unavailable"), &json!("unknown"))
    );
    assert!(
        elapsed < ipc::DEADLINE + Duration::from_secs(2),
        "{elapsed:?}"
    );
    std::thread::sleep(Duration::from_secs(1));
    let reread = app.run(&["settings", "get", "paste_method", "--json"])?;
    assert_eq!(
        reread.envelope()?["data"]["paste_method"],
        json!(method),
        "the late commit is visible"
    );
    endpoint.close();
    Ok(())
}
#[test]
fn a_stale_endpoint_name_is_reclaimed_under_the_lease_and_removed_at_close() -> TestResult {
    use std::os::unix::{fs::DirBuilderExt, net::UnixListener};
    let app = Portable::new()?;
    let path = ipc::endpoint_path(&app.store()?)?;
    let parent = path.parent().ok_or("no parent")?;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(parent)?;
    drop(UnixListener::bind(&path)?);
    let (instance, endpoint) = serve(&app)?;
    assert_eq!(app.run(&["settings", "get", "--json"])?.code, 0);
    endpoint.close();
    drop(instance);
    assert!(!path.exists(), "a normal shutdown leaves no name behind");
    let (_instance, restarted) = serve(&app)?;
    assert_eq!(app.run(&["settings", "keys", "--json"])?.code, 0);
    restarted.close();
    Ok(())
}
#[test]
fn a_live_listener_on_the_endpoint_name_is_never_replaced() -> TestResult {
    use std::os::unix::{fs::DirBuilderExt, fs::MetadataExt, net::UnixListener};
    let app = Portable::new()?;
    let path = ipc::endpoint_path(&app.store()?)?;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(path.parent().ok_or("no parent")?)?;
    let _squatter = UnixListener::bind(&path)?;
    let inode = std::fs::symlink_metadata(&path)?.ino();
    let instance = Arc::new(running_instance(&app)?);
    assert!(ipc::serve(&app.store()?, instance).is_err());
    assert_eq!(std::fs::symlink_metadata(&path)?.ino(), inode);
    std::fs::remove_file(&path)?;
    Ok(())
}
