//! Private same-directory generations are flushed before replace and directory sync.
use super::{errors::SettingsError, store::persistence_failed};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
/// Names already taken (for example a retained temp of a crashed process whose PID was reused)
/// are skipped untouched. 64 consecutive collisions in one private directory can only come from
/// deliberate pre-creation, so the bound fails closed instead of looping or deleting files.
const TEMP_NAME_ATTEMPTS: u32 = 64;
/// Injection boundary corresponds to real transaction and durable-write stages, never a mock repository.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PersistStage {
    /// After the fresh locked read and before any candidate or temporary file exists.
    AfterLoad,
    /// The private temp exists and is still empty.
    BeforeWrite,
    /// The whole generation is written to the temp but not yet synced.
    AfterWrite,
    /// The temp is synced to disk; replacement has not been authorized yet.
    AfterFileSync,
    /// Just before the parent/temp revalidation and final authorization that precede replacement.
    BeforeReplace,
    /// The new generation replaced the store; directory durability is not yet confirmed.
    AfterReplace,
    /// The parent directory was synced after replacement.
    AfterDirectorySync,
}
/// Filesystem fault port; normal production commits use the inert implementation.
pub trait PersistObserver {
    /// Observe/inject one real durable stage; returned failure is mapped according to commit certainty.
    fn observe(&self, stage: PersistStage) -> std::io::Result<()>;
}
impl PersistObserver for () {
    fn observe(&self, _: PersistStage) -> std::io::Result<()> {
        Ok(())
    }
}

