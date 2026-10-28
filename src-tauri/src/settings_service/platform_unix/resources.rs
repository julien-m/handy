//! Pin and validate owner-only directories and lockable regular files.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// A private filesystem resource identity, independent of its pathname.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResourceIdentity {
    device: u64,
    inode: u64,
    owner: u32,
}

impl ResourceIdentity {
    /// Capture device, inode and effective owner from already-read metadata.
    pub(super) fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner: metadata.uid(),
        }
    }
}

/// Return the effective UID used to scope local resources.
pub(crate) fn current_uid() -> u32 {
    // SAFETY: geteuid has no pointer arguments and does not mutate Rust memory.
    unsafe { libc::geteuid() }
}

/// Return a payload-free permission error when private identity cannot be trusted.
pub(super) fn unavailable() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "private resource unavailable",
    )
}

/// A canonical owner-only directory pinned open for durable writes.
// @spec FR-004: Owner-only filesystem resources — .specs/features/008-settings-cli/spec.md#fr-004
pub(crate) struct PrivateDirectory {
    path: PathBuf,
    directory: File,
    identity: ResourceIdentity,
}

impl PrivateDirectory {
    /// Open an existing absolute canonical 0700 directory without creating it.
    /// Reject symlink aliases, ownership/mode ambiguity and replacement races.
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() || path.canonicalize()? != path {
            return Err(unavailable());
        }
        let metadata = fs::symlink_metadata(path)?;
        validate_directory(&metadata)?;
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let identity = ResourceIdentity::from_metadata(&metadata);
        if ResourceIdentity::from_metadata(&directory.metadata()?) != identity {
            return Err(unavailable());
        }
        Ok(Self {
            path: path.to_path_buf(),
            directory,
            identity,
        })
    }

    /// Return the injected private path; never the real singleton address.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Refuse to act if the private directory path has been replaced or opened.
    pub(crate) fn validate_current(&self) -> io::Result<()> {
        let metadata = fs::symlink_metadata(&self.path)?;
        validate_directory(&metadata)?;
        if ResourceIdentity::from_metadata(&metadata) != self.identity {
            return Err(unavailable());
        }
        Ok(())
    }

    /// Sync pinned directory entries after durable file creation or a completed rename.
    pub(crate) fn sync(&self) -> io::Result<()> {
        self.directory.sync_all()
    }
}

fn validate_directory(metadata: &fs::Metadata) -> io::Result<()> {
    if !metadata.is_dir() || metadata.uid() != current_uid() || metadata.mode() & 0o777 != 0o700 {
        return Err(unavailable());
    }
    Ok(())
}

/// Open a private regular single-link file, refusing symlink/type/owner aliases.
pub(crate) fn open_private_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    validate_private_file(&file.metadata()?)?;
    Ok(file)
}

/// Reject metadata unless it identifies an owner-only regular single-link file.
pub(super) fn validate_private_file(metadata: &fs::Metadata) -> io::Result<()> {
    if !metadata.is_file()
        || metadata.uid() != current_uid()
        || metadata.mode() & 0o777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(unavailable());
    }
    Ok(())
}

/// Hold a nonblocking OS file lock; failure never permits an offline mutation.
pub(crate) fn lock_exclusive(file: &File) -> io::Result<()> {
    // SAFETY: the descriptor is borrowed from a live File; flock has no pointers.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
