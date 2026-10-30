//! "Show the bubble when no text field is focused" through 008's settings service (010
//! AC-007, plan Step 3), on the real binary with an isolated store and on the shared
//! validators: gate closed it does not exist (`not_found`/4, absent from `get-all`, a value
//! stored by an open build is preserved); gate open on macOS it defaults to `true`, commits
//! once, keeps the revision on a no-op and refuses invalid types and stale revisions; without
//! field detection (Windows, Linux, injected here) reads are `false` and every write is refused
//! with `unsupported_platform` by the CLI, the domain and the UI validation alike.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use handy_app_lib::recovery::setting::{normalized, SettingBuild, KEY};
use harness::{Fallible, Portable, TestResult};
use serde_json::{json, Value};

fn public_keys(app: &Portable) -> Fallible<Value> {
    Ok(app.run(&["settings", "get", "--json"])?.envelope()?["data"].clone())
}

// @spec AC-007: The setting through the settings CLI — .specs/features/010-recovery-bubble/spec.md#ac-007
#[test]
fn off_macos_the_value_reads_false_and_open_macos_keeps_the_choice() {
    let windows_linux = SettingBuild {
        wired: true,
        macos: false,
    };
    for stored in [None, Some(true), Some(false)] {
        assert_eq!(normalized(stored, windows_linux), Some(false));
    }
    let mac = SettingBuild {
        wired: true,
        macos: true,
    };
    assert_eq!(normalized(None, mac), Some(true));
    assert_eq!(normalized(Some(false), mac), Some(false));
}

/// A write while an older Handy (pre-008) owns the store is unavailable and writes nothing, in
/// both gate states (the ownership check comes first).
#[test]
fn a_write_with_an_older_handy_running_writes_nothing() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let refused = app.run(&["settings", "output", "set", KEY, "false", "--json"])?;
    let code = &refused.envelope()?["error"]["code"];
    assert_eq!((refused.code, code), (5, &json!("unavailable")));
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

#[cfg(not(feature = "destination-return"))]
mod closed {
    use super::*;

    #[test]
    fn the_key_does_not_exist_for_any_command() -> TestResult {
        let app = Portable::new()?;
        for args in [
            vec!["settings", "get", KEY, "--json"],
            vec!["settings", "set", KEY, "false", "--json"],
            vec!["settings", "output", "get", KEY, "--json"],
            vec!["settings", "output", "set", KEY, "false", "--json"],
        ] {
            let run = app.run(&args)?;
            // The diagnostic goes to stderr; stdout holds the JSON envelope only.
            assert!(run.stderr.contains("Unknown setting key"), "{args:?}");
            let code = &run.envelope()?["error"]["code"];
            assert_eq!((run.code, code), (4, &json!("not_found")), "{args:?}");
        }
        assert!(public_keys(&app)?.get(KEY).is_none());
        let output = app
            .run(&["settings", "output", "get", "--json"])?
            .envelope()?;
        assert!(output["data"].get(KEY).is_none());
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn a_value_stored_by_an_open_build_survives_an_unrelated_closed_write() -> TestResult {
        let app = Portable::new()?;
        let stored = json!({"settings_revision": 3, "settings": {KEY: false}});
        std::fs::write(app.store()?, serde_json::to_vec(&stored)?)?;
        assert!(public_keys(&app)?.get(KEY).is_none());
        let write = app.run(&["settings", "set", "append_trailing_space", "true", "--json"])?;
        assert_eq!(write.code, 0);
        let raw: Value = serde_json::from_slice(&std::fs::read(app.store()?)?)?;
        assert_eq!(raw["settings"][KEY], json!(false));
        Ok(())
    }
}

#[cfg(all(feature = "destination-return", target_os = "macos"))]
mod open_macos {
    use super::*;

    fn set(app: &Portable, value: &str, extra: &[&str]) -> Fallible<harness::Run> {
        let mut args = vec!["settings", "output", "set", KEY, value, "--json"];
        args.extend_from_slice(extra);
        app.run(&args)
    }
    #[test]
    fn the_default_is_true_and_reads_never_write() -> TestResult {
        let app = Portable::new()?;
        let read = app.run(&["settings", "output", "get", KEY, "--json"])?;
        let expected = json!({"ok": true, "revision": 0, "data": {KEY: true}});
        assert_eq!((read.code, read.envelope()?), (0, expected));
        assert_eq!(read.stdout.lines().count(), 1, "one JSON envelope only");
        assert_eq!(public_keys(&app)?[KEY], json!(true));
        let generic = app.run(&["settings", "get", KEY])?;
        assert_eq!(generic.stdout, "true\n");
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn a_change_commits_once_a_no_op_keeps_the_revision_and_stale_or_invalid_writes_are_refused(
    ) -> TestResult {
        let app = Portable::new()?;
        let off = set(&app, "false", &[])?;
        let expected = json!({"ok": true, "revision": 1, "data": {KEY: false}});
        assert_eq!(
            (off.code, off.envelope()?, off.stderr.as_str()),
            (0, expected, "")
        );
        assert_eq!(set(&app, "false", &[])?.envelope()?["revision"], json!(1));
        let stale = set(&app, "true", &["--expected-revision", "0"])?;
        assert_eq!(stale.code, 3);
        assert_eq!(
            stale.envelope()?["error"]["details"]["current_revision"],
            json!(1)
        );
        for invalid in ["yes", "null", "1"] {
            let refused = set(&app, invalid, &[])?;
            let error = &refused.envelope()?["error"];
            assert_eq!(
                (refused.code, &error["code"]),
                (2, &json!("validation_failed"))
            );
            assert_eq!(error["details"]["field"], json!(KEY));
        }
        let generic = app.run(&["settings", "set", KEY, "true", "--json"])?;
        assert_eq!(generic.envelope()?["revision"], json!(2));
        Ok(())
    }
}

#[cfg(feature = "destination-return")]
mod without_field_detection {
    use super::*;
    use handy_app_lib::settings_service::{
        errors::{ErrorReason, SettingsErrorCode},
        platform_capability::{PlatformCapabilities, PlatformCapability},
        validation::{validate_cli, validate_ui, ValidationContext},
    };

    #[test]
    fn cli_and_ui_validation_refuse_every_write_with_the_platform_reason() -> TestResult {
        // Windows and Linux builds declare `Destination` (Windows) but never field detection.
        for supported in [&[PlatformCapability::Destination][..], &[]] {
            let context = ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
                platform: PlatformCapabilities::only(supported),
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
                assert_eq!(cli.detail_reason(), Some(ErrorReason::UnsupportedPlatform));
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn this_build_reads_false_and_refuses_writes() -> TestResult {
        let app = Portable::new()?;
        assert_eq!(public_keys(&app)?[KEY], json!(false));
        let refused = app.run(&["settings", "output", "set", KEY, "true", "--json"])?;
        let details = &refused.envelope()?["error"]["details"];
        assert_eq!(
            (refused.code, &details["reason"]),
            (2, &json!("unsupported_platform"))
        );
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
}
