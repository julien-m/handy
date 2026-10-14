//! Exchange files on disk: an export is written atomically (private temporary file in the
//! destination directory, created with its final permissions, flushed, then renamed over the
//! destination), so a failed or interrupted write leaves an existing destination intact and never
//! a partial file. An existing destination is replaced without confirmation and keeps its
//! permissions; a new one is private (0600 on Unix). Orchestrator decision (2026-10-14, delivery
//! review): a destination that is a symbolic link (whatever its target) or a read-only file is
//! refused before anything is written, so an export never lands in a file the user did not name
//! nor replaces one the user protected.

use std::fs::{File, OpenOptions, Permissions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Mode of a new export file on Unix: only its owner can read the rules.
#[cfg(unix)]
const NEW_FILE_MODE: u32 = 0o600;

/// Why an export was not written; the destination is unchanged in every case.
#[derive(Debug)]
pub enum ExportError {
    /// The destination is a symbolic link (dangling or not).
    SymbolicLink,
    /// The destination is a file the user cannot write.
    ReadOnly,
    /// Any other I/O failure (missing directory, a directory in the way, permission, full disk).
    Io(std::io::Error),
}

/// A completed export: the destination holds the new content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// The rename was flushed to the directory, so it survives a crash.
    Durable,
    /// The directory could not be synced after the rename: the file is replaced, but a crash
    /// right now could still bring back the old content.
    NotSynced,
}

/// Write `content` to `path` atomically.
///
/// Side effects: creates and removes a temporary file next to `path`, then replaces `path`.
///
/// # Errors
/// `SymbolicLink` or `ReadOnly` before anything is written; `Io` for any other failure, after
/// which the temporary file is removed. The destination is unchanged in every case.
pub fn write_exchange_file(path: &Path, content: &str) -> Result<Written, ExportError> {
    let permissions = writable_destination(path)?;
    let temporary = temporary_path(path).map_err(ExportError::Io)?;
    // Order matters: the bytes and the permissions are durable in the temporary file before the
    // rename publishes them, so the destination is either the old file or the complete new one.
    let written = write_synced(&temporary, content, permissions.as_ref())
        .and_then(|()| std::fs::rename(&temporary, path));
    if let Err(error) = written {
        // Best effort: the temporary may not exist if creating it failed; the error is returned.
        let _ = std::fs::remove_file(&temporary);
        return Err(ExportError::Io(error));
    }
    // The destination already holds the export: a failed directory sync only leaves the rename's
    // durability unconfirmed, which the caller reports instead of claiming nothing changed.
    Ok(match sync_directory(path) {
        Ok(()) => Written::Durable,
        Err(_) => Written::NotSynced,
    })
}

/// Permissions of the existing destination, kept by its replacement (`None` for a new file),
/// after refusing a symbolic link and a regular file the user cannot write.
fn writable_destination(path: &Path) -> Result<Option<Permissions>, ExportError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ExportError::Io(error)),
    };
    if metadata.file_type().is_symlink() {
        return Err(ExportError::SymbolicLink);
    }
    if metadata.permissions().readonly() {
        return Err(ExportError::ReadOnly);
    }
    if metadata.is_file() {
        probe_write_access(path)?;
    }
    Ok(Some(metadata.permissions()))
}

/// Open the existing regular file for writing, without truncating or changing it, so the system
/// decides like a shell redirection whether this user may write it (owner, ACL, flags).
fn probe_write_access(path: &Path) -> Result<(), ExportError> {
    match OpenOptions::new().write(true).open(path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(ExportError::ReadOnly)
        }
        Err(error) => Err(ExportError::Io(error)),
    }
}

/// Same directory as `path`, hidden, unique to this process and instant.
fn temporary_path(path: &Path) -> std::io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let unique = format!(
        ".{}.{}-{nanos:x}.tmp",
        name.to_string_lossy(),
        std::process::id()
    );
    Ok(path.with_file_name(unique))
}

fn write_synced(
    path: &Path,
    content: &str,
    permissions: Option<&Permissions>,
) -> std::io::Result<()> {
    let mut file = create_private(path, permissions)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()
}

/// Unix: create the temporary with its final mode (the replaced file's, or `NEW_FILE_MODE`) and
/// set that mode exactly before any byte is written, so the rules are never readable more
/// widely than in the final file. `create_new` never writes through an existing name or link.
#[cfg(unix)]
fn create_private(path: &Path, permissions: Option<&Permissions>) -> std::io::Result<File> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let mode = permissions.map_or(NEW_FILE_MODE, |permissions| permissions.mode() & 0o7777);
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)?;
    // The umask may have narrowed the creation mode; the final mode is applied as is.
    file.set_permissions(Permissions::from_mode(mode))?;
    Ok(file)
}
/// Windows only has a read-only flag, and a read-only destination is refused before, so the new
/// file keeps its default attributes.
#[cfg(not(unix))]
fn create_private(path: &Path, _permissions: Option<&Permissions>) -> std::io::Result<File> {
    // The permissions are unused on purpose: see above.
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Unix: fsync the destination's directory so the rename is durable. Windows has no directory
/// handle to sync; NTFS journals the rename itself.
#[cfg(unix)]
fn sync_directory(destination: &Path) -> std::io::Result<()> {
    let directory = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    File::open(directory)?.sync_all()
}
#[cfg(not(unix))]
fn sync_directory(_destination: &Path) -> std::io::Result<()> {
    // The parameter is unused on purpose: see the Unix variant.
    Ok(())
}

#[cfg(test)]
#[path = "exchange_file_tests.rs"]
mod tests;