/// Write `value` as a new synchronized generation and atomically replace `path` with it.
///
/// Order matters: pin the parent, run the caller's authorization before any private temp (which
/// holds the whole document, secrets included) exists, write and fsync the temp, revalidate parent
/// and temp, authorize again immediately before replacement, replace, then make it durable.
/// Failures before replacement leave the old generation (`persistence_failed`) and discard this
/// call's own temp when its identity still matches the held handle; substitutes are never touched.
/// Failures after replacement report an unknown outcome.
pub(super) fn persist(
    path: &Path,
    value: &serde_json::Value,
    observer: &impl PersistObserver,
    authorize: impl Fn() -> Result<(), SettingsError>,
) -> Result<(), SettingsError> {
    let parent = path.parent().ok_or_else(persistence_failed)?;
    let directory = ParentDirectory::open(parent)?;
    authorize()?;
    let (temporary, mut file) = private_temp(parent)?;
    let staged = stage_generation(&directory, &temporary, &mut file, value, observer)
        .and_then(|()| authorize())
        .and_then(|()| {
            #[cfg(unix)]
            let replaced = directory.replace(&temporary, path);
            #[cfg(windows)]
            let replaced = directory.replace(&file, path);
            replaced
        });
    if let Err(error) = staged {
        discard_temp(&temporary, &file);
        return Err(error);
    }
    confirm_durable(&directory, observer)
}
/// Write, sync and revalidate the private generation before the final authorization.
fn stage_generation(
    directory: &ParentDirectory,
    temporary: &Path,
    file: &mut File,
    value: &serde_json::Value,
    observer: &impl PersistObserver,
) -> Result<(), SettingsError> {
    write_generation(file, value, observer)?;
    observer
        .observe(PersistStage::BeforeReplace)
        .map_err(|_| persistence_failed())?;
    directory.validate()?;
    validate_temp(temporary, file)
}
/// Remove this call's refused temp so no copy of the secrets is retained.
///
/// Unix deletes the pathname only while it still names the held inode (a same-user substitute is
/// left untouched; the check-then-unlink window is limited to the owner's private directory).
/// Windows deletes through the held handle itself, so no pathname race exists.
fn discard_temp(temporary: &Path, held: &File) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let (Ok(current), Ok(identity)) = (std::fs::symlink_metadata(temporary), held.metadata())
        else {
            return;
        };
        if (current.dev(), current.ino()) == (identity.dev(), identity.ino()) {
            // Best effort: the refusal is already reported; a failed unlink only retains the temp.
            let _ = std::fs::remove_file(temporary);
        }
    }
    #[cfg(windows)]
    {
        let _ = temporary;
        // Best effort: the refusal is already reported; a failed delete only retains the temp.
        let _ = super::windows_store::discard_owned(held);
    }
}
/// After replacement the new generation may already be visible, so every failure is an unknown outcome.
fn confirm_durable(
    directory: &ParentDirectory,
    observer: &impl PersistObserver,
) -> Result<(), SettingsError> {
    observer
        .observe(PersistStage::AfterReplace)
        .map_err(|_| SettingsError::unknown_outcome())?;
    directory.sync()?;
    observer
        .observe(PersistStage::AfterDirectorySync)
        .map_err(|_| SettingsError::unknown_outcome())
}
/// Parent directory retained from before the temporary generation exists until the final sync.
struct ParentDirectory {
    #[cfg(unix)]
    handle: File,
    #[cfg(unix)]
    identity: std::fs::Metadata,
    #[cfg(unix)]
    path: PathBuf,
    #[cfg(windows)]
    pinned: super::windows_store::PinnedDirectory,
}
#[cfg(unix)]
impl ParentDirectory {
    fn open(path: &Path) -> Result<Self, SettingsError> {
        let handle = File::open(path).map_err(|_| persistence_failed())?;
        let identity = handle.metadata().map_err(|_| persistence_failed())?;
        Ok(Self {
            handle,
            identity,
            path: path.to_path_buf(),
        })
    }
    fn validate(&self) -> Result<(), SettingsError> {
        use std::os::unix::fs::MetadataExt;
        let current = std::fs::symlink_metadata(&self.path).map_err(|_| persistence_failed())?;
        super::locator::validate_owner(&current)?;
        if (current.dev(), current.ino()) != (self.identity.dev(), self.identity.ino()) {
            return Err(persistence_failed());
        }
        Ok(())
    }
    /// Unix replacement is pathname-based: rename(2) atomically swaps the directory entry, so the
    /// held temp handle is not needed here (its identity was checked by `validate_temp`).
    fn replace(&self, temporary: &Path, target: &Path) -> Result<(), SettingsError> {
        std::fs::rename(temporary, target).map_err(|_| persistence_failed())
    }
    /// fsync the parent so the rename itself survives a crash.
    fn sync(&self) -> Result<(), SettingsError> {
        self.handle
            .sync_all()
            .map_err(|_| SettingsError::unknown_outcome())
    }
}
#[cfg(windows)]
impl ParentDirectory {
    fn open(path: &Path) -> Result<Self, SettingsError> {
        let pinned = super::windows_store::pin_parent(path).map_err(|_| persistence_failed())?;
        Ok(Self { pinned })
    }
    fn validate(&self) -> Result<(), SettingsError> {
        self.pinned.validate().map_err(|_| persistence_failed())
    }
    /// Windows replacement is handle-relative: the open temp handle is renamed inside the pinned
    /// parent, so the temp pathname is not needed here.
    fn replace(&self, file: &File, target: &Path) -> Result<(), SettingsError> {
        let name = target.file_name().ok_or_else(persistence_failed)?;
        super::windows_store::replace_owned(file, &self.pinned, name)
            // OS diagnostics remain private; the replacement phase determines the fixed public code.
            .map_err(|error| match error {
                super::windows_store::ReplaceError::Before(error) => {
                    let _ = error;
                    persistence_failed()
                }
                super::windows_store::ReplaceError::After(error) => {
                    let _ = error;
                    SettingsError::unknown_outcome()
                }
            })
    }
    /// `replace_owned` already flushes the renamed handle; Windows has no directory fsync to add.
    fn sync(&self) -> Result<(), SettingsError> {
        Ok(())
    }
}
fn write_generation(
    file: &mut File,
    value: &serde_json::Value,
    observer: &impl PersistObserver,
) -> Result<(), SettingsError> {
    observer
        .observe(PersistStage::BeforeWrite)
        .map_err(|_| persistence_failed())?;
    serde_json::to_writer(&mut *file, value).map_err(|_| persistence_failed())?;
    file.flush().map_err(|_| persistence_failed())?;
    observer
        .observe(PersistStage::AfterWrite)
        .map_err(|_| persistence_failed())?;
    file.sync_all().map_err(|_| persistence_failed())?;
    observer
        .observe(PersistStage::AfterFileSync)
        .map_err(|_| persistence_failed())?;
    Ok(())
}
fn private_temp(parent: &Path) -> Result<(PathBuf, File), SettingsError> {
    private_temp_with(parent, || TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed))
}
/// Create an exclusive temp named by PID and sequence, skipping names that already exist.
fn private_temp_with(
    parent: &Path,
    mut next_sequence: impl FnMut() -> u64,
) -> Result<(PathBuf, File), SettingsError> {
    for _ in 0..TEMP_NAME_ATTEMPTS {
        let path = parent.join(format!(
            ".settings-{}-{}.tmp",
            std::process::id(),
            next_sequence()
        ));
        match create_private(&path) {
            Ok(file) => return Ok((path, file)),
            // Occupied name: leave the existing file untouched and try the next sequence number.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(persistence_failed()),
        }
    }
    Err(persistence_failed())
}
fn create_private(path: &Path) -> std::io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
    }
    #[cfg(windows)]
    {
        super::windows_store::create_private(path)
    }
}
fn validate_temp(path: &Path, held: &File) -> Result<(), SettingsError> {
    let current = std::fs::symlink_metadata(path).map_err(|_| persistence_failed())?;
    super::locator::validate_owner(&current)?;
    if !current.is_file() {
        return Err(persistence_failed());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let identity = held.metadata().map_err(|_| persistence_failed())?;
        if (current.dev(), current.ino(), current.uid())
            != (identity.dev(), identity.ino(), identity.uid())
            || current.nlink() != 1
            || current.mode() & 0o777 != 0o600
        {
            return Err(persistence_failed());
        }
    }
    #[cfg(windows)]
    {
        let reopened = super::windows_store::open_checked(
            path,
            super::windows_store::ResourceKind::File,
            false,
        )
        .map_err(|_| persistence_failed())?;
        if super::windows_store::identity(&reopened).map_err(|_| persistence_failed())?
            != super::windows_store::identity(held).map_err(|_| persistence_failed())?
        {
            return Err(persistence_failed());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::private_temp_with;

    #[test]
    fn occupied_temporary_names_are_skipped_without_modification(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let pid = std::process::id();
        for sequence in 0..2 {
            let retained = directory
                .path()
                .join(format!(".settings-{pid}-{sequence}.tmp"));
            std::fs::write(retained, b"RETAINED")?;
        }
        let mut sequence = 0;
        let (path, _file) = private_temp_with(directory.path(), || {
            sequence += 1;
            sequence - 1
        })
        .map_err(|_| "temp creation failed")?;
        assert_eq!(
            path,
            directory.path().join(format!(".settings-{pid}-2.tmp"))
        );
        for sequence in 0..2 {
            let retained = directory
                .path()
                .join(format!(".settings-{pid}-{sequence}.tmp"));
            assert_eq!(std::fs::read(retained)?, b"RETAINED");
        }
        Ok(())
    }
    #[test]
    fn exhausted_temporary_names_fail_closed_without_deleting(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let retained = directory
            .path()
            .join(format!(".settings-{}-0.tmp", std::process::id()));
        std::fs::write(&retained, b"RETAINED")?;
        let error = private_temp_with(directory.path(), || 0)
            .err()
            .ok_or("occupied name reused")?;
        assert_eq!(
            error.code(),
            super::super::errors::SettingsErrorCode::PersistenceFailed
        );
        assert_eq!(std::fs::read(retained)?, b"RETAINED");
        Ok(())
    }
}
