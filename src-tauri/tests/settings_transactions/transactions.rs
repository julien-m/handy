//! Durable revision/CAS/no-op/refusal contracts use real raw files.
use handy_app_lib::settings_service::{
    errors::SettingsErrorCode,
    protocol::Revision,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::json;
fn mutation(
    key: &str,
    value: &str,
) -> Result<
    handy_app_lib::settings_service::validation::SettingsMutation,
    handy_app_lib::settings_service::errors::SettingsError,
> {
    validate_cli(
        key,
        value,
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )
}
#[test]
fn changed_patch_is_durable_increments_once_and_preserves_unknowns_and_secrets(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let original = json!({"settings_revision":7,"future":{"secret":"CANARY_TOP"},"settings":{"future":{"secret":"CANARY_NESTED"},"post_process_api_keys":{"custom":"CANARY_KEY"},"audio_feedback":true}});
    std::fs::write(&path, serde_json::to_vec(&original)?)?;
    let read = SettingsRepository::new(path.clone()).commit_with_probe(
        mutation("audio_feedback", "false")?,
        Some(Revision(7)),
        &probe(&directory),
        &(),
    )?;
    assert_eq!(read.snapshot.revision, Revision(8));
    let disk: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(disk["settings_revision"], 8);
    assert_eq!(disk["settings"]["audio_feedback"], false);
    for key in ["future", "post_process_api_keys"] {
        assert_eq!(disk["settings"][key], original["settings"][key]);
    }
    assert_eq!(disk["future"], original["future"]);
    Ok(())
}
#[test]
fn stale_compare_and_set_refuses_without_changing_bytes() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let bytes =
        serde_json::to_vec(&json!({"settings_revision":5,"settings":{"audio_feedback":true}}))?;
    std::fs::write(&path, &bytes)?;
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(
            mutation("audio_feedback", "false")?,
            Some(Revision(4)),
            &probe(&directory),
            &(),
        )
        .err()
        .ok_or("stale CAS accepted")?;
    assert_eq!(error.code(), SettingsErrorCode::RevisionConflict);
    assert_eq!(
        serde_json::to_value(error)?["details"]["current_revision"],
        5
    );
    assert_eq!(std::fs::read(path)?, bytes);
    Ok(())
}
#[test]
fn corrupt_or_overflow_revision_and_broken_json_refuse_write(
) -> Result<(), Box<dyn std::error::Error>> {
    for bytes in [
        serde_json::to_vec(
            &json!({"settings_revision":"CANARY_COUNTER","settings":{"audio_feedback":true}}),
        )?,
        serde_json::to_vec(
            &json!({"settings_revision":u64::MAX,"settings":{"audio_feedback":true}}),
        )?,
        b"CANARY_BROKEN".to_vec(),
    ] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("settings_store.json");
        std::fs::write(&path, &bytes)?;
        let error = SettingsRepository::new(path.clone())
            .commit_with_probe(
                mutation("audio_feedback", "false")?,
                None,
                &probe(&directory),
                &(),
            )
            .err()
            .ok_or("unsafe continuity accepted")?;
        assert_eq!(error.code(), SettingsErrorCode::PersistenceFailed);
        assert!(!serde_json::to_string(&error)?.contains("CANARY"));
        assert_eq!(std::fs::read(path)?, bytes);
    }
    Ok(())
}
#[test]
fn semantic_noop_does_not_rewrite_or_increment_and_first_change_is_one(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let repository = SettingsRepository::new(path.clone());
    let defaults = serde_json::to_value(repository.read_snapshot()?.snapshot.public_settings())?;
    let raw = defaults["audio_feedback"].as_bool().ok_or("default bool")?;
    let read = repository.commit_with_probe(
        mutation("audio_feedback", if raw { "true" } else { "false" })?,
        None,
        &probe(&directory),
        &(),
    )?;
    assert_eq!(read.snapshot.revision, Revision(0));
    assert!(!path.exists());
    let read = repository.commit_with_probe(
        mutation("audio_feedback", if raw { "false" } else { "true" })?,
        None,
        &probe(&directory),
        &(),
    )?;
    assert_eq!(read.snapshot.revision, Revision(1));
    assert!(path.exists());
    Ok(())
}

fn probe(
    directory: &tempfile::TempDir,
) -> handy_app_lib::settings_service::ownership::LegacyPathProbe {
    handy_app_lib::settings_service::ownership::LegacyPathProbe::new(
        directory.path().join("absent-legacy.sock"),
    )
}
#[test]
fn prompt_mutation_revalidates_membership_against_the_locked_fresh_snapshot(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let bytes = serde_json::to_vec(
        &json!({"settings_revision":4,"settings":{"post_process_prompts":[{"id":"current","name":"Current","prompt":"Fixture prompt"}]}}),
    )?;
    std::fs::write(&path, &bytes)?;
    let stale = validate_cli(
        "post_process_selected_prompt_id",
        "removed",
        &ValidationContext {
            prompt_ids: &["removed"],
            updates_allowed: true,
        },
    )?;
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(stale, None, &probe(&directory), &())
        .err()
        .ok_or("removed prompt committed")?;
    assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
    assert_eq!(std::fs::read(path)?, bytes);
    Ok(())
}
