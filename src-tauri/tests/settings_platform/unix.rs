//! Private resource, passive legacy-owner and authenticated peer integration tests.

use super::{instance_probe, platform_unix};
use instance_probe::{inspect_legacy_path, LegacyState};
use platform_unix::{current_uid, Accepted, PrivateDirectory, PrivateSocket, StartupFence};
use std::fs;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

fn fixture() -> std::io::Result<(tempfile::TempDir, PrivateDirectory)> {
    let temp = tempfile::tempdir()?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))?;
    let directory = PrivateDirectory::open(&temp.path().canonicalize()?)?;
    Ok((temp, directory))
}

#[test]
fn absent_probe_creates_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("legacy.sock");
    assert_eq!(
        inspect_legacy_path(&path, current_uid()),
        LegacyState::Absent
    );
    assert!(!path.exists());
    Ok(())
}

#[test]
fn occupied_probe_never_notifies_or_modifies_store() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("legacy.sock");
    let store = directory.path().join("settings_store.json");
    fs::write(&store, b"{\"revision\":17}")?;
    let before = fs::read(&store)?;
    let listener = UnixListener::bind(&path)?;
    listener.set_nonblocking(true)?;
    let inode = fs::symlink_metadata(&path)?.ino();
    assert_eq!(
        inspect_legacy_path(&path, current_uid()),
        LegacyState::Occupied
    );
    assert_eq!(
        listener.accept().expect_err("passive probe").kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert_eq!(fs::symlink_metadata(&path)?.ino(), inode);
    assert_eq!(fs::read(store)?, before);
    Ok(())
}

#[test]
fn ambiguous_legacy_resources_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("legacy.sock");
    drop(UnixListener::bind(&path)?);
    assert_eq!(
        inspect_legacy_path(&path, current_uid()),
        LegacyState::Occupied
    );
    assert_eq!(
        inspect_legacy_path(&path, current_uid().wrapping_add(1)),
        LegacyState::Unavailable
    );
    fs::remove_file(&path)?;
    fs::write(&path, b"retained")?;
    assert_eq!(
        inspect_legacy_path(&path, current_uid()),
        LegacyState::Unavailable
    );
    let link = directory.path().join("alias.sock");
    symlink(&path, &link)?;
    assert_eq!(
        inspect_legacy_path(&link, current_uid()),
        LegacyState::Unavailable
    );
    assert_eq!(fs::read(path)?, b"retained");
    Ok(())
}

#[test]
fn private_directory_refuses_permissions_alias_and_replacement(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755))?;
    assert!(PrivateDirectory::open(directory.path()).is_err());
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))?;
    let link = directory.path().join("alias");
    symlink(directory.path(), &link)?;
    assert!(PrivateDirectory::open(&link).is_err());
    fs::remove_file(link)?;
    let moved = directory.path().with_extension("moved");
    let endpoint = directory.path().join("settings.sock");
    fs::rename(directory.path(), &moved)?;
    fs::create_dir(directory.path())?;
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))?;
    assert!(PrivateSocket::bind(&directory, "settings.sock").is_err());
    assert!(!endpoint.exists());
    fs::remove_dir_all(moved)?;
    Ok(())
}

#[test]
fn new_socket_authenticates_peer_and_refuses_replaced_path(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let mut socket = PrivateSocket::bind(&directory, "settings.sock")?;
    assert_eq!(
        fs::symlink_metadata(directory.path())?.mode() & 0o777,
        0o700
    );
    assert_eq!(fs::symlink_metadata(socket.path())?.uid(), current_uid());
    assert!(PrivateSocket::bind(&directory, "settings.sock").is_err());
    socket.set_nonblocking(true)?;
    let (wait, uid) = (Duration::from_millis(50), current_uid());
    assert!(matches!(socket.accept_within(wait, uid)?, Accepted::Idle));
    let client = UnixStream::connect(socket.path())?;
    let Accepted::Peer(server) = socket.accept_within(Duration::from_secs(1), uid)? else {
        return Err("the client of the same user was not accepted".into());
    };
    platform_unix::verify_peer(&server)?;
    assert!(platform_unix::verify_peer_uid(&server, current_uid().wrapping_add(1)).is_err());
    drop(client);
    fs::remove_file(socket.path())?;
    fs::write(socket.path(), b"replacement")?;
    assert!(socket.accept_within(wait, uid).is_err());
    assert!(socket.close_owned_listener().is_err());
    assert_eq!(fs::read(socket.path())?, b"replacement");
    Ok(())
}

#[test]
fn close_retains_socket_and_releases_lock_by_lifetime() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let mut socket = PrivateSocket::bind(&directory, "settings.sock")?;
    let path = socket.path().to_path_buf();
    socket.close_owned_listener()?;
    assert!(path.exists());
    assert!(UnixStream::connect(&path).is_err());
    let fence = StartupFence::acquire(&directory, "instance.lock")?;
    assert!(StartupFence::acquire(&directory, "instance.lock").is_err());
    fence.validate_current()?;
    drop(fence);
    assert!(StartupFence::acquire(&directory, "instance.lock").is_ok());
    Ok(())
}

#[test]
fn lock_and_file_refuse_hostile_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let target = directory.path().join("retained");
    fs::write(&target, b"retained")?;
    symlink(&target, directory.path().join("instance.lock"))?;
    assert!(StartupFence::acquire(&directory, "instance.lock").is_err());
    assert_eq!(fs::read(target)?, b"retained");
    Ok(())
}

#[test]
fn stale_socket_name_is_reclaimed_only_with_lease_proof() -> Result<(), Box<dyn std::error::Error>>
{
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("endpoint");
    drop(UnixListener::bind(&path)?);
    let denied = PrivateSocket::bind_reclaiming(&directory, "endpoint", || {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    });
    assert!(denied.is_err() && path.exists(), "no proof, no removal");
    let lease = StartupFence::acquire(&directory, "endpoint.lease")?;
    let mut socket =
        PrivateSocket::bind_reclaiming(&directory, "endpoint", || lease.validate_current())?;
    let _client = UnixStream::connect(&path)?;
    socket.close_and_remove(|| lease.validate_current())?;
    assert!(!path.exists(), "a proven close removes the name");
    Ok(())
}

#[test]
fn a_listening_socket_name_is_never_reclaimed() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, directory) = fixture()?;
    let path = directory.path().join("endpoint");
    let _live = UnixListener::bind(&path)?;
    let inode = fs::symlink_metadata(&path)?.ino();
    let lease = StartupFence::acquire(&directory, "endpoint.lease")?;
    let outcome =
        PrivateSocket::bind_reclaiming(&directory, "endpoint", || lease.validate_current());
    assert!(outcome.is_err());
    assert_eq!(fs::symlink_metadata(&path)?.ino(), inode);
    Ok(())
}
