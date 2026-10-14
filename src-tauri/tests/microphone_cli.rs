//! Process tests of `handy settings microphones` (feature 001) on the real binary and an isolated
//! portable store: set-priority offline and routed (complete order, empty order, exact and
//! hyphen-leading names, refusals with items, compare-and-set, dry run), list refused without a
//! running Handy's audio manager, generic `set` of either microphone key refused, one envelope.
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
use std::sync::Arc;

fn microphones(app: &Portable, args: &[&str]) -> Fallible<Run> {
    let mut argv = vec!["settings", "--json", "microphones"];
    argv.extend_from_slice(args);
    app.run(&argv)
}
fn envelope(run: &Run) -> Fallible<Value> {
    run.envelope()
}

// @spec AC-008: set-priority with the 008 envelope and exits — .specs/features/001-microphone-priority/spec.md#ac-008
#[test]
fn set_priority_offline_commits_the_order_that_get_reads_back() -> TestResult {
    let app = Portable::new()?;
    let set = microphones(&app, &["set-priority", "USB headset", "default"])?;
    let data = json!({"microphone_priority": ["USB headset", "default"]});
    assert_eq!((set.code, &envelope(&set)?["data"]), (0, &data));
    let get = app.run(&["settings", "get", "microphone_priority", "--json"])?;
    assert_eq!(envelope(&get)?["data"], data);
    assert_eq!(envelope(&get)?["revision"], 1);
    let cleared = microphones(&app, &["set-priority", "--expected-revision", "1"])?;
    assert_eq!(
        envelope(&cleared)?["data"],
        json!({"microphone_priority": []})
    );
    Ok(())
}
#[test]
fn names_are_exact_including_spaces_and_a_leading_hyphen_after_the_separator() -> TestResult {
    let app = Portable::new()?;
    let set = microphones(&app, &["set-priority", "--", "-Mic", " Studio  mic "])?;
    let expected = json!({"microphone_priority": ["-Mic", " Studio  mic "]});
    assert_eq!((set.code, envelope(&set)?["data"].clone()), (0, expected));
    Ok(())
}
#[test]
fn refused_orders_and_stale_revisions_write_nothing() -> TestResult {
    let app = Portable::new()?;
    microphones(&app, &["set-priority", "USB headset"])?;
    let before = app.store_bytes()?;
    let duplicate = microphones(&app, &["set-priority", "Mac", "Mac"])?;
    let error = envelope(&duplicate)?["error"].clone();
    assert_eq!(
        (duplicate.code, &error["code"]),
        (2, &json!("validation_failed"))
    );
    assert_eq!(
        error["details"]["items"][0]["code"],
        "microphone_name_duplicate"
    );
    let stale = microphones(&app, &["set-priority", "Mac", "--expected-revision", "0"])?;
    let error = envelope(&stale)?["error"].clone();
    assert_eq!(
        (stale.code, &error["details"]["current_revision"]),
        (3, &json!(1))
    );
    assert_eq!(app.store_bytes()?, before);
    Ok(())
}
#[test]
fn generic_set_of_either_microphone_key_names_the_microphones_domain() -> TestResult {
    let app = Portable::new()?;
    for key in ["selected_microphone", "microphone_priority"] {
        let run = app.run(&["settings", "set", key, "USB", "--json"])?;
        let error = envelope(&run)?["error"].clone();
        assert_eq!(
            (run.code, &error["details"]["reason"]),
            (2, &json!("microphones_domain"))
        );
        assert_eq!(error["details"]["field"], key);
    }
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
// @spec AC-008: list needs the running Handy — .specs/features/001-microphone-priority/spec.md#ac-008
#[test]
fn list_without_a_running_handy_is_unavailable_with_empty_details() -> TestResult {
    let app = Portable::new()?;
    let list = microphones(&app, &["list"])?;
    let error = envelope(&list)?["error"].clone();
    assert_eq!((list.code, &error["code"]), (5, &json!("unavailable")));
    assert_eq!(error["details"], json!({}));
    let human = app.run(&["settings", "microphones", "list"])?;
    assert_eq!((human.code, human.stdout.as_str()), (5, ""));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
#[test]
fn a_dry_run_previews_without_writing_and_help_lists_the_domain() -> TestResult {
    let app = Portable::new()?;
    let preview = microphones(&app, &["set-priority", "USB headset", "--dry-run"])?;
    let changes = json!({"microphone_priority": ["USB headset"]});
    assert_eq!(envelope(&preview)?["data"]["changes"], changes);
    assert_eq!(app.store_bytes()?, None);
    let help = app.run(&["settings", "--help"])?;
    assert!(help.stdout.contains("microphones"));
    Ok(())
}

/// The running Handy as its settings endpoint sees it, without the audio manager of the app.
struct Instance(LiveSettings);
impl SettingsBackend for Instance {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(&self.0)
    }
    fn commit(
        &self,
        m: SettingsMutation,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(&self.0, m, e)
    }
    fn commit_change(
        &self,
        c: &DomainChange,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(&self.0, c, e)
    }
    fn is_live(&self) -> bool {
        true
    }
}
#[test]
fn a_running_instance_commits_set_priority_once_and_list_needs_its_audio_manager() -> TestResult {
    let app = Portable::new()?;
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    let live = LiveSettings::start_owned(SettingsRepository::new(app.store()?), &probe)?;
    let instance = Arc::new(Instance(live));
    let endpoint = ipc::serve(&app.store()?, instance.clone())?;
    let before = instance.0.snapshot().revision.0;
    let set = microphones(&app, &["set-priority", "USB headset"])?;
    assert_eq!(
        (set.code, &envelope(&set)?["revision"]),
        (0, &json!(before + 1))
    );
    assert_eq!(instance.0.snapshot().revision, Revision(before + 1));
    let list = microphones(&app, &["list"])?;
    assert_eq!(
        (list.code, &envelope(&list)?["error"]["code"]),
        (5, &json!("unavailable"))
    );
    endpoint.close();
    Ok(())
}
