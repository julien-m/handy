//! The shortcuts domain on the production framework, against a real store: list, add, remove,
//! reset, their 008 envelope codes and details, one revision per change, live-only policy and
//! dry run. The native preparation of a running instance is tested in `shortcut::live_change`
//! and by the CLI process tests; here the live backend commits without it.
use super::ShortcutsDomain;
use crate::settings_service::{
    domains::{execute, DomainCall, DomainChange},
    errors::SettingsError,
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use serde_json::{json, Value};
use std::path::PathBuf;
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A running instance's view of the service, committing live changes without native side.
struct Live<'a>(&'a LiveSettings);
impl SettingsBackend for Live<'_> {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(self.0, mutation, expected)
    }
    fn commit_live_change(
        &self,
        _key: &'static str,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self.0, change, expected)
    }
    fn is_live(&self) -> bool {
        true
    }
}
fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn service(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    LiveSettings::start_offline(SettingsRepository::new(store(directory)), Box::new(probe))
}
/// Run one `shortcuts` call; the reply as the CLI would render it.
fn call(backend: &dyn SettingsBackend, action: &str, arguments: &[&str], dry_run: bool) -> Reply {
    call_at(backend, action, arguments, dry_run, None)
}
fn call_at(
    backend: &dyn SettingsBackend,
    action: &str,
    arguments: &[&str],
    dry_run: bool,
    expected: Option<u64>,
) -> Reply {
    let arguments = arguments
        .iter()
        .map(|a| (*a).to_owned())
        .collect::<Vec<_>>();
    let call = DomainCall {
        domain: "shortcuts",
        action,
        arguments: &arguments,
        input: None,
        dry_run,
        expected: expected.map(Revision),
    };
    match execute(&call, backend, &[&ShortcutsDomain]) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    }
}
fn default_of(action: &str) -> Result<String, Box<dyn std::error::Error>> {
    let defaults = crate::settings::get_default_settings();
    Ok(defaults
        .bindings
        .get(action)
        .ok_or("no default binding")?
        .default_binding
        .clone())
}
fn error(reply: &Reply) -> &Value {
    &reply.envelope()["error"]
}

#[test]
fn list_shows_every_action_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let reply = call(&Live(&live), "list", &[], false);
    assert_eq!(reply.exit_code(), 0);
    let actions = reply.envelope()["data"]["actions"]
        .as_array()
        .ok_or("actions")?
        .clone();
    let ids: Vec<&str> = actions
        .iter()
        .filter_map(|a| a["action_id"].as_str())
        .collect();
    assert!(ids.contains(&"transcribe") && ids.contains(&"cancel"));
    let transcribe = call(&Live(&live), "list", &["transcribe"], false);
    let data = &transcribe.envelope()["data"];
    assert_eq!(data["shortcuts"], json!([default_of("transcribe")?]));
    assert_eq!(data["default_shortcut"], json!(default_of("transcribe")?));
    assert!(data["keyboard_implementation"].is_string());
    assert!(!store(&directory).exists(), "list wrote the store");
    Ok(())
}

#[test]
fn unknown_action_is_not_found_without_echo() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    for (action, arguments) in [
        ("list", vec!["CANARY_ACTION"]),
        ("reset", vec!["CANARY_ACTION"]),
    ] {
        let reply = call(&Live(&live), action, &arguments, false);
        assert_eq!(reply.exit_code(), 4);
        assert_eq!(error(&reply)["code"], "not_found");
        assert!(!reply.envelope().to_string().contains("CANARY_ACTION"));
    }
    Ok(())
}

// @spec AC-006: Shortcuts CLI add, list and conflicts — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-006
#[test]
fn add_commits_one_revision_and_a_duplicate_names_its_owner() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    let before = live.snapshot().revision.0;
    let added = call(&Live(&live), "add", &["transcribe", "ctrl+alt+d"], false);
    assert_eq!(added.exit_code(), 0, "{}", added.envelope());
    assert_eq!(added.envelope()["revision"], json!(before + 1));
    let data = &added.envelope()["data"];
    assert_eq!(
        data["shortcuts"],
        json!([default_of("transcribe")?, "ctrl+alt+d"])
    );
    let duplicate = call(
        &Live(&live),
        "add",
        &["transcribe_with_post_process", "alt+ctrl+d"],
        false,
    );
    assert_eq!(duplicate.exit_code(), 2);
    let details = &error(&duplicate)["details"];
    assert_eq!(error(&duplicate)["code"], "shortcut_conflict");
    assert_eq!(details["owning_action"], "transcribe");
    assert_eq!(details["owning_shortcut"], "ctrl+alt+d");
    assert_eq!(details["kind"], "duplicate");
    assert_eq!(live.snapshot().revision.0, before + 1);
    Ok(())
}

