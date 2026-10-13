//! Create private Unix sockets and authenticate their connected peers.

use super::resources::{current_uid, unavailable, PrivateDirectory, ResourceIdentity};
use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

/// New settings endpoint; never bind the historical singleton address.
/// Cleanup is explicit and refuses resources whose inode/owner changed.
// @spec FR-004: Private settings endpoint — .specs/features/008-settings-cli/spec.md#fr-004
pub(crate) struct PrivateSocket {
    path: PathBuf,
    listener: Option<UnixListener>,
    identity: ResourceIdentity,
    directory: PrivateDirectory,
}

impl PrivateSocket {
    /// Bind only a simple basename within a validated injected private directory.
    /// Never delete a stale socket or attempt to notify an existing instance.
    pub(crate) fn bind(directory: &PrivateDirectory, filename: &str) -> io::Result<Self> {
        Self::bind_with_creation_hook(directory, filename, |_| Ok(()))
    }

    /// Allow boundary tests to replace a newly created resource before revalidation.
    pub(super) fn bind_with_creation_hook(
        directory: &PrivateDirectory,
        filename: &str,
        after_creation: impl FnOnce(&Path) -> io::Result<()>,
    ) -> io::Result<Self> {
        if filename.is_empty() || filename.contains('/') || filename == "." || filename == ".." {
            return Err(unavailable());
        }
        directory.validate_current()?;
        let path = directory.path().join(filename);
        let listener = UnixListener::bind(&path)?;
        let created = fs::symlink_metadata(&path)?;
        validate_private_socket(&created)?;
        let identity = ResourceIdentity::from_metadata(&created);
        after_creation(&path)?;
        directory.validate_current()?;
        let metadata = fs::symlink_metadata(&path)?;
        validate_private_socket(&metadata)?;
        if ResourceIdentity::from_metadata(&metadata) != identity {
            return Err(unavailable());
        }
        Ok(Self {
            path,
            listener: Some(listener),
            identity,
            directory: PrivateDirectory::open(directory.path())?,
        })
    }

    /// Return the new settings endpoint path (boundary tests).
    #[cfg(test)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Set nonblocking mode (boundary tests; the endpoint accepts on a dedicated blocking thread).
    #[cfg(test)]
    pub(crate) fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.listener
            .as_ref()
            .ok_or_else(unavailable)?
            .set_nonblocking(nonblocking)
    }

    /// Authenticate an accepted peer before returning its stream; no frame is consumed.
    pub(crate) fn accept(&self) -> io::Result<(UnixStream, std::os::unix::net::SocketAddr)> {
        self.accept_for_uid(current_uid())
    }

    /// Share the native owner boundary with deterministic identity-mismatch tests.
    pub(super) fn accept_for_uid(
        &self,
        expected_uid: u32,
    ) -> io::Result<(UnixStream, std::os::unix::net::SocketAddr)> {
        self.directory.validate_current()?;
        let (stream, address) = self.listener.as_ref().ok_or_else(unavailable)?.accept()?;
        // Authenticate kernel identity before publishing a stream or reading any frame.
        verify_peer_uid(&stream, expected_uid)?;
        Ok((stream, address))
    }

    /// Bind like `bind`, reclaiming a stale name only with lease proof: `prove_lease` must show
    /// that the caller holds this endpoint's lease, so no live listener of the endpoint can
    /// exist. A name that still accepts connections is refused and never removed. The stat and
    /// unlink below cannot exclude a same-user replacement in between; inside the owner-only
    /// directory only the same user can do that, which is outside the threat model.
    pub(crate) fn bind_reclaiming(
        directory: &PrivateDirectory,
        filename: &str,
        prove_lease: impl Fn() -> io::Result<()>,
    ) -> io::Result<Self> {
        match Self::bind(directory, filename) {
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                prove_lease()?;
                remove_stale(directory, filename)?;
                prove_lease()?;
                Self::bind(directory, filename)
            }
            other => other,
        }
    }

    /// Close this listener and remove its name while `prove_lease` shows that the endpoint lease
    /// is still held, so a normal restart finds no stale name.
    pub(crate) fn close_and_remove(
        &mut self,
        prove_lease: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        self.close_owned_listener()?;
        prove_lease()?;
        self.directory.validate_current()?;
        let metadata = fs::symlink_metadata(&self.path)?;
        if ResourceIdentity::from_metadata(&metadata) != self.identity {
            return Err(unavailable());
        }
        fs::remove_file(&self.path)
    }

    /// Close only this listener; retain its socket name for later proven reclamation.
    /// No stat/unlink pair can atomically exclude same-UID pathname replacement.
    /// Routing must prove the lease is dead before reclaiming a stale endpoint.
    pub(crate) fn close_owned_listener(&mut self) -> io::Result<()> {
        self.directory.validate_current()?;
        let metadata = fs::symlink_metadata(&self.path)?;
        if !metadata.file_type().is_socket()
            || ResourceIdentity::from_metadata(&metadata) != self.identity
        {
            return Err(unavailable());
        }
        self.listener.take();
        Ok(())
    }
}

/// Remove a same-user socket name that no longer accepts connections.
fn remove_stale(directory: &PrivateDirectory, filename: &str) -> io::Result<()> {
    directory.validate_current()?;
    let path = directory.path().join(filename);
    validate_private_socket(&fs::symlink_metadata(&path)?)?;
    if UnixStream::connect(&path).is_ok() {
        return Err(unavailable());
    }
    fs::remove_file(&path)
}

fn validate_private_socket(metadata: &fs::Metadata) -> io::Result<()> {
    if !metadata.file_type().is_socket() || metadata.uid() != current_uid() || metadata.nlink() != 1
    {
        return Err(unavailable());
    }
    Ok(())
}

/// Authenticate the effective UID of the connected local peer (boundary tests; the endpoint
/// passes its expected UID to `verify_peer_uid`).
#[cfg(test)]
pub(crate) fn verify_peer(stream: &UnixStream) -> io::Result<()> {
    verify_peer_uid(stream, current_uid())
}

/// Check a peer against the endpoint owner; exposed to deterministic identity tests.
pub(crate) fn verify_peer_uid(stream: &UnixStream, expected_uid: u32) -> io::Result<()> {
    if peer_uid(stream)? != expected_uid {
        return Err(unavailable());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    let mut uid: libc::uid_t = 0;
    let mut gid: libc::gid_t = 0;
    // SAFETY: the live fd and initialized UID/GID outputs are valid for getpeereid.
    if unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(uid)
}

#[cfg(target_os = "linux")]
fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    // SAFETY: libc::ucred is a plain C struct; getsockopt fills its fields below.
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the writable ucred buffer matches length, and stream owns the live fd.
    if unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut credentials as *mut _ as *mut libc::c_void,
            &mut length,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    if length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(unavailable());
    }
    Ok(credentials.uid)
}
