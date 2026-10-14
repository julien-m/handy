//! Exchange files on disk: an export is written atomically (private temporary file in the
//! destination directory, flushed, then renamed over the destination), so a failed or interrupted
//! write leaves an existing destination intact and never a partial file. An existing destination
//! is replaced without confirmation and keeps its permissions; a symbolic link is kept and its
//! target receives the export, like a shell redirection.

use std::fs::{File, OpenOptions, Permissions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `content` to `path` atomically.
///
/// Side effects: creates and removes a temporary file next to the destination; replaces the
/// destination, which is `path` or, when `path` is a symbolic link, the file it points to.
///
/// # Errors
/// Any I/O failure (missing directory, dangling link, permission, full disk); the destination is
/// then unchanged and the temporary file is removed.
pub fn write_exchange_file(path: &Path, content: &str) -> std::io::Result<()> {
    let destination = resolve_destination(path)?;
    let permissions = existing_permissions(&destination)?;
    let temporary = temporary_path(&destination)?;
    // Order matters: the bytes and the permissions are durable in the temporary file before the
    // rename publishes them, so the destination is either the old file or the complete new one;
    // the directory is synced last so the rename itself survives a crash.
    let written = write_synced(&temporary, content, permissions)
        .and_then(|()| std::fs::rename(&temporary, &destination))
        .and_then(|()| sync_directory(&destination));
    if written.is_err() {
        // Best effort: the temporary may not exist if creating it failed; the error is returned.
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

/// The file to replace: `path`, or the file a symbolic link `path` points to (the link stays a
/// link). A dangling link fails with `NotFound` instead of being replaced by a regular file.
fn resolve_destination(path: &Path) -> std::io::Result<PathBuf> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => std::fs::canonicalize(path),
        _ => Ok(path.to_path_buf()),
    }
}

/// Permissions of an existing destination, kept by its replacement; `None` for a new file, which
/// gets the default permissions of a new file.
fn existing_permissions(path: &Path) -> std::io::Result<Option<Permissions>> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.permissions())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
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
    permissions: Option<Permissions>,
) -> std::io::Result<()> {
    // create_new: never write through a file or link that already has this name.
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(content.as_bytes())?;
    keep_permissions(&file, permissions)?;
    file.sync_all()
}

/// Unix: the mode of the replaced file. Windows only has a read-only flag, and a read-only
/// destination already refuses the rename, so the new file keeps its default attributes.
#[cfg(unix)]
fn keep_permissions(file: &File, permissions: Option<Permissions>) -> std::io::Result<()> {
    permissions.map_or(Ok(()), |permissions| file.set_permissions(permissions))
}
#[cfg(not(unix))]
fn keep_permissions(_file: &File, _permissions: Option<Permissions>) -> std::io::Result<()> {
    // Both parameters are unused on purpose: see the Unix variant.
    Ok(())
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
mod tests {
    use super::write_exchange_file;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn replace_an_existing_file_and_leave_no_temporary() -> TestResult {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("rules.txt");
        std::fs::write(&path, "old")?;
        write_exchange_file(&path, "recognized;replacement\na;b\n")?;
        assert_eq!(
            std::fs::read_to_string(&path)?,
            "recognized;replacement\na;b\n"
        );
        assert_eq!(std::fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }

    #[test]
    fn a_failed_write_keeps_the_destination_and_removes_the_temporary() -> TestResult {
        let directory = tempfile::tempdir()?;
        let missing = directory.path().join("missing").join("rules.txt");
        assert!(write_exchange_file(&missing, "a;b\n").is_err());
        // A directory cannot be replaced by a file: the rename fails after the temporary exists.
        let occupied = directory.path().join("occupied");
        std::fs::create_dir(&occupied)?;
        std::fs::write(occupied.join("kept"), "kept")?;
        assert!(write_exchange_file(&occupied, "a;b\n").is_err());
        assert_eq!(std::fs::read_to_string(occupied.join("kept"))?, "kept");
        assert_eq!(std::fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn an_existing_destination_keeps_its_permissions() -> TestResult {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("rules.txt");
        std::fs::write(&path, "old")?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        write_exchange_file(&path, "a;b\n")?;
        let mode = std::fs::metadata(&path)?.permissions().mode() & 0o777;
        assert_eq!(
            (std::fs::read_to_string(&path)?, mode),
            ("a;b\n".into(), 0o600)
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_stays_and_its_target_gets_the_export() -> TestResult {
        let directory = tempfile::tempdir()?;
        let target = directory.path().join("target.txt");
        let link = directory.path().join("link.txt");
        std::fs::write(&target, "old")?;
        std::os::unix::fs::symlink(&target, &link)?;
        write_exchange_file(&link, "a;b\n")?;
        assert!(std::fs::symlink_metadata(&link)?.file_type().is_symlink());
        assert_eq!(std::fs::read_to_string(&target)?, "a;b\n");
        let dangling = directory.path().join("dangling.txt");
        std::os::unix::fs::symlink(directory.path().join("absent.txt"), &dangling)?;
        assert!(write_exchange_file(&dangling, "a;b\n").is_err());
        assert!(std::fs::symlink_metadata(&dangling)?
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read_dir(directory.path())?.count(), 3);
        Ok(())
    }
}
