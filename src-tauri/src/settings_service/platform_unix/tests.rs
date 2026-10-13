//! Native authentication, socket replacement and unrelated-file permission tests.

use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{fs, io};

fn fixture() -> io::Result<(tempfile::TempDir, PrivateDirectory)> {
    let temp = tempfile::tempdir()?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))?;
    let directory = PrivateDirectory::open(&temp.path().canonicalize()?)?;
    Ok((temp, directory))
}

#[test]
fn native_peer_identity_mismatch_never_returns_stream() -> io::Result<()> {
    let (_temp, directory) = fixture()?;
    let socket = PrivateSocket::bind(&directory, "settings.sock")?;
    socket.set_nonblocking(true)?;
    let _client = std::os::unix::net::UnixStream::connect(socket.path())?;
    let error = socket
        .accept_for_uid(current_uid().wrapping_add(1))
        .expect_err("a native peer with another expected owner must not be published");
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    Ok(())
}

#[test]
fn bind_rejects_regular_replacement_without_changing_permissions() -> io::Result<()> {
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("settings.sock");
    let outcome = PrivateSocket::bind_with_creation_hook(&directory, "settings.sock", |path| {
        fs::remove_file(path)?;
        fs::write(path, b"retained")?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o640))
    });
    assert!(outcome.is_err());
    assert_eq!(fs::symlink_metadata(&path)?.mode() & 0o777, 0o640);
    assert_eq!(fs::read(path)?, b"retained");
    Ok(())
}

#[test]
fn bind_rejects_socket_replacement_with_matching_owner_and_mode() -> io::Result<()> {
    let (_temp, directory) = fixture()?;
    let mut replacement = None;
    let mut replacement_mode = None;
    let outcome = PrivateSocket::bind_with_creation_hook(&directory, "settings.sock", |path| {
        fs::remove_file(path)?;
        replacement = Some(std::os::unix::net::UnixListener::bind(path)?);
        replacement_mode = Some(fs::symlink_metadata(path)?.mode() & 0o777);
        Ok(())
    });
    assert!(outcome.is_err());
    assert_eq!(
        fs::symlink_metadata(directory.path().join("settings.sock"))?.mode() & 0o777,
        replacement_mode.ok_or_else(|| io::Error::other("missing replacement mode"))?
    );
    assert!(replacement.is_some());
    Ok(())
}

#[test]
fn concurrent_socket_binds_preserve_unrelated_file_creation_permissions() -> io::Result<()> {
    let (_temp, directory) = fixture()?;
    let before = directory.path().join("unrelated-before");
    fs::write(&before, b"before")?;
    let expected_mode = fs::symlink_metadata(&before)?.mode() & 0o777;
    let path = directory.path().to_path_buf();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let worker_barrier = std::sync::Arc::clone(&barrier);
    // Thirty-two pairs exercise interleaving while keeping native fixture work small;
    // the source gate proves absence of mask writes, rather than relying on stress alone.
    const SAMPLE_COUNT: usize = 32;
    let worker = std::thread::spawn(move || -> io::Result<Vec<u32>> {
        worker_barrier.wait();
        (0..SAMPLE_COUNT)
            .map(|index| {
                let file = path.join(format!("unrelated-{index}"));
                fs::write(&file, b"retained")?;
                Ok(fs::symlink_metadata(file)?.mode() & 0o777)
            })
            .collect()
    });
    barrier.wait();
    for index in 0..SAMPLE_COUNT {
        let socket = PrivateSocket::bind(&directory, &format!("settings-{index}.sock"))?;
        drop(socket);
    }
    let modes = worker
        .join()
        .map_err(|_| io::Error::other("creator thread failed"))??;
    assert!(modes.iter().all(|mode| *mode == expected_mode));
    assert_eq!(fs::read(before)?, b"before");
    Ok(())
}
