//! Replacement rules through the actual settings service on temporary stores: a CLI-routed commit
//! is used by the next dictation (never by one already recording), rules survive a restart in
//! order, a legacy store starts with no rules and keeps its other settings and revision, and a
//! no-op import writes nothing.

use std::sync::Arc;

use handy_app_lib::settings_service::{
    live::LiveSettings,
    operations::{execute, Reply},
    ownership::LegacyPathProbe,
    protocol::SettingsOperation,
    store::SettingsRepository,
};
use handy_app_lib::word_replacements::{
    dictation::DictationRuleCapture, matcher::CompiledReplacements, model::ReplacementRule,
    published::PublishedReplacements,
};
use serde_json::{json, Value};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

fn store(directory: &tempfile::TempDir) -> std::path::PathBuf {
    directory.path().join("settings_store.json")
}
/// The running instance (an owned live service), as after an app start.
fn instance(directory: &tempfile::TempDir) -> Fallible<LiveSettings> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    Ok(LiveSettings::start_owned(
        SettingsRepository::new(store(directory)),
        &probe,
    )?)
}
/// A replacements operation answered by `live`, as for a CLI call routed to it.
fn routed(live: &LiveSettings, action: &str, arguments: &[&str], input: Option<&str>) -> Reply {
    let operation = SettingsOperation::Domain {
        domain: "replacements".into(),
        action: action.into(),
        arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
        input: input.map(str::to_owned),
        dry_run: false,
        expected_revision: None,
    };
    execute(&operation, live)
}
/// What a dictation starting now captures: the matcher of the published revision.
fn matcher_now(
    live: &LiveSettings,
    published: &PublishedReplacements,
) -> Arc<CompiledReplacements> {
    let snapshot = live.snapshot();
    let rules: Vec<ReplacementRule> = serde_json::to_value(snapshot.public_settings())
        .ok()
        .and_then(|public| serde_json::from_value(public["word_replacements"].clone()).ok())
        .unwrap_or_default();
    published.matcher_for(snapshot.revision.0, &rules)
}

#[test]
fn a_cli_commit_applies_from_the_next_dictation_not_the_current_one() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = instance(&directory)?;
    let published = PublishedReplacements::default();
    routed(&live, "add", &["cloud code", "Claude Code"], None);
    let recording = matcher_now(&live, &published);
    let reply = routed(
        &live,
        "import",
        &["merge"],
        Some("cloud code;Claude Code CLI\n"),
    );
    assert_eq!(reply.exit_code(), 0);
    assert_eq!(
        recording.apply("open cloud code now").text(),
        "open Claude Code now"
    );
    let next = matcher_now(&live, &published);
    assert_eq!(
        next.apply("open cloud code now").text(),
        "open Claude Code CLI now"
    );
    Ok(())
}

#[test]
fn the_stop_path_keeps_the_matcher_captured_at_recording_start() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = instance(&directory)?;
    let published = PublishedReplacements::default();
    routed(&live, "add", &["get hub", "GitHub"], None);
    let capture = DictationRuleCapture::default();
    capture.capture("transcribe", matcher_now(&live, &published));
    routed(&live, "add", &["cloud code", "Claude Code"], None);
    let stopped = capture.rules_for_stop("transcribe", || matcher_now(&live, &published));
    let text = stopped.apply("get hub and cloud code").text();
    assert_eq!(
        text, "GitHub and cloud code",
        "the snapshot captured at start"
    );
    Ok(())
}

#[test]
fn rules_survive_a_restart_in_their_order() -> TestResult {
    let directory = tempfile::tempdir()?;
    let input = "zeta;Z\nalpha;A\nmid;M\n";
    {
        let live = instance(&directory)?;
        assert_eq!(
            routed(&live, "import", &["replace"], Some(input)).exit_code(),
            0
        );
    }
    let restarted = instance(&directory)?;
    let listed = routed(&restarted, "export", &[], None);
    let content = &listed.envelope()["data"]["content"];
    assert_eq!(content, &json!(format!("recognized;replacement\n{input}")));
    Ok(())
}

#[test]
fn a_legacy_store_starts_without_rules_and_keeps_its_settings_and_revision() -> TestResult {
    let directory = tempfile::tempdir()?;
    let legacy = json!({"settings": {"paste_method": "direct", "custom_words": ["Handy"]},
                        "settings_revision": 41});
    std::fs::write(store(&directory), serde_json::to_vec(&legacy)?)?;
    let live = instance(&directory)?;
    let public = serde_json::to_value(live.snapshot().public_settings())?;
    assert_eq!(public["word_replacements"], json!([]));
    assert_eq!(
        (&public["paste_method"], &public["custom_words"]),
        (&json!("direct"), &json!(["Handy"]))
    );
    assert!(live.snapshot().revision.0 >= 41, "never reset");
    Ok(())
}

#[test]
fn a_no_op_import_returns_the_current_revision_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = instance(&directory)?;
    routed(&live, "add", &["a", "b"], None);
    let before = std::fs::read(store(&directory))?;
    let revision = live.snapshot().revision;
    let reply = routed(&live, "import", &["merge"], Some("a;b\n"));
    let envelope: &Value = reply.envelope();
    assert_eq!(
        (&envelope["revision"], &envelope["data"]["unchanged"]),
        (&json!(revision.0), &json!(1))
    );
    assert_eq!(std::fs::read(store(&directory))?, before);
    Ok(())
}
