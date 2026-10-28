//! Writes and startup on not-yet-normalized, empty or unreadable stores keep every visible field.
use handy_app_lib::settings_service::platform_capability::PlatformCapabilities;
use handy_app_lib::settings_service::{
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn public(repository: &SettingsRepository) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_value(
        repository.read_snapshot()?.snapshot.public_settings(),
    )?)
}
/// Commit one CLI value on a store seeded with `stored`; return the repository and the commit revision.
fn commit_on(
    directory: &tempfile::TempDir,
    stored: &Value,
    key: &str,
    value: &str,
) -> Result<(SettingsRepository, Revision), Box<dyn std::error::Error>> {
    let path = directory.path().join("settings_store.json");
    std::fs::write(&path, serde_json::to_vec(stored)?)?;
    let repository = SettingsRepository::new(path);
    let mutation = validate_cli(
        key,
        value,
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
            platform: PlatformCapabilities::current(),
        },
    )?;
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let revision = repository
        .commit_with_probe(mutation, None, &probe, &())?
        .snapshot
        .revision;
    Ok((repository, revision))
}
#[test]
fn gpu_device_write_on_schema_one_store_survives_the_next_read() -> TestResult {
    let directory = tempfile::tempdir()?;
    let stored = json!({"settings":{"settings_schema_version":1,"onboarding_completed":true,"whats_new_last_seen_version":"x","overlay_style":"live","chinese_script":"as_transcribed"}});
    let (repository, revision) =
        commit_on(&directory, &stored, "transcribe_gpu_device", "gpu-handle-1")?;
    assert_eq!(revision, Revision(1));
    let after = public(&repository)?;
    assert_eq!(after["transcribe_gpu_device"], "gpu-handle-1");
    assert_eq!(repository.read_snapshot()?.snapshot.revision, Revision(1));
    Ok(())
}
#[test]
fn overlay_position_write_on_legacy_hidden_overlay_keeps_overlay_style() -> TestResult {
    let directory = tempfile::tempdir()?;
    let stored = json!({"settings":{"overlay_position":"none","settings_schema_version":2,"onboarding_completed":true,"whats_new_last_seen_version":"x","chinese_script":"as_transcribed"}});
    let (repository, _) = commit_on(&directory, &stored, "overlay_position", "top")?;
    let after = public(&repository)?;
    assert_eq!(after["overlay_position"], "top");
    assert_eq!(after["overlay_style"], "none");
    Ok(())
}
#[test]
fn chinese_script_write_on_legacy_language_intent_keeps_selected_language() -> TestResult {
    let directory = tempfile::tempdir()?;
    let stored = json!({"settings":{"selected_language":"zh-Hant","settings_schema_version":2,"onboarding_completed":true,"whats_new_last_seen_version":"x","overlay_style":"live"}});
    let (repository, _) = commit_on(&directory, &stored, "chinese_script", "simplified")?;
    let after = public(&repository)?;
    assert_eq!(after["chinese_script"], "simplified");
    assert_eq!(after["selected_language"], "zh");
    Ok(())
}
#[test]
fn normalized_write_changes_only_the_mutated_visible_field() -> TestResult {
    let directory = tempfile::tempdir()?;
    let stored = json!({"settings":{"settings_schema_version":1,"push_to_talk":true,"selected_language":"zh-Hans","overlay_position":"none","unknown_payload":"CANARY_KEEP"},"unknown_top":"CANARY_TOP"});
    let path = directory.path().join("settings_store.json");
    std::fs::write(&path, serde_json::to_vec(&stored)?)?;
    let mut expected = public(&SettingsRepository::new(path.clone()))?;
    let (repository, _) = commit_on(&directory, &stored, "audio_feedback", "true")?;
    expected["audio_feedback"] = json!(true);
    assert_eq!(public(&repository)?, expected);
    let disk: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(disk["settings"]["unknown_payload"], "CANARY_KEEP");
    assert_eq!(disk["unknown_top"], "CANARY_TOP");
    Ok(())
}
#[test]
fn store_without_settings_object_reads_as_a_fresh_install() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    std::fs::write(&path, b"{}")?;
    let existing = public(&SettingsRepository::new(path))?;
    let missing = public(&SettingsRepository::new(
        directory.path().join("none").join("settings_store.json"),
    ))?;
    assert_eq!(existing, missing);
    Ok(())
}
#[test]
fn startup_on_unreadable_json_or_malformed_revision_recovers_read_only_with_lease() -> TestResult {
    for bytes in [
        b"CANARY_BROKEN_JSON".to_vec(),
        serde_json::to_vec(
            &json!({"settings_revision":"CANARY_COUNTER","settings":{"audio_feedback":false}}),
        )?,
    ] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("settings_store.json");
        std::fs::write(&path, &bytes)?;
        let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
        let repository = SettingsRepository::new(path.clone());
        let (read, lease) = repository.initialize(&probe)?;
        assert!(read.warning);
        lease.validate()?;
        assert_eq!(std::fs::read(&path)?, bytes);
        let mutation = validate_cli(
            "audio_feedback",
            "true",
            &ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
                platform: PlatformCapabilities::current(),
            },
        )?;
        let offline = repository
            .commit_with_probe(mutation, None, &probe, &())
            .err()
            .ok_or("offline write accepted while the recovered instance holds the lease")?;
        assert_eq!(
            offline.code(),
            handy_app_lib::settings_service::errors::SettingsErrorCode::Unavailable
        );
        assert_eq!(std::fs::read(&path)?, bytes);
    }
    Ok(())
}
/// Commit one CLI value on an existing repository with an absent passive owner.
fn commit_value(
    repository: &SettingsRepository,
    directory: &tempfile::TempDir,
    key: &str,
    value: &str,
) -> Result<
    handy_app_lib::settings_service::store::SettingsRead,
    handy_app_lib::settings_service::errors::SettingsError,
