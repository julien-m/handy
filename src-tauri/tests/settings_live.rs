//! The running app's write path (settings UI adapter over the live service) against real stores:
//! same decisions and stored values as the CLI path, one revision per commit under concurrency,
//! and startup continuity with offline writers.
use handy_app_lib::settings_service::{
    errors::SettingsError,
    live::LiveSettings,
    ownership::LegacyPathProbe,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// (key, typed value the UI command serializes, equivalent CLI text): accepted values. Device
/// selectors: the UI "Default" entry (null) and the CLI `null` both mean the system default, and
/// "default" is a literal device name on both surfaces.
fn accepted_cases() -> Vec<(&'static str, Value, &'static str)> {
    vec![
        ("paste_method", json!("ctrl_v"), "ctrl_v"),
        ("sound_theme", json!("pop"), "pop"),
        ("theme", json!("dark"), "dark"),
        ("overlay_position", json!("top"), "top"),
        ("keyboard_implementation", json!("handy_keys"), "handy_keys"),
        ("log_level", json!("warn"), "warn"),
        ("hold_threshold_ms", json!(250), "250"),
        ("paste_delay_ms", json!(4999), "4999"),
        ("audio_feedback", json!(true), "true"),
        (
            "external_script_path",
            json!("/opt/paste.sh"),
            "/opt/paste.sh",
        ),
        ("external_script_path", Value::Null, "null"),
        ("recording_retention_period", json!("weeks2"), "weeks2"),
        ("selected_microphone", Value::Null, "null"),
        ("clamshell_microphone", Value::Null, "null"),
        ("selected_output_device", Value::Null, "null"),
        ("selected_microphone", json!("default"), "default"),
        (
            "selected_microphone",
            json!("USB Microphone"),
            "USB Microphone",
        ),
    ]
}
/// Refused values: formerly replaced by defaults in UI commands, now refused on both surfaces.
fn refused_cases() -> Vec<(&'static str, Value, &'static str)> {
    vec![
        ("paste_method", json!("invalid_method"), "invalid_method"),
        ("sound_theme", json!("theremin"), "theremin"),
        ("theme", json!("sepia"), "sepia"),
        ("overlay_position", json!("none"), "none"),
        ("keyboard_implementation", json!("rdev"), "rdev"),
        ("log_level", json!("verbose"), "verbose"),
        ("hold_threshold_ms", json!(99), "99"),
        ("history_limit", json!(1001), "1001"),
        ("audio_feedback_volume", json!(1.5), "1.5"),
        ("recording_retention_period", json!("forever"), "forever"),
        ("transcribe_accelerator", json!("gpu"), "gpu"),
        (
            "post_process_selected_prompt_id",
            json!("missing"),
            "missing",
        ),
        ("selected_model", json!("whisper-small"), "whisper-small"),
    ]
}
fn store(directory: &tempfile::TempDir) -> PathBuf {
    directory.path().join("settings_store.json")
}
fn absent(directory: &tempfile::TempDir) -> LegacyPathProbe {
    LegacyPathProbe::new(directory.path().join("absent.sock"))
}
/// Decision and stored public settings of one path, comparable across paths.
fn outcome(
    result: Result<(), SettingsError>,
    path: PathBuf,
) -> Result<(Value, Value), Box<dyn std::error::Error>> {
    let decision = match result {
        Ok(()) => json!("committed"),
        Err(error) => serde_json::to_value(error)?,
    };
    let read = SettingsRepository::new(path).read_snapshot()?;
    let mut public = serde_json::to_value(read.snapshot.public_settings())?;
    public["revision"] = json!(read.snapshot.revision.0);
    Ok((decision, public))
}
fn ui_path(key: &str, value: Value) -> Result<(Value, Value), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let live = LiveSettings::start_owned(
        SettingsRepository::new(store(&directory)),
        &absent(&directory),
    )?;
    let result = live.commit_ui_value(key, value).map(|_| ());
    drop(live);
    outcome(result, store(&directory))
}
fn cli_path(key: &str, text: &str) -> Result<(Value, Value), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let repository = SettingsRepository::new(store(&directory));
    let (_, lease) = repository.initialize(&absent(&directory))?;
    drop(lease);
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
    };
    let result = validate_cli(key, text, &context).and_then(|mutation| {
        repository
            .commit_with_probe(mutation, None, &absent(&directory), &())
            .map(|_| ())
    });
    outcome(result, store(&directory))
}
#[test]
fn ui_adapter_and_cli_store_the_same_value_or_refuse_with_the_same_error() -> TestResult {
    for (key, ui_value, cli_text) in [accepted_cases(), refused_cases()].concat() {
        let ui = ui_path(key, ui_value)?;
        let cli = cli_path(key, cli_text)?;
        assert_eq!(ui, cli, "UI and CLI diverge for {key}={cli_text}");
    }
    Ok(())
}
/// Distinct boolean keys written concurrently, one thread each.
const CONCURRENT_KEYS: [&str; 8] = [
    "audio_feedback",
    "start_hidden",
    "translate_to_english",
    "auto_submit",
    "mute_while_recording",
    "append_trailing_space",
    "experimental_enabled",
    "filler_word_removal_enabled",
];
fn owned(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    LiveSettings::start_owned(
        SettingsRepository::new(store(directory)),
        &absent(directory),
    )
}
#[test]
fn concurrent_ui_commits_lose_no_field_and_increment_once_each() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = Arc::new(owned(&directory)?);
    let start = live.snapshot().revision.0;
    let before = serde_json::to_value(live.snapshot().public_settings())?;
    let writers = CONCURRENT_KEYS.map(|key| {
        let live = Arc::clone(&live);
        let value = !before[key].as_bool().unwrap_or(false);
        std::thread::spawn(move || live.commit_ui_value(key, json!(value)).map(|_| ()))
    });
    for writer in writers {
        writer.join().map_err(|_| "writer thread panicked")??;
    }
    let reread = SettingsRepository::new(store(&directory)).read_snapshot()?;
    assert_eq!(
        reread.snapshot.revision.0,
        start + CONCURRENT_KEYS.len() as u64
    );
    let after = serde_json::to_value(reread.snapshot.public_settings())?;
    for key in CONCURRENT_KEYS {
        assert_ne!(after[key], before[key], "{key} lost");
    }
    Ok(())
}
/// Offline (CLI-like) commit of `paste_delay_ms`, optionally compare-and-set.
fn offline_delay(
    directory: &tempfile::TempDir,
    value: &str,
    expected: Option<u64>,
) -> Result<u64, SettingsError> {
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
    };
    let mutation = validate_cli("paste_delay_ms", value, &context)?;
    let expected = expected.map(handy_app_lib::settings_service::protocol::Revision);
    SettingsRepository::new(store(directory))
        .commit_with_probe(mutation, expected, &absent(directory), &())
        .map(|read| read.snapshot.revision.0)
}
#[test]
fn instance_fences_offline_writers_and_a_restart_sees_their_later_commit() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let first = live
        .commit_ui_value("paste_delay_ms", json!(120))?
        .revision
        .0;
    let fenced = offline_delay(&directory, "130", None)
        .err()
        .ok_or("offline write accepted while the instance runs")?;
    assert_eq!(serde_json::to_value(&fenced)?["code"], "unavailable");
    drop(live);
    assert_eq!(offline_delay(&directory, "130", Some(first))?, first + 1);
    let snapshot = owned(&directory)?.snapshot();
    assert_eq!(snapshot.revision.0, first + 1);
    let public = serde_json::to_value(snapshot.public_settings())?;
    assert_eq!(public["paste_delay_ms"], 130);
    Ok(())
}
#[test]
fn device_selectors_store_null_for_the_default_and_literal_names_otherwise() -> TestResult {
    for key in [
        "selected_microphone",
        "clamshell_microphone",
        "selected_output_device",
    ] {
        let (_, ui) = ui_path(key, Value::Null)?;
        let (_, cli) = cli_path(key, "null")?;
        assert!(ui[key].is_null() && cli[key].is_null(), "{key}");
        let (_, ui) = ui_path(key, json!("default"))?;
        let (_, cli) = cli_path(key, "default")?;
        assert_eq!(
            (&ui[key], &cli[key]),
            (&json!("default"), &json!("default"))
        );
    }
    Ok(())
}
