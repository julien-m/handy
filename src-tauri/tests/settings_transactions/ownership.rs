//! Native isolated owner resources and startup leases prevent offline commits.
use handy_app_lib::settings_service::{
    errors::SettingsErrorCode,
    ownership::LegacyPathProbe,
    store::SettingsRepository,
    transaction::{PersistObserver, PersistStage},
    validation::{validate_cli, ValidationContext},
};
use serde_json::json;
#[cfg(unix)]
struct AppearingOwner {
    path: std::path::PathBuf,
    listener: std::sync::Mutex<Option<std::os::unix::net::UnixListener>>,
}
#[cfg(unix)]
impl PersistObserver for AppearingOwner {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == PersistStage::AfterFileSync {
            *self
                .listener
                .lock()
                .map_err(|_| std::io::Error::other("listener mutex"))? =
                Some(std::os::unix::net::UnixListener::bind(&self.path)?);
        }
        Ok(())
    }
}
#[test]
#[cfg(unix)]
fn native_owner_appearing_after_temp_sync_refuses_before_replace(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let bytes =
        serde_json::to_vec(&json!({"settings_revision":7,"settings":{"audio_feedback":true}}))?;
    std::fs::write(&path, &bytes)?;
    let owner = directory.path().join("legacy.sock");
    let probe = LegacyPathProbe::new(owner.clone());
    let observer = AppearingOwner {
        path: owner,
        listener: std::sync::Mutex::new(None),
    };
    let mutation = validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?;
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(mutation, None, &probe, &observer)
        .err()
        .ok_or("native owner ignored at commit")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert_eq!(std::fs::read(path)?, bytes);
    assert!(super::faults::temporary_files(directory.path())?.is_empty());
    Ok(())
}
#[test]
fn startup_normalizes_once_preserves_raw_secrets_and_keeps_offline_fenced(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let stored = json!({"settings_revision":41,"unknown_top":"CANARY_TOP","settings":{"push_to_talk":true,"selected_model":"chosen","post_process_api_keys":{"custom":"CANARY_KEY"},"post_process_providers":[{"id":"custom","label":"Custom","base_url":"https://example.invalid","unknown_secret":"CANARY_PROVIDER"}],"unknown_payload":"CANARY_PAYLOAD"}});
    std::fs::write(&path, serde_json::to_vec(&stored)?)?;
    let repository = SettingsRepository::new(path.clone());
    let (read, lease) = repository.initialize(&probe)?;
    assert_eq!(read.snapshot.revision.0, 42);
    lease.validate()?;
    let bytes = std::fs::read(&path)?;
    let disk: serde_json::Value = serde_json::from_slice(&bytes)?;
    assert_eq!(disk["unknown_top"], stored["unknown_top"]);
    assert_eq!(
        disk["settings"]["unknown_payload"],
        stored["settings"]["unknown_payload"]
    );
    assert_eq!(
        disk["settings"]["post_process_api_keys"],
        stored["settings"]["post_process_api_keys"]
    );
    assert_eq!(
        disk["settings"]["post_process_providers"][0]["unknown_secret"],
        "CANARY_PROVIDER"
    );
    let mutation = validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?;
    let error = repository
        .commit_with_probe(mutation, None, &probe, &())
        .err()
        .ok_or("busy startup permitted offline")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert_eq!(std::fs::read(&path)?, bytes);
    drop(lease);
    let (second, lease) = repository.initialize(&probe)?;
    assert_eq!(second.snapshot.revision.0, 42);
    assert_eq!(std::fs::read(path)?, bytes);
    drop(lease);
    Ok(())
}
#[test]
#[cfg(unix)]
fn passive_occupied_socket_refuses_without_connecting_or_deleting(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let owner = directory.path().join("legacy.sock");
    let listener = std::os::unix::net::UnixListener::bind(&owner)?;
    listener.set_nonblocking(true)?;
    let probe = LegacyPathProbe::new(owner.clone());
    let mutation = validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?;
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(mutation, None, &probe, &())
        .err()
        .ok_or("occupied owner accepted")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert!(!path.exists());
    assert!(owner.exists());
    assert_eq!(
        listener
            .accept()
            .err()
            .ok_or("passive probe connected")?
            .kind(),
        std::io::ErrorKind::WouldBlock
    );
    Ok(())
}
#[test]
fn startup_merge_never_copies_unknown_fields_between_items_with_duplicate_ids(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let stored = json!({"settings_revision":3,"settings":{"post_process_prompts":[{"id":"dup","name":"First","prompt":"First","extra":"CANARY_FIRST"},{"id":"dup","name":"Second","prompt":"Second"}]}});
    std::fs::write(&path, serde_json::to_vec(&stored)?)?;
    let (_, lease) = SettingsRepository::new(path.clone()).initialize(&probe)?;
    drop(lease);
    let disk: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let prompts = disk["settings"]["post_process_prompts"]
        .as_array()
        .ok_or("prompts array")?;
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0]["name"], "First");
    assert_eq!(prompts[1]["name"], "Second");
    assert!(prompts[1].get("extra").is_none());
    Ok(())
}
#[test]
fn lease_holding_instance_commits_while_offline_writers_stay_refused(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let repository = SettingsRepository::new(path.clone());
    let (read, lease) = repository.initialize(&probe)?;
    let start = read.snapshot.revision;
    let context = ValidationContext {
        prompt_ids: &[],
        updates_allowed: true,
    };
    let owned = repository.commit_owned(
        &lease,
        validate_cli("start_hidden", "true", &context)?,
        Some(start),
    )?;
    assert_eq!(owned.snapshot.revision.0, start.0 + 1);
    let bytes = std::fs::read(&path)?;
    let disk: serde_json::Value = serde_json::from_slice(&bytes)?;
    assert_eq!(disk["settings"]["start_hidden"], true);
    assert_eq!(disk["settings_revision"], start.0 + 1);
    let offline = repository
        .commit_with_probe(
            validate_cli("audio_feedback", "false", &context)?,
            None,
            &probe,
            &(),
        )
        .err()
        .ok_or("offline writer accepted while the instance holds the lease")?;
    assert_eq!(offline.code(), SettingsErrorCode::Unavailable);
    let stale = repository
        .commit_owned(
            &lease,
            validate_cli("start_hidden", "false", &context)?,
            Some(start),
        )
        .err()
        .ok_or("stale owner CAS accepted")?;
    assert_eq!(stale.code(), SettingsErrorCode::RevisionConflict);
    assert_eq!(std::fs::read(&path)?, bytes);
    let other = tempfile::tempdir()?;
    let (_, foreign) = SettingsRepository::new(other.path().join("settings_store.json"))
        .initialize(&LegacyPathProbe::new(other.path().join("absent.sock")))?;
    let mismatched = repository
        .commit_owned(
            &foreign,
            validate_cli("start_hidden", "false", &context)?,
            None,
        )
        .err()
        .ok_or("lease of another store accepted")?;
    assert_eq!(mismatched.code(), SettingsErrorCode::Unavailable);
    assert_eq!(std::fs::read(&path)?, bytes);
    drop(lease);
    Ok(())
}
/// Revalidated value of `key` on the defaults, so committing it is a semantic no-op on an absent store.
fn default_noop(
    key: &str,
) -> Result<handy_app_lib::settings_service::validation::SettingsMutation, Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    let defaults =
        SettingsRepository::new(directory.path().join("absent").join("settings_store.json"))
            .read_snapshot()?;
    let value = serde_json::to_value(defaults.snapshot.public_settings())?[key].to_string();
    Ok(validate_cli(
        key,
        &value,
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?)
}
#[test]
fn foreign_lease_noop_on_absent_store_is_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let owned = tempfile::tempdir()?;
    let probe = LegacyPathProbe::new(owned.path().join("absent.sock"));
    let (_, lease) =
        SettingsRepository::new(owned.path().join("settings_store.json")).initialize(&probe)?;
    let other = tempfile::tempdir()?;
    let absent = SettingsRepository::new(other.path().join("settings_store.json"));
    let error = absent
        .commit_owned(&lease, default_noop("audio_feedback")?, None)
        .err()
        .ok_or("foreign lease accepted on an absent store")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    assert!(std::fs::read_dir(other.path())?.next().is_none());
    Ok(())
}
#[test]
fn offline_noop_on_deleted_store_while_instance_runs_is_unavailable(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let repository = SettingsRepository::new(path.clone());
    let (_, lease) = repository.initialize(&probe)?;
    std::fs::remove_file(&path)?;
    let error = repository
        .commit_with_probe(default_noop("audio_feedback")?, None, &probe, &())
        .err()
        .ok_or("offline no-op accepted while the instance holds the lease")?;
    assert_eq!(error.code(), SettingsErrorCode::Unavailable);
    let owned = repository.commit_owned(&lease, default_noop("audio_feedback")?, None)?;
    assert_eq!(owned.snapshot.revision.0, 0);
    assert!(!path.exists());
    Ok(())
}
