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
