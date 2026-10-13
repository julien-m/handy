//! Windows endpoint of the private settings channel: a named pipe keyed by the logon SID and the
//! store identity, with a protected current-logon DACL and no remote clients. The server checks
//! the client's logon SID before any byte and the client checks the server's before writing.
use super::ipc::{Connection, Duplex};
use super::platform_windows::{current_logon_sid, pipe_name, verify_server, PrivatePipe};
use std::fs::File;
use std::io;
use std::time::Duration;

// ERROR_PIPE_BUSY: the single pipe instance is serving another client; retried like an absent
// endpoint until the deadline.
const ERROR_PIPE_BUSY: i32 = 231;

/// Synchronous pipes have no read timeout; the client deadline is enforced by its worker thread.
impl Duplex for File {
    fn limit(&self, _timeout: Duration) -> io::Result<()> {
        Ok(())
    }
}

/// The pipe of one store for the current logon session.
pub(crate) struct Endpoint {
    name: String,
    logon_sid: String,
    identity: String,
}
/// Serving side: one first-instance pipe is created per accepted connection.
pub(crate) struct Listener {
    identity: String,
}

impl Endpoint {
    /// Resolve the pipe name; nothing is created.
    pub(crate) fn for_identity(identity: &str) -> io::Result<Self> {
        let logon_sid = current_logon_sid()?;
        Ok(Self {
            name: pipe_name(&logon_sid, identity)?,
            logon_sid,
            identity: identity.to_owned(),
        })
    }
    /// Open the pipe and authenticate its server before returning it.
    pub(crate) fn connect(&self) -> Connection {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.name)
        {
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    || error.raw_os_error() == Some(ERROR_PIPE_BUSY) =>
            {
                Connection::NotListening
            }
            Err(_) => Connection::Refused,
            Ok(pipe) => match verify_server(&pipe, &self.logon_sid) {
                Ok(()) => Connection::Connected(Box::new(pipe)),
                Err(_) => Connection::Refused,
            },
        }
    }
    /// Pipes disappear with their process, so there is no stale name to reclaim.
    pub(crate) fn listen(&self) -> io::Result<Listener> {
        Ok(Listener {
            identity: self.identity.clone(),
        })
    }
}
impl Listener {
    /// Create the pipe instance and wait for one authenticated client.
    pub(crate) fn accept(&self) -> io::Result<Box<dyn Duplex>> {
        let pipe = PrivatePipe::create(&self.identity)?.accept()?;
        Ok(Box::new(pipe))
    }
    /// Nothing to remove: no pipe instance outlives its handle.
    pub(crate) fn close(self) -> io::Result<()> {
        Ok(())
    }
}
