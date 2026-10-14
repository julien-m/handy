//! Windows endpoint of the private settings channel: a named pipe keyed by the logon SID and the
//! store identity, with a protected current-logon DACL and no remote clients. The server checks
//! the client's logon SID before any byte and the client checks the server's before writing.
use super::ipc::{Accept, Connection, Duplex, Incoming, Publish};
use super::platform_windows::{
    current_logon_sid, pipe_name, verify_server, PrivatePipe, ServerPipe,
};
use std::fs::File;
use std::io;
use std::time::Duration;

// ERROR_PIPE_BUSY: no instance is free at this moment (the server is between two waits or at
// its instance bound); retried like an absent endpoint until the deadline.
const ERROR_PIPE_BUSY: i32 = 231;

/// Client end: synchronous, with no read timeout; the client deadline is enforced by its worker
/// thread.
impl Duplex for File {
    fn limit(&self, _timeout: Duration) -> io::Result<()> {
        Ok(())
    }
}
/// Server end: each later read and write is bounded by `timeout`.
impl Duplex for ServerPipe {
    fn limit(&self, timeout: Duration) -> io::Result<()> {
        self.set_timeout(Some(timeout));
        Ok(())
    }
}

/// The pipe of one store for the current logon session.
pub(crate) struct Endpoint {
    name: String,
    logon_sid: String,
    identity: String,
}
/// Serving side: one instance waits for the next client while earlier ones are served.
pub(crate) struct Listener {
    identity: String,
    waiting: Option<PrivatePipe>,
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
    /// Publish the name with its first instance, refused while any server (live or squatting)
    /// holds it. Pipes disappear with their process, so there is no stale name to reclaim.
    pub(crate) fn listen(&self) -> io::Result<Listener> {
        Ok(Listener {
            identity: self.identity.clone(),
            waiting: Some(PrivatePipe::create(&self.identity, true)?),
        })
    }
}
impl Publish for Endpoint {
    fn publish(&self) -> io::Result<Box<dyn Accept>> {
        Ok(Box::new(self.listen()?))
    }
}
impl Accept for Listener {
    /// Next authenticated client within `wait`. An error means no instance could be created or
    /// waited on; the server then backs off.
    fn accept(&mut self, wait: Duration) -> io::Result<Incoming> {
        let pipe = match self.waiting.take() {
            Some(pipe) => pipe,
            None => PrivatePipe::create(&self.identity, false)?,
        };
        if !pipe.wait_for_client(wait)? {
            self.waiting = Some(pipe);
            return Ok(Incoming::Idle);
        }
        // Order matters: the next instance exists before this one is served, so the name never
        // lapses and the next client never waits for this request. A creation failure (instance
        // bound reached) is retried by the next accept, which backs off if it fails again.
        self.waiting = PrivatePipe::create(&self.identity, false).ok();
        Ok(match pipe.authenticate() {
            Ok(server) => Incoming::Peer(Box::new(server)),
            Err(_) => Incoming::Refused,
        })
    }
    /// Pipe names cannot be removed or replaced from outside while an instance is open.
    fn published(&self) -> bool {
        true
    }
    /// Nothing to remove: no pipe instance outlives its handle.
    fn close(self: Box<Self>) -> io::Result<()> {
        Ok(())
    }
}
