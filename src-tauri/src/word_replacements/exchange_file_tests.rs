//! Export file writes: atomic replacement, refused symbolic links and read-only files, private
//! permissions from creation, and an unconfirmed directory sync after a successful rename.
use super::{write_exchange_file, ExportError, Written};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn replace_an_existing_file_and_leave_no_temporary() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rules.txt");
    std::fs::write(&path, "old")?;
    let written = write_exchange_file(&path, "recognized;replacement\na;b\n");
    assert!(matches!(written, Ok(Written::Durable)));
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
    assert!(matches!(
        write_exchange_file(&missing, "a;b\n"),
        Err(ExportError::Io(_))
    ));
    // A directory cannot be replaced by a file: the rename fails after the temporary exists.
    let occupied = directory.path().join("occupied");
    std::fs::create_dir(&occupied)?;
    std::fs::write(occupied.join("kept"), "kept")?;
    assert!(matches!(
        write_exchange_file(&occupied, "a;b\n"),
        Err(ExportError::Io(_))
    ));
    assert_eq!(std::fs::read_to_string(occupied.join("kept"))?, "kept");
    assert_eq!(std::fs::read_dir(directory.path())?.count(), 1);
    Ok(())
}

#[cfg(unix)]
fn mode(path: &std::path::Path) -> std::io::Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
}

#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(unix)]
#[test]
fn an_existing_destination_keeps_its_permissions_and_a_new_one_is_private() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rules.txt");
    std::fs::write(&path, "old")?;
    set_mode(&path, 0o640)?;
    write_exchange_file(&path, "a;b\n").map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        (std::fs::read_to_string(&path)?, mode(&path)?),
        ("a;b\n".into(), 0o640)
    );
    let new = directory.path().join("new.txt");
    write_exchange_file(&new, "a;b\n").map_err(|error| format!("{error:?}"))?;
    assert_eq!(mode(&new)?, 0o600);
    Ok(())
}

#[cfg(unix)]
#[test]
fn the_temporary_file_has_its_final_mode_before_any_byte_is_written() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let new = directory.path().join(".new.tmp");
    let file = super::create_private(&new, None)?;
    assert_eq!((file.metadata()?.len(), mode(&new)?), (0, 0o600));
    let kept = directory.path().join(".kept.tmp");
    let replaced = std::fs::Permissions::from_mode(0o640);
    let file = super::create_private(&kept, Some(&replaced))?;
    assert_eq!((file.metadata()?.len(), mode(&kept)?), (0, 0o640));
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_is_refused_and_nothing_is_written() -> TestResult {
    let directory = tempfile::tempdir()?;
    let target = directory.path().join("target.txt");
    let link = directory.path().join("link.txt");
    std::fs::write(&target, "old")?;
    std::os::unix::fs::symlink(&target, &link)?;
    let dangling = directory.path().join("dangling.txt");
    std::os::unix::fs::symlink(directory.path().join("absent.txt"), &dangling)?;
    for path in [&link, &dangling] {
        let written = write_exchange_file(path, "a;b\n");
        assert!(matches!(written, Err(ExportError::SymbolicLink)));
        assert!(std::fs::symlink_metadata(path)?.file_type().is_symlink());
    }
    assert_eq!(std::fs::read_to_string(&target)?, "old");
    assert_eq!(std::fs::read_dir(directory.path())?.count(), 3);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_destination_the_user_cannot_write_is_refused_and_unchanged() -> TestResult {
    let directory = tempfile::tempdir()?;
    // 0444 has no write bit at all; 0466 lets others write but not the owner.
    for (name, mode_bits) in [("read-only.txt", 0o444), ("owner-locked.txt", 0o466)] {
        let path = directory.path().join(name);
        std::fs::write(&path, "old")?;
        set_mode(&path, mode_bits)?;
        let written = write_exchange_file(&path, "a;b\n");
        assert!(matches!(written, Err(ExportError::ReadOnly)), "{name}");
        assert_eq!(
            (std::fs::read_to_string(&path)?, mode(&path)?),
            ("old".into(), mode_bits)
        );
    }
    assert_eq!(std::fs::read_dir(directory.path())?.count(), 2);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_failed_directory_sync_after_the_rename_is_reported_as_not_synced() -> TestResult {
    let directory = tempfile::tempdir()?;
    let unreadable = directory.path().join("write-only");
    std::fs::create_dir(&unreadable)?;
    let path = unreadable.join("rules.txt");
    std::fs::write(&path, "old")?;
    // Write and search but no read: the rename works, opening the directory to sync it fails.
    set_mode(&unreadable, 0o300)?;
    let written = write_exchange_file(&path, "new\n");
    set_mode(&unreadable, 0o700)?;
    assert!(matches!(written, Ok(Written::NotSynced)));
    assert_eq!(std::fs::read_to_string(&path)?, "new\n");
    assert_eq!(std::fs::read_dir(&unreadable)?.count(), 1);
    Ok(())
}
