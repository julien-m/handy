//! Process tests of `handy settings shortcuts` (feature 004) on the real binary and an isolated
//! portable store: offline list and refusal of changes, dry run, and changes routed to a running
//! instance simulated by a live settings service serving the production endpoint. The simulated
//! instance commits like the app but registers nothing natively (the native preparation is
//! tested in the library); it can play a dictation session in progress.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use handy_app_lib::settings_service::{
    domains::DomainChange,
    errors::SettingsError,
    ipc,
    live::LiveSettings,
    operations::SettingsBackend,
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use harness::{Fallible, Portable, Run, TestResult};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The running Handy, as seen through its settings endpoint.
struct Instance {
    live: LiveSettings,
    /// A dictation session owns the shortcuts: every shortcut change is refused.
    busy: AtomicBool,
}
impl SettingsBackend for Instance {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(&self.live)
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(&self.live, mutation, expected)
    }
    fn commit_live_change(
        &self,
        key: &'static str,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        if self.busy.load(Ordering::SeqCst) {
            return Err(SettingsError::session_busy(key));
        }
        SettingsBackend::commit_change(&self.live, change, expected)
    }
    fn is_live(&self) -> bool {
        true
    }
}
fn serve(app: &Portable) -> Fallible<(Arc<Instance>, ipc::EndpointHandle)> {
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    let live = LiveSettings::start_owned(SettingsRepository::new(app.store()?), &probe)?;
    let instance = Arc::new(Instance {
        live,
        busy: AtomicBool::new(false),
    });
    let handle = ipc::serve(&app.store()?, instance.clone())?;
    Ok((instance, handle))
}
fn shortcuts(app: &Portable, args: &[&str]) -> Fallible<Run> {
    let mut argv = vec!["settings", "shortcuts"];
    argv.extend_from_slice(args);
    argv.push("--json");
    app.run(&argv)
}
fn default_of(action: &str) -> Fallible<String> {
    let app = Portable::new()?;
    let list = shortcuts(&app, &["list", action])?;
    Ok(list.envelope()?["data"]["default_shortcut"]
        .as_str()
        .ok_or("no default")?
        .to_owned())
}
fn error_of(run: &Run) -> Fallible<Value> {
    Ok(run.envelope()?["error"].clone())
}

