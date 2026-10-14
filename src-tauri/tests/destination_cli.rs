//! `return_to_dictation_app` through the settings CLI (006 AC-008) and the shared validators:
//! the real binary reads and sets it offline and routed with one JSON envelope on stdout, no-op
//! keeps the revision, a stale revision and an invalid type are refused, and a build without the
//! destination capability refuses `true` and `false` alike with `validation_failed`/2 and
//! `details.reason = "unsupported_platform"` on the CLI, domain and UI validation paths.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use handy_app_lib::settings_service::{
    domains::{execute, output::OutputDomain, DomainCall},
    errors::{ErrorReason, SettingsErrorCode},
    ipc,
    live::LiveSettings,
    ownership::LegacyPathProbe,
    platform_capability::{PlatformCapabilities, PlatformCapability},
    store::SettingsRepository,
    validation::{validate_cli, validate_ui, ValidationContext},
};
use harness::{Fallible, Portable, TestResult};
use serde_json::{json, Value};
use std::sync::Arc;

const KEY: &str = "return_to_dictation_app";

fn supported() -> bool {
    PlatformCapabilities::current().supports(PlatformCapability::Destination)
}
fn set(app: &Portable, value: &str, extra: &[&str]) -> Fallible<harness::Run> {
    let mut args = vec!["settings", "output", "set", KEY, value, "--json"];
    args.extend_from_slice(extra);
    app.run(&args)
}

#[test]
fn output_get_reads_the_plain_value_with_the_platform_default() -> TestResult {
    let app = Portable::new()?;
    let read = app.run(&["settings", "output", "get", KEY, "--json"])?;
    let expected = json!({"ok": true, "revision": 0, "data": {KEY: supported()}});
    assert_eq!((read.code, read.envelope()?), (0, expected));
    assert_eq!(read.stdout.lines().count(), 1);
    let generic = app.run(&["settings", "get", KEY])?;
    assert_eq!(generic.stdout, format!("{}\n", supported()));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
#[test]
fn offline_set_commits_once_and_a_no_op_keeps_the_revision() -> TestResult {
    if !supported() {
        return refused_offline();
    }
    let app = Portable::new()?;
    let off = set(&app, "false", &[])?;
    let expected = json!({"ok": true, "revision": 1, "data": {KEY: false}});
    assert_eq!(
        (off.code, off.envelope()?, off.stderr.as_str()),
        (0, expected, "")
    );
    let again = set(&app, "false", &[])?;
    assert_eq!(again.envelope()?["revision"], json!(1));
    let stale = set(&app, "true", &["--expected-revision", "0"])?;
    assert_eq!(stale.code, 3);
    assert_eq!(
        stale.envelope()?["error"]["details"]["current_revision"],
        json!(1)
    );
    Ok(())
}
/// On a build without the capability (Linux), both values are refused and nothing is written.
fn refused_offline() -> TestResult {
    let app = Portable::new()?;
    for value in ["true", "false"] {
        let refused = set(&app, value, &[])?;
        let details = &refused.envelope()?["error"]["details"];
        assert_eq!(
            (refused.code, &details["reason"]),
            (2, &json!("unsupported_platform"))
        );
        assert_eq!(details["field"], json!(KEY));
    }
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
#[test]
fn an_invalid_value_is_a_canonical_type_refusal_before_the_platform_check() -> TestResult {
    let app = Portable::new()?;
    let refused = set(&app, "yes", &[])?;
    let details = &refused.envelope()?["error"]["details"];
    assert_eq!(refused.code, 2);
    assert_eq!(details["field"], json!(KEY));
    assert!(details.get("reason").is_none());
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
#[test]
fn a_set_routed_to_the_running_instance_is_committed_by_it() -> TestResult {
    if !supported() {
        return Ok(());
    }
    let app = Portable::new()?;
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    let repository = SettingsRepository::new(app.store()?);
    let instance = Arc::new(LiveSettings::start_owned(repository, &probe)?);
    let endpoint = ipc::serve(&app.store()?, instance.clone())?;
    let start = instance.snapshot().revision.0;
    let routed = set(&app, "false", &[])?;
    assert_eq!(routed.envelope()?["revision"], json!(start + 1));
    let public = serde_json::to_value(instance.snapshot().public_settings())?;
    assert_eq!(public[KEY], json!(false));
    endpoint.close();
    Ok(())
}
#[test]
fn without_the_capability_cli_domain_and_ui_validation_refuse_identically() -> TestResult {
    let none = PlatformCapabilities::only(&[]);
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
        platform: none,
    };
    for value in [true, false] {
        let cli = validate_cli(KEY, &value.to_string(), &context)
            .err()
            .ok_or("cli accepted")?;
        let ui = validate_ui(KEY, json!(value), &context)
            .err()
            .ok_or("ui accepted")?;
        assert_eq!(cli, ui);
        assert_eq!(
            (cli.code(), cli.exit_code()),
            (SettingsErrorCode::ValidationFailed, 2)
        );
        assert_eq!(
            (cli.detail_reason(), cli.field()),
            (Some(ErrorReason::UnsupportedPlatform), Some(KEY))
        );
        assert_eq!(
            domain_refusal(none, &value.to_string())?,
            serde_json::to_value(&cli)?
        );
    }
    Ok(())
}
/// The output domain's refusal under `platform`, against a real offline repository.
fn domain_refusal(platform: PlatformCapabilities, value: &str) -> Fallible<Value> {
    let directory = tempfile::tempdir()?;
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = directory.path().join("settings_store.json");
    let backend =
        LiveSettings::start_offline(SettingsRepository::new(store.clone()), Box::new(probe))?;
    let arguments = [KEY.to_owned(), value.to_owned()];
    let call = DomainCall {
        domain: "output",
        action: "set",
        arguments: &arguments,
        input: None,
        dry_run: false,
        expected: None,
        platform,
    };
    let refused = execute(&call, &backend, &[&OutputDomain])
        .err()
        .ok_or("domain accepted")?;
    assert!(!store.exists());
    Ok(serde_json::to_value(refused)?)
}
#[test]
fn with_an_older_handy_running_the_set_is_unavailable_and_writes_nothing() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let refused = set(&app, "false", &[])?;
    let code = &refused.envelope()?["error"]["code"];
    assert_eq!((refused.code, code), (5, &json!("unavailable")));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
