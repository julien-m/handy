//! Unix endpoint of the private settings channel: a socket in a current-user 0700 runtime
//! directory, named after the store identity and guarded by a lease beside it. Both ends check
//! the peer's kernel UID before any application byte; no network listener exists.
use super::ipc::{Connection, Duplex};
use super::platform_unix::{
    current_uid, verify_peer_uid, PrivateDirectory, PrivateSocket, StartupFence,
};
use std::io;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

// Socket paths must fit in sun_path (104 bytes on macOS): a short directory and 16 hex digits
// (64 bits of the SHA-256 identity) keep the full path well below it. Requests still carry the
// complete identity, which the instance checks.
const NAME_HEX_DIGITS: usize = 16;

impl Duplex for UnixStream {
    fn limit(&self, timeout: Duration) -> io::Result<()> {
        self.set_read_timeout(Some(timeout))?;
        self.set_write_timeout(Some(timeout))
    }
}

/// Where the endpoint of one store lives for the current user.
pub(crate) struct Endpoint {
    directory: PathBuf,
    name: String,
}
/// Published listener and the lease that proves its name belongs to this instance.
pub(crate) struct Listener {
    socket: PrivateSocket,
    lease: StartupFence,
}

impl Endpoint {
    /// Resolve the runtime directory without creating anything.
    pub(crate) fn for_identity(identity: &str) -> io::Result<Self> {
        let name = identity
            .get(..NAME_HEX_DIGITS)
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        Ok(Self {
            directory: runtime_base()?.join(format!("hs-{}", current_uid())),
            name: name.to_owned(),
        })
    }
    /// Connect and authenticate the listener as the current user before returning it.
    pub(crate) fn connect(&self) -> Connection {
        self.connect_as(current_uid())
    }
    /// `connect` with an injectable expected owner for identity-mismatch tests.
    pub(crate) fn connect_as(&self, expected_uid: u32) -> Connection {
        match PrivateDirectory::open(&self.directory) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Connection::NotListening
            }
            Err(_) => return Connection::Refused,
            Ok(_) => {}
        }
        match UnixStream::connect(self.directory.join(&self.name)) {
            Err(error) if not_listening(&error) => Connection::NotListening,
            Err(_) => Connection::Refused,
            Ok(stream) => match verify_peer_uid(&stream, expected_uid) {
                Ok(()) => Connection::Connected(Box::new(stream)),
                Err(_) => Connection::Refused,
            },
        }
    }
    /// Publish the endpoint: take its lease, then bind, reclaiming a stale name under the lease.
    pub(crate) fn listen(&self) -> io::Result<Listener> {
        let directory = private_directory(&self.directory)?;
        let lease = StartupFence::acquire(&directory, &format!("{}.lease", self.name))?;
        let socket =
            PrivateSocket::bind_reclaiming(&directory, &self.name, || lease.validate_current())?;
        Ok(Listener { socket, lease })
    }
    /// Pathname of the socket, for lifecycle tests.
    pub(crate) fn path(&self) -> PathBuf {
        self.directory.join(&self.name)
    }
}
impl Listener {
    /// Next authenticated connection; a peer of another user is closed before any byte.
    pub(crate) fn accept(&self) -> io::Result<Box<dyn Duplex>> {
        let (stream, _) = self.socket.accept()?;
        Ok(Box::new(stream))
    }
    /// Stop listening and remove the name while the lease is still held.
    pub(crate) fn close(mut self) -> io::Result<()> {
        let lease = &self.lease;
        self.socket.close_and_remove(|| lease.validate_current())
    }
}
/// A missing name, or a stale one whose listener is gone.
fn not_listening(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
    )
}
/// Create the owner-only directory once (no umask or chmod change), then pin and validate it.
fn private_directory(path: &std::path::Path) -> io::Result<PrivateDirectory> {
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new().mode(0o700).create(path) {
        Err(error) if error.kind() != io::ErrorKind::AlreadyExists => return Err(error),
        _ => {}
    }
    PrivateDirectory::open(path)
}
/// Per-user runtime base, independent of the caller's environment where the OS provides one, so
/// a CLI started from another shell or agent sandbox finds the same endpoint as the app.
fn runtime_base() -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = darwin_user_temp_dir()?;
    #[cfg(not(target_os = "macos"))]
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(std::env::temp_dir);
    base.canonicalize()
}
/// confstr(_CS_DARWIN_USER_TEMP_DIR): the per-user temporary directory launchd also gives the
/// app, without reading TMPDIR. Returns the length including the terminator, 0 on failure.
#[cfg(target_os = "macos")]
fn darwin_user_temp_dir() -> io::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let mut buffer = vec![0u8; libc::PATH_MAX as usize];
    // SAFETY: the buffer is writable for its whole length, which is passed as the bound.
    let length = unsafe {
        libc::confstr(
            libc::_CS_DARWIN_USER_TEMP_DIR,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    if length == 0 || length > buffer.len() {
        return Err(io::Error::last_os_error());
    }
    buffer.truncate(length - 1);
    Ok(PathBuf::from(std::ffi::OsString::from_vec(buffer)))
}
