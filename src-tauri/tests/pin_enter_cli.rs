//! `handy settings` for 009 through the real binary on an isolated portable store (AC-004,
//! AC-006): the closed `destination-return` gate keeps `dev` (no `destination` domain,
//! `automatic_enter` unknown, `auto_submit` writable); the open gate exposes `automatic_enter`
//! with the shared validation, refuses `auto_submit` like any unknown key, and answers the
//! live-only `destination` actions with the error precedence usage → platform → liveness, and an
//! unknown outcome when the instance does not answer in time.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use harness::{Portable, TestResult};
use serde_json::json;

// @spec AC-006: CLI pin, unpin, status and Enter keys — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-006
#[cfg(not(feature = "destination-return"))]
mod closed_gate {
    use super::*;

    #[test]
    fn destination_actions_are_a_usage_error_exactly_as_on_dev() -> TestResult {
        let app = Portable::new()?;
        for args in [
            &["settings", "destination", "status", "--json"][..],
            &[
                "settings",
                "destination",
                "pin",
                "ai.perplexity.comet",
                "--json",
            ],
            &["settings", "destination", "unpin", "--json"],
        ] {
            let run = app.run(args)?;
            assert_eq!(
                (run.code, &run.envelope()?["error"]["code"]),
                (2, &json!("usage_error"))
            );
        }
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn automatic_enter_is_unknown_and_auto_submit_commits_as_on_dev() -> TestResult {
        let app = Portable::new()?;
        let unknown = app.run(&["settings", "get", "no_such_key", "--json"])?;
        for args in [
            &[
                "settings",
                "output",
                "set",
                "automatic_enter",
                "always",
                "--json",
            ][..],
            &["settings", "get", "automatic_enter", "--json"],
        ] {
            let run = app.run(args)?;
            assert_eq!(run.code, 4);
            assert_eq!(run.envelope()?, unknown.envelope()?);
        }
        let set = app.run(&["settings", "output", "set", "auto_submit", "true", "--json"])?;
        let expected = json!({"ok": true, "revision": 1, "data": {"auto_submit": true}});
        assert_eq!((set.code, set.envelope()?), (0, expected));
        Ok(())
    }
}

#[cfg(feature = "destination-return")]
mod open_gate {
    use super::*;
    use handy_app_lib::settings_service::platform_capability::{
        PlatformCapabilities, PlatformCapability,
    };

    fn supported() -> bool {
        PlatformCapabilities::current().supports(PlatformCapability::Destination)
    }