> {
    let mutation = validate_cli(
        key,
        value,
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
            platform: PlatformCapabilities::current(),
        },
    )?;
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    repository.commit_with_probe(mutation, None, &probe, &())
}
#[test]
fn gpu_accelerator_without_device_is_refused_and_changes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let repository = SettingsRepository::new(path.clone());
    commit_value(&repository, &directory, "audio_feedback", "true")?;
    let bytes = std::fs::read(&path)?;
    let error = commit_value(&repository, &directory, "transcribe_accelerator", "gpu")
        .err()
        .ok_or("gpu without device accepted")?;
    let wire = serde_json::to_value(&error)?;
    assert_eq!(wire["code"], "validation_failed");
    assert_eq!(wire["details"]["field"], "transcribe_accelerator");
    assert_eq!(
        wire["details"]["issues"][0]["reason"],
        "The GPU accelerator requires a selected GPU device."
    );
    assert_eq!(std::fs::read(&path)?, bytes);
    Ok(())
}
#[test]
fn gpu_accelerator_with_device_is_reread_as_returned_and_device_cannot_be_cleared() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let repository = SettingsRepository::new(path.clone());
    commit_value(&repository, &directory, "transcribe_gpu_device", "gpu-1")?;
    let committed = commit_value(&repository, &directory, "transcribe_accelerator", "gpu")?;
    let returned = serde_json::to_value(committed.snapshot.public_settings())?;
    assert_eq!(returned["transcribe_accelerator"], "gpu");
    assert_eq!(public(&repository)?, returned);
    let bytes = std::fs::read(&path)?;
    let error = commit_value(&repository, &directory, "transcribe_gpu_device", "null")
        .err()
        .ok_or("device cleared under the GPU accelerator")?;
    let wire = serde_json::to_value(&error)?;
    assert_eq!(wire["details"]["field"], "transcribe_gpu_device");
    assert_eq!(std::fs::read(&path)?, bytes);
    commit_value(&repository, &directory, "transcribe_accelerator", "auto")?;
    commit_value(&repository, &directory, "transcribe_gpu_device", "null")?;
    assert_eq!(public(&repository)?["transcribe_accelerator"], "auto");
    Ok(())
}
