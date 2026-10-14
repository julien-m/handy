//! Exchange files on disk: an export is written atomically (private temporary file in the
//! destination directory, flushed, then renamed over the destination), so a failed or interrupted
//! write leaves an existing destination intact and never a partial file.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `content` to `path` atomically.
///
/// Side effects: creates and removes a temporary file next to `path`; replaces `path`.
///
/// # Errors
/// Any I/O failure (missing directory, permission, full disk); the destination is then unchanged
/// and the temporary file is removed.
pub fn write_exchange_file(path: &Path, content: &str) -> std::io::Result<()> {
    let temporary = temporary_path(path)?;
    // Order matters: the bytes are durable in the temporary file before the rename publishes
    // them, so the destination is either the old file or the complete new one.
    let written =
        write_synced(&temporary, content).and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        // Best effort: the temporary may not exist if creating it failed; the error is returned.
        let _ = std::fs::remove_file(&temporary);
    }
    written
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

fn write_synced(path: &Path, content: &str) -> std::io::Result<()> {
    // create_new: never write through a file or link that already has this name.
    let mut file: File = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()
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
}
