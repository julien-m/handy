//! Private local channel between the CLI and the running instance: one request per connection,
//! length-prefixed frames bounded to 1 MiB, peers authenticated by the platform endpoint before
//! any application byte, and a total 5 s deadline after which the outcome is unknown.
#[cfg(unix)]
use super::endpoint_unix::{Endpoint, Listener};
#[cfg(windows)]
use super::endpoint_windows::{Endpoint, Listener};
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
use std::sync::{atomic::AtomicBool, atomic::Ordering, mpsc, Arc};
use std::time::{Duration, Instant};

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

/// Send one request and wait until `deadline`. Once bytes may have left, every failure (timeout,
/// broken or malformed reply) is `unavailable` with outcome `unknown`: no fallback, no replay.
pub(crate) fn exchange(
    mut stream: Box<dyn Duplex>,
    request: &OperationRequest,
    deadline: Instant,
) -> Reply {
    let Ok(frame) = serde_json::to_vec(request) else {
        return Reply::failure(SettingsError::new(SettingsErrorCode::InternalError));
    };
    let (sender, receiver) = mpsc::channel();
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

/// Published endpoint of a running instance; `close` removes it.
pub struct EndpointHandle {
    stop: Arc<AtomicBool>,
    endpoint: Endpoint,
    worker: Option<std::thread::JoinHandle<()>>,
}
/// Publish the endpoint of `store` and answer each authenticated connection with `backend`.
///
/// # Errors
/// `unavailable` when the endpoint cannot be published (for example a live foreign listener).
// @spec FR-004: Authenticated live routing endpoint — .specs/features/008-settings-cli/spec.md#fr-004
pub fn serve(
    store: &Path,
    backend: Arc<dyn SettingsBackend + Send + Sync>,
) -> Result<EndpointHandle, SettingsError> {
    let identity = store_identity(store)?;
    let endpoint = endpoint(&identity)?;
    let listener = endpoint.listen().map_err(|_| unavailable())?;
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let worker = std::thread::spawn(move || accept_loop(listener, &flag, &identity, &backend));
    Ok(EndpointHandle {
        stop,
        endpoint,
        worker: Some(worker),
    })
}
impl EndpointHandle {
    /// Stop accepting, then remove the endpoint name while its lease is still held.
    pub fn close(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wake the blocking accept; the loop sees the flag and closes the listener.
        drop(self.endpoint.connect());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn accept_loop(
    listener: Listener,
    stop: &AtomicBool,
    identity: &str,
    backend: &Arc<dyn SettingsBackend + Send + Sync>,
) {
    while !stop.load(Ordering::SeqCst) {
        // Refused peers and transient accept errors only skip that connection.
        let Ok(stream) = listener.accept() else {
            continue;
        };
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let (identity, backend) = (identity.to_owned(), Arc::clone(backend));
        std::thread::spawn(move || answer_connection(stream, &identity, backend.as_ref()));
    }
    if let Err(error) = listener.close() {
        log::warn!("Settings CLI endpoint could not be removed: {error}");
    }
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