// @spec AC-005: No action left without a shortcut — .specs/features/004-multiple-dictation-shortcuts/spec.md#ac-005
#[test]
fn limits_last_removal_and_unsupported_alternatives_are_refused() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    for shortcut in ["f13", "f14", "f15"] {
        assert_eq!(
            call(&Live(&live), "add", &["transcribe", shortcut], false).exit_code(),
            0
        );
    }
    let fifth = call(&Live(&live), "add", &["transcribe", "f16"], false);
    assert_eq!(
        (fifth.exit_code(), error(&fifth)["code"].clone()),
        (2, json!("limit_exceeded"))
    );
    assert_eq!(error(&fifth)["details"]["limit"], 4);
    let cancel = default_of("cancel")?;
    let last = call(&Live(&live), "remove", &["cancel", &cancel], false);
    assert_eq!(error(&last)["code"], "validation_failed");
    assert_eq!(error(&last)["details"]["reason"], "last_shortcut");
    let alternative = call(&Live(&live), "add", &["cancel", "f17"], false);
    assert_eq!(
        error(&alternative)["details"]["reason"],
        "alternatives_unsupported"
    );
    let missing = call(&Live(&live), "remove", &["transcribe", "f19"], false);
    assert_eq!(missing.exit_code(), 4);
    Ok(())
}

#[test]
fn remove_finds_the_stored_entry_in_any_modifier_order_and_promotes_the_next() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    call(&Live(&live), "add", &["transcribe", "ctrl+alt+d"], false);
    let first = default_of("transcribe")?;
    let removed = call(&Live(&live), "remove", &["transcribe", &first], false);
    assert_eq!(
        removed.envelope()["data"]["shortcuts"],
        json!(["ctrl+alt+d"])
    );
    call(&Live(&live), "add", &["transcribe", "f13"], false);
    let reordered = call(&Live(&live), "remove", &["transcribe", "alt+ctrl+d"], false);
    assert_eq!(reordered.exit_code(), 0, "{}", reordered.envelope());
    assert_eq!(reordered.envelope()["data"]["shortcuts"], json!(["f13"]));
    let stored = SettingsRepository::new(store(&directory)).read_snapshot()?;
    let binding = &stored.snapshot.settings.bindings["transcribe"];
    assert_eq!(
        (
            binding.current_binding.as_str(),
            binding.additional_bindings.len()
        ),
        ("f13", 0)
    );
    Ok(())
}

#[test]
fn reset_restores_the_single_default_and_a_no_op_keeps_the_revision() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    call(&Live(&live), "add", &["transcribe", "f13"], false);
    let revision = live.snapshot().revision.0;
    let reset = call(&Live(&live), "reset", &["transcribe"], false);
    assert_eq!(reset.envelope()["revision"], json!(revision + 1));
    assert_eq!(
        reset.envelope()["data"]["shortcuts"],
        json!([default_of("transcribe")?])
    );
    let again = call(&Live(&live), "reset", &["transcribe"], false);
    assert_eq!(again.envelope()["revision"], json!(revision + 1));
    Ok(())
}

#[test]
fn changes_need_a_running_instance_but_a_dry_run_previews_anywhere() -> TestResult {
    let directory = tempfile::tempdir()?;
    let offline = service(&directory)?;
    let refused = call(&offline, "add", &["transcribe", "f13"], false);
    assert_eq!(refused.exit_code(), 5);
    assert_eq!(error(&refused)["details"]["reason"], "instance_required");
    let preview = call(&offline, "add", &["transcribe", "f13"], true);
    assert_eq!(preview.exit_code(), 0);
    let data = &preview.envelope()["data"];
    assert_eq!(
        (
            data["dry_run"].clone(),
            data["registration_checked"].clone()
        ),
        (json!(true), json!(false))
    );
    assert_eq!(data["changes"]["shortcuts"][1], "f13");
    let invalid = call(&offline, "add", &["transcribe", "CANARY+++"], true);
    assert_eq!(error(&invalid)["details"]["reason"], "invalid_shortcut");
    assert!(!invalid.envelope().to_string().contains("CANARY"));
    assert!(
        !store(&directory).exists(),
        "a refusal or a dry run wrote the store"
    );
    Ok(())
}

#[test]
fn a_stale_expected_revision_changes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = service(&directory)?;
    call(&Live(&live), "add", &["transcribe", "f13"], false);
    let current = live.snapshot().revision.0;
    let stale = call_at(
        &Live(&live),
        "add",
        &["transcribe", "f14"],
        false,
        Some(current - 1),
    );
    assert_eq!(stale.exit_code(), 3);
    assert_eq!(error(&stale)["details"]["current_revision"], json!(current));
    let fresh = call_at(
        &Live(&live),
        "add",
        &["transcribe", "f14"],
        false,
        Some(current),
    );
    assert_eq!(fresh.exit_code(), 0);
    Ok(())
}