    #[test]
    fn destination_status_without_a_running_handy_is_unavailable_or_unsupported() -> TestResult {
        let app = Portable::new()?;
        for args in [
            &["settings", "destination", "status", "--json"][..],
            &["settings", "destination", "unpin", "--json"],
            &[
                "settings",
                "destination",
                "pin",
                "ai.perplexity.comet",
                "--json",
            ],
        ] {
            let run = app.run(args)?;
            let error = run.envelope()?["error"].clone();
            if supported() {
                assert_eq!(
                    (run.code, &error["code"], &error["details"]),
                    (5, &json!("unavailable"), &json!({}))
                );
            } else {
                assert_eq!(
                    (run.code, &error["code"], &error["details"]["reason"]),
                    (
                        2,
                        &json!("validation_failed"),
                        &json!("unsupported_platform")
                    )
                );
            }
            assert_eq!(run.stdout.lines().count(), 1);
        }
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn change_options_are_refused_at_parse_before_any_routing() -> TestResult {
        let app = Portable::new()?;
        for extra in [&["--dry-run"][..], &["--expected-revision", "12"]] {
            for action in [&["pin", "ai.perplexity.comet"][..], &["unpin"], &["status"]] {
                let mut args = vec!["settings", "destination"];
                args.extend_from_slice(action);
                args.extend_from_slice(extra);
                args.push("--json");
                let run = app.run(&args)?;
                assert_eq!(
                    (run.code, &run.envelope()?["error"]["code"]),
                    (2, &json!("usage_error")),
                    "{args:?}"
                );
            }
        }
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn auto_submit_is_refused_like_any_unknown_key_and_nothing_is_written() -> TestResult {
        let app = Portable::new()?;
        let unknown = app.run(&["settings", "set", "no_such_key", "true", "--json"])?;
        for args in [
            &["settings", "set", "auto_submit", "true", "--json"][..],
            &["settings", "get", "auto_submit", "--json"],
            &["settings", "output", "set", "auto_submit", "true", "--json"],
            &["settings", "output", "get", "auto_submit", "--json"],
        ] {
            let run = app.run(args)?;
            assert_eq!(run.code, 4, "{args:?}");
            assert_eq!(run.envelope()?["error"], unknown.envelope()?["error"]);
            assert!(!run.stdout.contains("auto_submit\""));
        }
        assert_eq!(app.store_bytes()?, None);
        Ok(())
    }
    #[test]
    fn automatic_enter_has_the_shared_validation_and_commit() -> TestResult {
        let app = Portable::new()?;
        let fresh = app.run(&["settings", "output", "get", "automatic_enter", "--json"])?;
        let expected = json!({"ok": true, "revision": 0, "data": {"automatic_enter": "never"}});
        assert_eq!(fresh.envelope()?, expected);
        let invalid = app.run(&[
            "settings",
            "output",
            "set",
            "automatic_enter",
            "sometimes",
            "--json",
        ])?;
        let error = invalid.envelope()?["error"].clone();
        assert_eq!(
            (invalid.code, &error["code"]),
            (2, &json!("validation_failed"))
        );
        assert_eq!(
            error["details"]["issues"][0]["reason"],
            json!("Expected never, always or pin_only.")
        );
        assert_eq!(app.store_bytes()?, None, "no revision change");
        let set = app.run(&[
            "settings",
            "output",
            "set",
            "automatic_enter",
            "pin_only",
            "--json",
        ])?;
        let expected = json!({"ok": true, "revision": 1, "data": {"automatic_enter": "pin_only"}});
        assert_eq!((set.code, set.envelope()?), (0, expected));
        let key = app.run(&["settings", "output", "set", "auto_submit_key", "cmd_enter"])?;
        assert_eq!(key.code, 0);
        Ok(())
    }
    #[test]
    fn keys_and_keyless_reads_list_automatic_enter_instead_of_auto_submit() -> TestResult {
        let app = Portable::new()?;
        let keys = app.run(&["settings", "keys", "--json"])?.envelope()?;
        let listed = keys["data"].to_string();
        assert!(listed.contains("\"automatic_enter\""));
        assert!(!listed.contains("\"auto_submit\""));
        for args in [
            &["settings", "get", "--json"][..],
            &["settings", "output", "get", "--json"],
        ] {
            let data = app.run(args)?.envelope()?["data"].clone();
            assert_eq!(data["automatic_enter"], json!("never"));
            assert!(data.get("auto_submit").is_none());
        }
        Ok(())
    }
}

// @spec AC-004: An older Handy running keeps its store untouched — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-004
#[test]
fn with_an_older_handy_running_the_enter_setting_is_never_written_behind_it() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let key = if cfg!(feature = "destination-return") {
        ["automatic_enter", "always"]
    } else {
        ["auto_submit", "true"]
    };
    let run = app.run(&["settings", "output", "set", key[0], key[1], "--json"])?;
    assert_eq!(
        (run.code, &run.envelope()?["error"]["code"]),
        (5, &json!("unavailable"))
    );
    assert!(!run.stderr.contains(key[1]), "no value echoed on stderr");
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}

/// A running instance that answers only after the CLI deadline (open gate).
#[cfg(feature = "destination-return")]
mod timeout {
    use super::*;
    use handy_app_lib::settings_service::{
        errors::SettingsError,
        ipc,
        live::LiveSettings,
        operations::SettingsBackend,
        ownership::LegacyPathProbe,
        protocol::Revision,
        store::{SettingsRead, SettingsRepository},
        validation::SettingsMutation,
    };
    use std::sync::Arc;
    use std::time::Duration;

    struct Stalled(Arc<LiveSettings>);
    impl SettingsBackend for Stalled {
        fn read(&self) -> Result<SettingsRead, SettingsError> {
            std::thread::sleep(ipc::DEADLINE + Duration::from_millis(500));
            self.0.read()
        }
        fn commit(
            &self,
            mutation: SettingsMutation,
            expected: Option<Revision>,
        ) -> Result<SettingsRead, SettingsError> {
            self.0.commit(mutation, expected)
        }
    }

    #[test]
    fn a_late_instance_is_an_unknown_outcome_without_retry() -> TestResult {
        let app = Portable::new()?;
        let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
        let instance = Arc::new(LiveSettings::start_owned(
            SettingsRepository::new(app.store()?),
            &probe,
        )?);
        let endpoint = ipc::serve(&app.store()?, Arc::new(Stalled(instance)))?;
        let run = app.run(&["settings", "destination", "unpin", "--json"])?;
        let error = run.envelope()?["error"].clone();
        assert_eq!(
            (run.code, &error["code"], &error["details"]["outcome"]),
            (5, &json!("unavailable"), &json!("unknown"))
        );
        endpoint.close();
        Ok(())
    }
}
