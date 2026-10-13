//! Fault stages expose only complete old/new JSON; substitutes are retained unmodified.
use handy_app_lib::settings_service::{
    errors::SettingsErrorCode,
    ownership::LegacyPathProbe,
    store::SettingsRepository,
    transaction::{PersistObserver, PersistStage},
    validation::{validate_cli, ValidationContext},
};
use serde_json::json;
struct FailAt(PersistStage);
impl PersistObserver for FailAt {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == self.0 {
            return Err(std::io::Error::other("injected filesystem failure"));
        }
        Ok(())
    }
}
#[test]
fn every_durable_fault_preserves_a_complete_generation_and_reports_uncertainty(
) -> Result<(), Box<dyn std::error::Error>> {
    for stage in [
        PersistStage::AfterLoad,
        PersistStage::BeforeWrite,
        PersistStage::AfterWrite,
        PersistStage::AfterFileSync,
        PersistStage::BeforeReplace,
        PersistStage::AfterReplace,
        PersistStage::AfterDirectorySync,
    ] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("settings_store.json");
        let original = json!({"settings_revision":7,"settings":{"audio_feedback":true,"unknown":"CANARY_KEEP"}});
        std::fs::write(&path, serde_json::to_vec(&original)?)?;
        let mutation = validate_cli(
            "audio_feedback",
            "false",
            &ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
            },
        )?;
        let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
        let error = SettingsRepository::new(path.clone())
            .commit_with_probe(mutation, None, &probe, &FailAt(stage))
            .err()
            .ok_or("fault succeeded")?;
        let disk: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        if matches!(
            stage,
            PersistStage::AfterReplace | PersistStage::AfterDirectorySync
        ) {
            assert_eq!(error.code(), SettingsErrorCode::Unavailable);
            assert_eq!(
                serde_json::to_value(error)?["details"]["outcome"],
                "unknown"
            );
            assert_eq!(disk["settings_revision"], 8);
            assert_eq!(disk["settings"]["audio_feedback"], false);
        } else {
            assert_eq!(error.code(), SettingsErrorCode::PersistenceFailed);
            assert_eq!(disk, original);
        }
        assert_eq!(disk["settings"]["unknown"], "CANARY_KEEP");
        assert_eq!(
            temporary_files(directory.path())?,
            Vec::<std::path::PathBuf>::new()
        );
    }
    Ok(())
}
/// Private generations left in `directory`; a refused commit must not retain one holding secrets.
pub(super) fn temporary_files(
    directory: &std::path::Path,
) -> std::io::Result<Vec<std::path::PathBuf>> {
    Ok(std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "tmp"))
        .collect())
}
struct ReplaceTemp {
    directory: std::path::PathBuf,
    substitute: std::sync::Mutex<Option<std::path::PathBuf>>,
}
impl PersistObserver for ReplaceTemp {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == PersistStage::BeforeReplace {
            let path = std::fs::read_dir(&self.directory)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| path.extension().is_some_and(|extension| extension == "tmp"))
                .ok_or_else(|| std::io::Error::other("private temp missing"))?;
            std::fs::rename(&path, self.directory.join("retained-original"))?;
            std::fs::write(&path, b"CANARY_SUBSTITUTE")?;
            *self
                .substitute
                .lock()
                .map_err(|_| std::io::Error::other("substitute mutex"))? = Some(path);
        }
        Ok(())
    }
}
#[test]
fn replaced_temporary_path_never_replaces_store_or_deletes_substitute(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let bytes =
        serde_json::to_vec(&json!({"settings_revision":7,"settings":{"audio_feedback":true}}))?;
    std::fs::write(&path, &bytes)?;
    let observer = ReplaceTemp {
        directory: directory.path().to_path_buf(),
        substitute: std::sync::Mutex::new(None),
    };
    let mutation = validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?;
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(mutation, None, &probe, &observer)
        .err()
        .ok_or("substitute persisted")?;
    assert_eq!(error.code(), SettingsErrorCode::PersistenceFailed);
    assert_eq!(std::fs::read(path)?, bytes);
    let substitute = observer
        .substitute
        .lock()
        .map_err(|_| "mutex")?
        .clone()
        .ok_or("substitute path")?;
    assert_eq!(std::fs::read(substitute)?, b"CANARY_SUBSTITUTE");
    Ok(())
}
#[test]
#[cfg(unix)]
fn stable_lock_inode_survives_replacement_and_is_retained_after_release(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::MetadataExt;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("settings_store.json");
    let repository = SettingsRepository::new(path.clone());
    repository.read_snapshot()?;
    let lock = directory.path().join("settings.lock");
    let inode = std::fs::metadata(&lock)?.ino();
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    for value in ["false", "true"] {
        let mutation = validate_cli(
            "audio_feedback",
            value,
            &ValidationContext {
                prompt_ids: &[],
                updates_allowed: true,
            },
        )?;
        repository.commit_with_probe(mutation, None, &probe, &())?;
        assert_eq!(std::fs::metadata(&lock)?.ino(), inode);
    }
    assert!(lock.is_file());
    Ok(())
}
