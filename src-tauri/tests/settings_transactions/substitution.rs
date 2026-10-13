//! Same-user parent and alias substitutions are detected and refused without writing.
use handy_app_lib::settings_service::{
    errors::SettingsErrorCode,
    locator::StoreLocator,
    ownership::{LegacyPathProbe, OwnerProbe, OwnerState},
    store::SettingsRepository,
    transaction::{PersistObserver, PersistStage},
    validation::{validate_cli, SettingsMutation, ValidationContext},
};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn disable_feedback(
) -> Result<SettingsMutation, handy_app_lib::settings_service::errors::SettingsError> {
    validate_cli(
        "audio_feedback",
        "false",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )
}
/// Directory names used by one rename-and-restore (ABA) sequence.
struct Swap {
    store: PathBuf,
    original: PathBuf,
    substitute: PathBuf,
}
/// First passive probe runs after locking: move the locked parent away and install a substitute.
struct SwapBeforeLoad {
    swap: Swap,
    swapped: AtomicBool,
}
impl OwnerProbe for SwapBeforeLoad {
    fn inspect(&self) -> OwnerState {
        if self.swapped.swap(true, Ordering::SeqCst) {
            return OwnerState::Absent;
        }
        let moved = std::fs::rename(&self.swap.store, &self.swap.original)
            .and_then(|()| std::fs::rename(&self.swap.substitute, &self.swap.store));
        if moved.is_err() {
            return OwnerState::Unavailable;
        }
        OwnerState::Absent
    }
}
/// After the substituted read, restore the original parent so pathname and lock checks pass again.
struct RestoreAfterLoad<'a>(&'a Swap);
impl PersistObserver for RestoreAfterLoad<'_> {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == PersistStage::AfterLoad {
            std::fs::rename(&self.0.store, &self.0.substitute)?;
            std::fs::rename(&self.0.original, &self.0.store)?;
        }
        Ok(())
    }
}
#[test]
#[cfg(unix)]
fn parent_substituted_before_read_and_restored_before_replace_is_refused() -> TestResult {
    let root = tempfile::tempdir()?;
    let swap = Swap {
        store: root.path().join("store"),
        original: root.path().join("original"),
        substitute: root.path().join("substitute"),
    };
    std::fs::create_dir(&swap.store)?;
    std::fs::create_dir(&swap.substitute)?;
    let path = swap.store.join("settings_store.json");
    let original =
        serde_json::to_vec(&json!({"settings_revision":7,"settings":{"audio_feedback":true}}))?;
    let substituted = serde_json::to_vec(
        &json!({"settings_revision":7,"settings":{"audio_feedback":true,"unknown":"CANARY_SUBSTITUTE"}}),
    )?;
    std::fs::write(&path, &original)?;
    std::fs::write(swap.substitute.join("settings_store.json"), &substituted)?;
    let probe = SwapBeforeLoad {
        swap: Swap {
            store: swap.store.clone(),
            original: swap.original.clone(),
            substitute: swap.substitute.clone(),
        },
        swapped: AtomicBool::new(false),
    };
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(disable_feedback()?, None, &probe, &RestoreAfterLoad(&swap))
        .err()
        .ok_or("candidate derived from a substituted parent was committed")?;
    assert_eq!(error.code(), SettingsErrorCode::PersistenceFailed);
    assert_eq!(std::fs::read(&path)?, original);
    assert_eq!(
        std::fs::read(swap.substitute.join("settings_store.json"))?,
        substituted
    );
    Ok(())
}
#[test]
#[cfg(unix)]
fn store_file_symlink_is_refused_for_reads_and_writes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let target = directory.path().join("elsewhere.json");
    let bytes =
        serde_json::to_vec(&json!({"settings_revision":3,"settings":{"audio_feedback":true}}))?;
    std::fs::write(&target, &bytes)?;
    let link = directory.path().join("settings_store.json");
    std::os::unix::fs::symlink(&target, &link)?;
    let repository = SettingsRepository::new(link.clone());
    let read = repository.read_snapshot().err().ok_or("store alias read")?;
    assert_eq!(read.code(), SettingsErrorCode::Unavailable);
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let write = repository
        .commit_with_probe(disable_feedback()?, None, &probe, &())
        .err()
        .ok_or("store alias written")?;
    assert_eq!(write.code(), SettingsErrorCode::Unavailable);
    assert_eq!(std::fs::read(&target)?, bytes);
    assert!(std::fs::symlink_metadata(&link)?.file_type().is_symlink());
    Ok(())
}
#[test]
#[cfg(unix)]
fn ancestor_directory_symlink_is_refused_by_locator_and_repository() -> TestResult {
    let directory = tempfile::tempdir()?;
    let real = directory.path().join("real");
    std::fs::create_dir(&real)?;
    let alias = directory.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias)?;
    let executable = directory.path().join("handy");
    let resolved = StoreLocator::resolve(&executable, &alias.join("nested"))
        .err()
        .ok_or("ancestor alias resolved")?;
    assert_eq!(resolved.code(), SettingsErrorCode::Unavailable);
    let read = SettingsRepository::new(alias.join("settings_store.json"))
        .read_snapshot()
        .err()
        .ok_or("ancestor alias read")?;
    assert_eq!(read.code(), SettingsErrorCode::Unavailable);
    assert!(!real.join("settings.lock").exists());
    Ok(())
}
