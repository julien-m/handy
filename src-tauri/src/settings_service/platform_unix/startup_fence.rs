//! Hold a private startup lease independently of the settings transaction lock.

use super::resources::{
    lock_exclusive, open_private_file, unavailable, validate_private_file, PrivateDirectory,
    ResourceIdentity,
};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

/// A nonblocking OS lease separate from the transaction lock; drop releases ownership.
// @spec FR-004: Fence new startup against offline writes — .specs/features/008-settings-cli/spec.md#fr-004
pub(crate) struct StartupFence {
    file: File,
    path: PathBuf,
    identity: ResourceIdentity,
    directory: PrivateDirectory,
}

impl StartupFence {
    /// Acquire a stable owner-only lock file; no legacy listener or callback is created.
    pub(crate) fn acquire(directory: &PrivateDirectory, filename: &str) -> io::Result<Self> {
        if filename.is_empty() || filename.contains('/') || filename == "." || filename == ".." {
            return Err(unavailable());
        }
        directory.validate_current()?;
        let path = directory.path().join(filename);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
        {
            Ok(file) => {
                file.sync_all()?;
                directory.sync()?;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        let file = open_private_file(&path)?;
        lock_exclusive(&file)?;
        let fence = Self {
            identity: ResourceIdentity::from_metadata(&file.metadata()?),
            file,
            path,
            directory: PrivateDirectory::open(directory.path())?,
        };
        fence.validate_current()?;
        Ok(fence)
    }

    /// Refuse ownership if another pathname replaced the leased file or directory.
    pub(crate) fn validate_current(&self) -> io::Result<()> {
        self.directory.validate_current()?;
        let metadata = fs::symlink_metadata(&self.path)?;
        validate_private_file(&metadata)?;
        if ResourceIdentity::from_metadata(&metadata) != self.identity
            || ResourceIdentity::from_metadata(&self.file.metadata()?) != self.identity
        {
            return Err(unavailable());
        }
        Ok(())
    }
}