// @spec AC-006: Shortcuts CLI list offline without writing — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-006
#[test]
fn offline_list_reads_without_creating_the_store() -> TestResult {
    let app = Portable::new()?;
    let list = shortcuts(&app, &["list"])?;
    let envelope = list.envelope()?;
    assert_eq!((list.code, &envelope["ok"]), (0, &json!(true)));
    let actions = envelope["data"]["actions"].as_array().ok_or("actions")?;
    assert!(actions
        .iter()
        .any(|action| action["action_id"] == "transcribe"));
    let human = app.run(&["settings", "shortcuts", "list", "transcribe"])?;
    let transcribe = default_of("transcribe")?;
    assert_eq!(
        human.stdout,
        format!("transcribe: {transcribe} (default: {transcribe})\n")
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

#[test]
fn offline_changes_are_refused_but_a_dry_run_previews_them() -> TestResult {
    let app = Portable::new()?;
    let refused = shortcuts(&app, &["add", "transcribe", "f13"])?;
    let error = error_of(&refused)?;
    assert_eq!((refused.code, &error["code"]), (5, &json!("unavailable")));
    assert_eq!(error["details"]["reason"], "instance_required");
    let preview = shortcuts(&app, &["add", "transcribe", "f13", "--dry-run"])?;
    let data = preview.envelope()?["data"].clone();
    assert_eq!(
        (preview.code, &data["registration_checked"]),
        (0, &json!(false))
    );
    assert_eq!(data["changes"]["shortcuts"][1], "f13");
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

// @spec AC-006: Live CLI changes with stable codes — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-006
#[test]
fn a_running_instance_commits_each_change_as_one_revision() -> TestResult {
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    let start = instance.live.snapshot().revision.0;
    let added = shortcuts(&app, &["add", "transcribe", "ctrl+alt+d"])?;
    let envelope = added.envelope()?;
    assert_eq!((added.code, &envelope["revision"]), (0, &json!(start + 1)));
    let transcribe = default_of("transcribe")?;
    assert_eq!(
        envelope["data"]["shortcuts"],
        json!([transcribe, "ctrl+alt+d"])
    );
    let list = shortcuts(&app, &["list", "transcribe"])?;
    assert_eq!(list.envelope()?["data"]["shortcuts"][1], "ctrl+alt+d");
    let removed = shortcuts(&app, &["remove", "transcribe", "alt+ctrl+d"])?;
    assert_eq!(removed.envelope()?["revision"], json!(start + 2));
    let reset = shortcuts(&app, &["reset", "transcribe"])?;
    assert_eq!(reset.envelope()?["revision"], json!(start + 2), "a no-op");
    let stored = SettingsRepository::new(app.store()?).read_snapshot()?;
    assert_eq!(stored.snapshot.revision.0, start + 2);
    endpoint.close();
    Ok(())
}

#[test]
fn conflicts_and_the_limit_are_refused_with_their_owner_and_limit() -> TestResult {
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    let start = instance.live.snapshot().revision.0;
    shortcuts(&app, &["add", "transcribe", "f13"])?;
    let conflict = shortcuts(&app, &["add", "transcribe_with_post_process", "f13"])?;
    let error = error_of(&conflict)?;
    assert_eq!(
        (conflict.code, &error["code"]),
        (2, &json!("shortcut_conflict"))
    );
    assert_eq!(error["details"]["owning_action"], "transcribe");
    for shortcut in ["f14", "f15"] {
        assert_eq!(shortcuts(&app, &["add", "transcribe", shortcut])?.code, 0);
    }
    let limited = shortcuts(&app, &["add", "transcribe", "f16"])?;
    assert_eq!(error_of(&limited)?["code"], "limit_exceeded");
    let revision = instance.live.snapshot().revision.0;
    assert_eq!(revision, start + 3, "one revision per accepted change");
    endpoint.close();
    Ok(())
}

#[test]
fn last_removal_unknown_action_and_stale_revision_change_nothing() -> TestResult {
    let app = Portable::new()?;
    let (_instance, endpoint) = serve(&app)?;
    let before = app.store_bytes()?;
    let cancel = default_of("cancel")?;
    let last = shortcuts(&app, &["remove", "cancel", &cancel])?;
    assert_eq!(error_of(&last)?["details"]["reason"], "last_shortcut");
    let unknown = shortcuts(&app, &["reset", "no_such_action"])?;
    assert_eq!(
        (unknown.code, error_of(&unknown)?["code"].clone()),
        (4, json!("not_found"))
    );
    let stale = shortcuts(
        &app,
        &["add", "transcribe", "f13", "--expected-revision", "0"],
    )?;
    assert_eq!(stale.code, 3);
    assert_eq!(app.store_bytes()?, before);
    endpoint.close();
    Ok(())
}

// @spec FR-002: Changes wait for a quiescent coordinator — .specs/features/004-multiple-dictation-shortcuts/spec.md#fr-002
#[test]
fn a_dictation_in_progress_refuses_the_change_with_shortcut_session_busy() -> TestResult {
    let app = Portable::new()?;
    let (instance, endpoint) = serve(&app)?;
    instance.busy.store(true, Ordering::SeqCst);
    let before = app.store_bytes()?;
    let busy = shortcuts(&app, &["add", "transcribe", "f13"])?;
    let error = error_of(&busy)?;
    assert_eq!((busy.code, &error["code"]), (5, &json!("unavailable")));
    assert_eq!(error["details"]["reason"], "shortcut_session_busy");
    assert_eq!(app.store_bytes()?, before);
    let human = app.run(&["settings", "shortcuts", "reset", "transcribe"])?;
    assert!(
        human.stderr.contains("reason: shortcut_session_busy"),
        "{}",
        human.stderr
    );
    instance.busy.store(false, Ordering::SeqCst);
    assert_eq!(shortcuts(&app, &["add", "transcribe", "f13"])?.code, 0);
    endpoint.close();
    Ok(())
}

#[test]
fn help_lists_the_shortcut_actions_and_generic_set_points_to_them() -> TestResult {
    let app = Portable::new()?;
    let help = app.run(&["settings", "shortcuts", "--help"])?;
    for action in ["list", "add", "remove", "reset", "instance_required"] {
        assert!(help.stdout.contains(action), "{action}");
    }
    let set = app.run(&["settings", "set", "bindings", "x", "--json"])?;
    let issues = error_of(&set)?["details"]["issues"].clone();
    assert_eq!(
        issues[0]["reason"],
        "Use handy settings shortcuts to change shortcuts."
    );
    Ok(())
}

#[test]
fn an_older_handy_without_the_settings_channel_refuses_even_the_list() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let list = shortcuts(&app, &["list"])?;
    assert_eq!(
        (list.code, error_of(&list)?["code"].clone()),
        (5, json!("unavailable"))
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
