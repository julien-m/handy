//! Private local channel between the CLI and the running instance: one request per connection,
//! length-prefixed frames bounded to 1 MiB, peers authenticated by the platform endpoint before
//! any application byte, and a total 5 s deadline after which the outcome is unknown.
#[cfg(unix)]
use super::endpoint_unix::Endpoint;
#[cfg(windows)]
use super::endpoint_windows::Endpoint;
use super::{
    errors::{SettingsError, SettingsErrorCode},
    operations::{execute, Reply, SettingsBackend},
    protocol::{OperationRequest, ProtocolVersion, SettingsOperation},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[path = "ipc_server.rs"]
mod server;
pub use server::{serve, EndpointHandle};

/// Total answer deadline of the specification, including lock, persistence and transport waits.
pub const DEADLINE: Duration = Duration::from_secs(5);
/// Technical frame bound of the plan; domain imports share it until their owner sets another.
pub const MAX_FRAME_BYTES: usize = 1 << 20;

/// A connected byte stream of the platform endpoint.
pub(crate) trait Duplex: Read + Write + Send {
    /// Bound blocking reads and writes where the platform supports it.
    fn limit(&self, timeout: Duration) -> io::Result<()>;
}
/// Result of one connection attempt.
pub(crate) enum Connection {
    /// Authenticated instance endpoint, ready for one request.
    Connected(Box<dyn Duplex>),
    /// Nobody serves the endpoint (absent, stale or busy between connections).
    NotListening,
    /// Something answered but failed authentication or privacy checks: never write to it.
    Refused,
}
/// What one bounded wait on the listener produced.
pub(crate) enum Incoming {
    /// An authenticated peer, ready for one request.
    Peer(Box<dyn Duplex>),
    /// A peer failed authentication and was closed before any byte.
    Refused,
    /// Nobody connected during the wait.
    Idle,
}
/// Serving side of a published endpoint: the platform listener, or a test double.
pub(crate) trait Accept: Send {
    /// Wait at most `wait` for one connection.
    ///
    /// # Errors
    /// The listener, or the name it published, failed; the server backs off before retrying.
    fn accept(&mut self, wait: Duration) -> io::Result<Incoming>;
    /// False once the published name no longer belongs to this listener (removed or replaced).
    fn published(&self) -> bool;
    /// Stop listening and remove the name while it is still this listener's.
    ///
    /// # Errors
    /// The name was already gone or replaced; nothing else is removed.
    fn close(self: Box<Self>) -> io::Result<()>;
}
/// Publishes the endpoint's listener, again when a published name was lost.
pub(crate) trait Publish: Send + Sync {
    /// Publish a new listener under the endpoint lease.
    ///
    /// # Errors
    /// The runtime directory, the lease or the name cannot be taken.
    fn publish(&self) -> io::Result<Box<dyn Accept>>;
}
/// A request serialized within the frame bound, with the id its reply must carry.
pub(crate) struct EncodedRequest {
    request_id: String,
    frame: Vec<u8>,
}
/// Instance answer: the envelope, with the request it answers.
#[derive(Serialize, Deserialize)]
struct WireReply {
    request_id: String,
    warning: bool,
    envelope: Value,
}

/// SHA-256 of the user and canonical store path: endpoint name and request binding.
///
/// # Errors
/// `unavailable` when the current user identity cannot be read.
pub fn store_identity(store: &Path) -> Result<String, SettingsError> {
    let user = user_identity().map_err(|_| unavailable())?;
    let mut hash = Sha256::new();
    hash.update(b"handy-settings/1\0");
    hash.update(user.as_bytes());
    hash.update(b"\0");
    hash.update(store.as_os_str().as_encoded_bytes());
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
#[cfg(unix)]
fn user_identity() -> io::Result<String> {
    Ok(super::platform_unix::current_uid().to_string())
}
#[cfg(windows)]
fn user_identity() -> io::Result<String> {
    super::platform_windows::current_logon_sid()
}
fn unavailable() -> SettingsError {
    SettingsError::new(SettingsErrorCode::Unavailable)
}

/// The endpoint of one store for the current user.
///
/// # Errors
/// `unavailable` when the user identity or the private runtime directory cannot be resolved.
pub(crate) fn endpoint(identity: &str) -> Result<Endpoint, SettingsError> {
    Endpoint::for_identity(identity).map_err(|_| unavailable())
}

/// Socket pathname of the endpoint of `store`, for lifecycle checks.
///
/// # Errors
/// `unavailable` when the user identity or the runtime directory cannot be resolved.
#[cfg(unix)]
pub fn endpoint_path(store: &Path) -> Result<std::path::PathBuf, SettingsError> {
    Ok(endpoint(&store_identity(store)?)?.path())
}

/// Serialize `request` before any connection, so an oversized one is refused while nothing has
/// been sent (never an unknown outcome).
///
/// # Errors
/// `limit_exceeded` when the request, input escaped and wrapped, exceeds `MAX_FRAME_BYTES`;
/// `internal_error` if it cannot be serialized.
pub(crate) fn encode(request: &OperationRequest) -> Result<EncodedRequest, SettingsError> {
    let frame = serde_json::to_vec(request)
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    if frame.len() > MAX_FRAME_BYTES {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    Ok(EncodedRequest {
        request_id: request.request_id.clone(),
        frame,
    })
}
/// Send one encoded request and wait until `deadline`. Once bytes may have left, every failure
/// (timeout, broken or malformed reply) is `unavailable` with outcome `unknown`: no fallback, no
/// replay.
pub(crate) fn exchange(
    mut stream: Box<dyn Duplex>,
    request: &EncodedRequest,
    deadline: Instant,
) -> Reply {
    let (sender, receiver) = mpsc::channel();
    let frame = request.frame.clone();
    // The worker owns the stream; after the deadline it is abandoned with this process.
    std::thread::spawn(move || {
        let answer = write_frame(&mut stream, &frame).and_then(|()| read_frame(&mut stream));
        let _ = sender.send(answer);
    });
    let remaining = deadline.saturating_duration_since(Instant::now());
    match receiver.recv_timeout(remaining) {
        Ok(Ok(answer)) => decode(&answer, &request.request_id),
        _ => None,
    }
    .unwrap_or_else(|| Reply::failure(SettingsError::unknown_outcome()))
}
fn decode(answer: &[u8], request_id: &str) -> Option<Reply> {
    let reply = serde_json::from_slice::<WireReply>(answer).ok()?;
    (reply.request_id == request_id).then_some(())?;
    Reply::from_wire(reply.envelope, reply.warning)
}
/// A versioned request for this store, without raw argv.
pub(crate) fn request(identity: &str, operation: SettingsOperation) -> OperationRequest {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    OperationRequest {
        protocol_version: ProtocolVersion,
        request_id: format!("{:x}-{nanos:x}", std::process::id()),
        store_identity: identity.to_owned(),
        operation,
    }
}

pub(crate) fn write_frame(stream: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let length = u32::try_from(bytes.len())
        .ok()
        .filter(|length| *length as usize <= MAX_FRAME_BYTES)
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(bytes)?;
    stream.flush()
}
/// Refuse an oversized frame before allocating it.
pub(crate) fn read_frame(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut header = [0; 4];
    stream.read_exact(&mut header)?;
    let length = u32::from_be_bytes(header) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut frame = vec![0; length];
    stream.read_exact(&mut frame)?;
    Ok(frame)
}

/// Read one request, answer it, close. Malformed input gets a fixed error, never an echo.
pub(crate) fn answer_connection(
    mut stream: Box<dyn Duplex>,
    identity: &str,
    backend: &dyn SettingsBackend,
) {
    if stream.limit(DEADLINE).is_err() {
        return;
    }
    let Ok(frame) = read_frame(&mut stream) else {
        return;
    };
    let reply = answer(&frame, identity, backend);
    if let Ok(bytes) = serde_json::to_vec(&reply) {
        let _ = write_frame(&mut stream, &bytes);
    }
}
fn answer(frame: &[u8], identity: &str, backend: &dyn SettingsBackend) -> WireReply {
    let request = serde_json::from_slice::<Value>(frame)
        .map_err(|_| SettingsError::new(SettingsErrorCode::UsageError))
        .and_then(OperationRequest::from_value);
    let (request_id, reply) = match request {
        Ok(request) if request.store_identity == identity => {
            let reply = execute(&request.operation, backend);
            (request.request_id, reply)
        }
        Ok(request) => (request.request_id, Reply::failure(unavailable())),
        Err(error) => (String::new(), Reply::failure(error)),
    };
    WireReply {
        request_id,
        warning: reply.warning(),
        envelope: reply.envelope().clone(),
    }
}
#[cfg(test)]
#[path = "ipc_tests.rs"]
mod tests;
