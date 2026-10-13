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

/// A store the instance cannot own never stops the app: it runs read-only on its best read and
/// refuses every write, leaving the store bytes untouched.
#[cfg(unix)]
mod read_only_startup {
    use super::*;
    use handy_app_lib::settings_service::errors::SettingsErrorCode;
    use std::os::unix::fs::PermissionsExt;

    /// A normal store holding `paste_method = direct`, then released by its instance.
    fn prepared_store(directory: &tempfile::TempDir) -> TestResult {
        let live = owned(directory)?;
        live.commit_ui_value("paste_method", json!("direct"))?;
        Ok(())
    }
    fn start(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
        LiveSettings::start_owned_or_read_only(
            SettingsRepository::new(store(directory)),
            &absent(directory),
        )
    }
    /// Starts read-only, shows `paste_method` as `expected`, refuses a write, changes nothing.
    fn assert_read_only(directory: &tempfile::TempDir, expected: &str) -> TestResult {
        let bytes = std::fs::read(directory.path().join("real.json")).ok();
        let live = start(directory)?;
        assert!(live.read_only() && live.recovered());
        let public = serde_json::to_value(live.snapshot().public_settings())?;
        assert_eq!(public["paste_method"], json!(expected));
        let refused = live
            .commit_ui_value("paste_method", json!("ctrl_v"))
            .err()
            .ok_or("read-only startup accepted a write")?;
        assert_ne!(refused.code(), SettingsErrorCode::ValidationFailed);
        assert_eq!(
            std::fs::read(directory.path().join("real.json")).ok(),
            bytes
        );
        Ok(())
    }
    #[test]
    fn linked_store_starts_read_only_on_its_target() -> TestResult {
        let directory = tempfile::tempdir()?;
        prepared_store(&directory)?;
        std::fs::rename(store(&directory), directory.path().join("real.json"))?;
        std::os::unix::fs::symlink(directory.path().join("real.json"), store(&directory))?;
        assert_read_only(&directory, "direct")
    }
    #[test]
    fn hard_linked_store_starts_read_only_on_its_content() -> TestResult {
        let directory = tempfile::tempdir()?;
        prepared_store(&directory)?;
        std::fs::hard_link(store(&directory), directory.path().join("real.json"))?;
        assert_read_only(&directory, "direct")
    }
    #[test]
    fn unreadable_store_starts_read_only_on_defaults() -> TestResult {
        let directory = tempfile::tempdir()?;
        prepared_store(&directory)?;
        std::fs::set_permissions(store(&directory), std::fs::Permissions::from_mode(0o000))?;
        let fresh = tempfile::tempdir()?;
        let default = serde_json::to_value(owned(&fresh)?.snapshot().public_settings())?;
        let default = default["paste_method"].as_str().ok_or("no default")?;
        let result = assert_read_only(&directory, default);
        std::fs::set_permissions(store(&directory), std::fs::Permissions::from_mode(0o600))?;
        result
    }
    #[test]
    fn shared_lock_permissions_start_read_only_on_the_store_content() -> TestResult {
        let directory = tempfile::tempdir()?;
        prepared_store(&directory)?;
        let lock = directory.path().join("settings.lock");
        std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o644))?;
        assert_read_only(&directory, "direct")
    }
    #[test]
    fn a_real_lease_conflict_is_the_only_startup_refusal() -> TestResult {
        let directory = tempfile::tempdir()?;
        let instance = owned(&directory)?;
        let refused = start(&directory)
            .err()
            .ok_or("a second owner started while the lease is held")?;
        assert_eq!(refused.code(), SettingsErrorCode::Unavailable);
        drop(instance);
        assert!(!start(&directory)?.read_only());
        Ok(())
    }
}

/// SC-001 over every settable scalar key, generated from the descriptors: a valid value, an
/// out-of-domain value and a wrong type where the CLI can express one, compared between the UI
/// write path and the CLI path (same decision, code, reason and stored value).
mod generated_parity {
    use super::*;
    use handy_app_lib::settings_service::descriptor::{
        descriptors, SettingDescriptor, SettingType,
    };

    /// CLI text that denotes the same JSON value.
    fn cli_text(value: &Value) -> String {
        match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    }
    fn valid(entry: &SettingDescriptor, default: &Value) -> Value {
        if let Some(choice) = entry.allowed_values.iter().find(|v| default != &json!(v)) {
            return json!(choice);
        }
        match (entry.value_type, entry.range) {
            (SettingType::Boolean, _) => json!(!default.as_bool().unwrap_or(false)),
            (SettingType::Integer, Some(range)) => json!(range.max as u64),
            (SettingType::Number, Some(range)) => json!(range.min),
            _ if entry.nullable => Value::Null,
            _ => default.clone(),
        }
    }
    /// Out-of-domain and wrong-type candidates the CLI can also type.
    fn invalid(entry: &SettingDescriptor) -> Vec<Value> {
        let mut cases = Vec::new();
        if !entry.allowed_values.is_empty() {
            cases.push(json!("not_a_value"));
        }
        match (entry.value_type, entry.range) {
            (SettingType::Integer, Some(range)) => {
                cases.extend([json!(range.max as u64 + 1), json!(-1)])
            }
            (SettingType::Number, Some(range)) => {
                cases.extend([json!(range.max + 1.0), json!("abc")])
            }
            (SettingType::Boolean, _) => cases.push(json!("yes")),
            _ => {}
        }
        cases
    }
    fn settable(entry: &&SettingDescriptor) -> bool {
        !entry.read_only
            && !entry.secret
            && !matches!(entry.value_type, SettingType::Array | SettingType::Object)
    }
    #[test]
    fn every_settable_scalar_key_gets_the_same_decision_on_ui_and_cli() -> TestResult {
        let fresh = tempfile::tempdir()?;
        let defaults = serde_json::to_value(owned(&fresh)?.snapshot().public_settings())?;
        let mut covered = 0;
        for entry in descriptors().iter().filter(settable) {
            let mut cases = vec![valid(entry, &defaults[entry.key])];
            cases.extend(invalid(entry));
            for value in cases {
                let text = cli_text(&value);
                let ui = ui_path(entry.key, value)?;
                let cli = cli_path(entry.key, &text)?;
                assert_eq!(ui, cli, "UI and CLI diverge for {}={text}", entry.key);
            }
            covered += 1;
        }
        assert_eq!(covered, descriptors().iter().filter(settable).count());
        assert!(covered >= 45, "only {covered} settable scalar keys");
        Ok(())
    }
}
