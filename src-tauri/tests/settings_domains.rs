//! Domain framework through the real binary: `handy settings --help` lists exactly the domains of
//! this build (`output`, `replacements` tested in `replacement_cli`, and `shortcuts` from feature
//! 004), and the output domain reads and changes the existing output settings with the
//! validation, dry run, revision and channels of the settings contract, offline and routed to a
//! running instance.
#![cfg(unix)]

#[path = "settings_cli/harness.rs"]
mod harness;

use handy_app_lib::settings_service::{
    domains::output::OUTPUT_KEYS, ipc, live::LiveSettings, ownership::LegacyPathProbe,
    store::SettingsRepository,
};
use harness::{Fallible, Portable, TestResult};
use serde_json::json;
use std::sync::Arc;

/// A running Handy holds the startup lease (stood in for by an owned live service).
fn running_instance(app: &Portable) -> Fallible<LiveSettings> {
    let probe = LegacyPathProbe::new(app.root.path().join("absent.sock"));
    Ok(LiveSettings::start_owned(
        SettingsRepository::new(app.store()?),
        &probe,
    )?)
}
/// Command names of a clap help text (the indented first words of its "Commands:" section).
fn commands(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|line| !line.starts_with("Commands:"))
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .collect()
}
#[test]
fn help_lists_the_generic_operations_and_exactly_the_domains_of_this_build() -> TestResult {
    let app = Portable::new()?;
    let help = app.run(&["settings", "--help"])?;
    assert_eq!(help.code, 0);
    assert_eq!(
        commands(&help.stdout),
        [
            "get",
            "keys",
            "set",
            "output",
            "replacements",
            "shortcuts",
            "vocabulary",
            "microphones",
            "help"
        ]
    );
    let microphones = app.run(&["settings", "microphones", "--help"])?;
    assert_eq!(
        commands(&microphones.stdout),
        ["list", "set-priority", "help"]
    );
    let output = app.run(&["settings", "output", "--help"])?;
    assert_eq!(commands(&output.stdout), ["get", "set", "help"]);
    let shortcuts = app.run(&["settings", "shortcuts", "--help"])?;
    assert_eq!(
        commands(&shortcuts.stdout),
        ["list", "add", "remove", "reset", "help"]
    );
    Ok(())
}
#[test]
fn output_get_reads_exactly_the_output_settings_without_creating_the_store() -> TestResult {
    let app = Portable::new()?;
    let all = app
        .run(&["settings", "output", "get", "--json"])?
        .envelope()?;
    let keys = all["data"]
        .as_object()
        .map(|data| data.keys().cloned().collect::<Vec<_>>());
    let mut expected = OUTPUT_KEYS.map(str::to_owned).to_vec();
    expected.sort();
    assert_eq!(keys, Some(expected));
    let one = app.run(&["settings", "output", "get", "paste_method"])?;
    assert_eq!((one.code, one.stdout.lines().count()), (0, 1));
    let other = app.run(&["settings", "output", "get", "theme", "--json"])?;
    assert_eq!(
        (other.code, &other.envelope()?["error"]["code"]),
        (4, &json!("not_found"))
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
const DRY_RUN: [&str; 7] = [
    "settings",
    "output",
    "set",
    "paste_method",
    "direct",
    "--dry-run",
    "--json",
];
#[test]
fn output_dry_run_previews_with_exit_zero_and_never_writes() -> TestResult {
    let app = Portable::new()?;
    app.run(&["settings", "set", "auto_submit", "true"])?;
    let before = app.store_bytes()?;
    let preview = app.run(&DRY_RUN)?.envelope()?;
    let expected = json!({"dry_run": true, "changes": {"paste_method": "direct"}});
    assert_eq!(
        (&preview["revision"], &preview["data"]),
        (&json!(1), &expected)
    );
    let human = app.run(&DRY_RUN[..6])?;
    assert!(human
        .stdout
        .starts_with("Dry run: nothing was changed (revision 1)"));
    assert_eq!(app.store_bytes()?, before);
    Ok(())
}
#[test]
fn invalid_output_dry_run_is_refused_with_exit_two_and_no_preview() -> TestResult {
    let app = Portable::new()?;
    let mut args = DRY_RUN;
    args[4] = "invalid_method";
    let invalid = app.run(&args)?;
    let envelope = invalid.envelope()?;
    assert_eq!(
        (invalid.code, &envelope["error"]["code"]),
        (2, &json!("validation_failed"))
    );
    assert!(envelope.get("data").is_none());
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
#[test]
fn output_set_commits_through_the_shared_path() -> TestResult {
    let app = Portable::new()?;
    let set = app.run(&[
        "settings",
        "output",
        "set",
        "paste_delay_ms",
        "120",
        "--json",
    ])?;
    let expected = json!({"ok": true, "revision": 1, "data": {"paste_delay_ms": 120}});
    assert_eq!(set.envelope()?, expected);
    let generic = app.run(&["settings", "get", "paste_delay_ms"])?;
    assert_eq!(generic.stdout, "120\n");
    let foreign = app.run(&["settings", "output", "set", "theme", "dark"])?;
    assert_eq!(foreign.code, 4);
    Ok(())
}
#[test]
fn stale_output_set_is_a_revision_conflict_reported_on_stderr() -> TestResult {
    let app = Portable::new()?;
    app.run(&["settings", "output", "set", "paste_delay_ms", "120"])?;
    let args = [
        "settings",
        "output",
        "set",
        "paste_delay_ms",
        "130",
        "--expected-revision",
        "0",
    ];
    let stale = app.run(&args)?;
    assert_eq!(stale.code, 3);
    assert!(stale.stdout.is_empty() && stale.stderr.contains("current revision: 1"));
    Ok(())
}
#[test]
fn output_set_routed_to_a_running_instance_is_committed_by_the_instance() -> TestResult {
    let app = Portable::new()?;
    let instance = Arc::new(running_instance(&app)?);
    let endpoint = ipc::serve(&app.store()?, instance.clone())?;
    let start = instance.snapshot().revision.0;
    let set = app.run(&[
        "settings",
        "output",
        "set",
        "clipboard_handling",
        "copy_to_clipboard",
        "--json",
    ])?;
    assert_eq!(set.envelope()?["revision"], json!(start + 1));
    let public = serde_json::to_value(instance.snapshot().public_settings())?;
    assert_eq!(public["clipboard_handling"], json!("copy_to_clipboard"));
    endpoint.close();
    Ok(())
}
#[test]
fn output_set_with_an_older_handy_running_is_unavailable_and_writes_nothing() -> TestResult {
    let app = Portable::new()?;
    let _older = std::os::unix::net::UnixListener::bind(app.legacy_socket())?;
    let refused = app.run(&["settings", "output", "set", "auto_submit", "true", "--json"])?;
    assert_eq!(
        (refused.code, &refused.envelope()?["error"]["code"]),
        (5, &json!("unavailable"))
    );
    assert_eq!(app.store_bytes()?, None);
    Ok(())
}
