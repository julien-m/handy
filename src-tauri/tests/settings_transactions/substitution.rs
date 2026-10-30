//! Same-user parent and alias substitutions are detected and refused without writing.
use handy_app_lib::settings_service::platform_capability::PlatformCapabilities;
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
            platform: PlatformCapabilities::current(),
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
fn same_user_ancestor_symlink_resolves_to_its_canonical_directory() -> TestResult {
    let directory = tempfile::tempdir()?;
    let real = directory.path().join("real");
    std::fs::create_dir(&real)?;
    let alias = directory.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias)?;
    let executable = directory.path().join("handy");
    let resolved = StoreLocator::resolve(&executable, &alias.join("nested"))?;
    assert_eq!(
        resolved,
        real.canonicalize()?
            .join("nested")
            .join("settings_store.json")
    );
    let repository = SettingsRepository::new(alias.join("settings_store.json"));
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let start_hidden = validate_cli(
        "start_hidden",
        "true",
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
            platform: PlatformCapabilities::current(),
        },
    )?;
    let committed = repository.commit_with_probe(start_hidden, None, &probe, &())?;
    assert_eq!(committed.snapshot.revision.0, 1);
    let disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(real.join("settings_store.json"))?)?;
    assert_eq!(disk["settings"]["start_hidden"], true);
    assert!(real.join("settings.lock").is_file());
    assert!(std::fs::symlink_metadata(&alias)?.file_type().is_symlink());
    Ok(())
}
#[test]
#[cfg(unix)]
fn ancestor_symlink_into_another_users_directory_is_refused() -> TestResult {
    // SAFETY: geteuid has no pointers and does not mutate Rust memory.
    if unsafe { libc::geteuid() } == 0 {
        return Ok(()); // root owns the system target, so no user boundary is crossed.
    }
    let directory = tempfile::tempdir()?;
    let alias = directory.path().join("alias");
    // `/usr` exists on macOS and Linux and is owned by root, never by the test user.
    std::os::unix::fs::symlink("/usr", &alias)?;
    let executable = directory.path().join("handy");
    let resolved = StoreLocator::resolve(&executable, &alias.join("nested"))
        .err()
        .ok_or("foreign alias resolved")?;
    assert_eq!(resolved.code(), SettingsErrorCode::Unavailable);
    let read = SettingsRepository::new(alias.join("settings_store.json"))
        .read_snapshot()
        .err()
        .ok_or("foreign alias read")?;
    assert_eq!(read.code(), SettingsErrorCode::Unavailable);
    Ok(())
}
/// After the locked read, move the store and both locks into a fresh directory at the same path:
/// every file identity still matches, so only the parent (dev, ino) comparison can refuse.
struct MoveIntoNewParent<'a>(&'a Swap);
impl PersistObserver for MoveIntoNewParent<'_> {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == PersistStage::AfterLoad {
            std::fs::rename(&self.0.store, &self.0.original)?;
            std::fs::create_dir(&self.0.store)?;
            for name in [
                "settings_store.json",
                "settings.lock",
                "settings.instance.lock",
            ] {
                std::fs::rename(self.0.original.join(name), self.0.store.join(name))?;
            }
        }
        Ok(())
    }
}
#[test]
#[cfg(unix)]
fn store_and_locks_moved_into_a_substituted_parent_are_refused_by_parent_identity() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir()?;
    let swap = Swap {
        store: root.path().join("store"),
        original: root.path().join("original"),
        substitute: root.path().join("unused"),
    };
    std::fs::create_dir(&swap.store)?;
    std::fs::set_permissions(&swap.store, std::fs::Permissions::from_mode(0o700))?;
    let path = swap.store.join("settings_store.json");
    let original =
        serde_json::to_vec(&json!({"settings_revision":7,"settings":{"audio_feedback":true}}))?;
    std::fs::write(&path, &original)?;
    let probe = LegacyPathProbe::new(root.path().join("absent.sock"));
    let error = SettingsRepository::new(path.clone())
        .commit_with_probe(disable_feedback()?, None, &probe, &MoveIntoNewParent(&swap))
        .err()
        .ok_or("commit accepted in a substituted parent")?;
    assert_eq!(error.code(), SettingsErrorCode::PersistenceFailed);
    assert_eq!(std::fs::read(&path)?, original);
    assert!(super::faults::temporary_files(&swap.store)?.is_empty());
    assert!(super::faults::temporary_files(&swap.original)?.is_empty());
    Ok(())
}
