//! Non-creating reads and recovery preserve revision continuity and raw bytes.
use handy_app_lib::settings_service::{protocol::Revision, store::SettingsRepository};
use serde_json::json;

#[test]
fn missing_store_returns_defaults_without_creating_parent_or_store(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("absent").join("settings_store.json");
    let read = SettingsRepository::new(path.clone()).read_snapshot()?;
    assert_eq!(read.snapshot.revision, Revision(0));
    assert!(!read.warning);
    assert_eq!(
        serde_json::to_value(read.snapshot.public_settings())?
            .as_object()
            .ok_or("object")?
            .len(),
        63
    );
    assert!(!path.parent().ok_or("parent")?.exists());
    Ok(())
}
#[test]
fn salvage_keeps_valid_revision_valid_fields_and_store_bytes(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let bytes = serde_json::to_vec(
        &json!({"settings_revision":41,"unknown_top":"CANARY_TOP","settings":{"paste_method":"invalid_method","audio_feedback":false,"post_process_api_keys":{"custom":"CANARY_SECRET"},"unknown_payload":"CANARY_PAYLOAD"}}),
    )?;
    std::fs::write(&path, &bytes)?;
    let read = SettingsRepository::new(path.clone()).read_snapshot()?;
    assert_eq!(read.snapshot.revision, Revision(41));
    assert!(read.warning);
    let public = serde_json::to_value(read.snapshot.public_settings())?;
    assert_eq!(public["audio_feedback"], false);
    assert_eq!(
        public["post_process_api_keys"]["custom"],
        json!({"present":true})
    );
    assert_eq!(std::fs::read(path)?, bytes);
    Ok(())
}
#[test]
fn broken_json_read_recovers_defaults_without_overwriting() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    std::fs::write(&path, b"CANARY_BROKEN_JSON")?;
    let read = SettingsRepository::new(path.clone()).read_snapshot()?;
    assert!(read.warning);
    assert_eq!(read.snapshot.revision, Revision(0));
    assert_eq!(std::fs::read(path)?, b"CANARY_BROKEN_JSON");
    Ok(())
}
