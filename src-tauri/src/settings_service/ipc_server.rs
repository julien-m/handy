//! Serving thread of the private endpoint. Waits on the listener are bounded, so the stop flag is
//! seen without a wake-up connection; a refused peer is skipped at once; a listener failure is
//! logged once and retried with a bounded backoff, never in a busy loop; a lost name (removed
//! socket or runtime directory) is published again under the endpoint lease, so the CLI finds the
//! instance again; and `close` never waits for the thread longer than `CLOSE_WAIT`.
use super::{
    answer_connection, endpoint, store_identity, unavailable, Accept, Incoming, Publish,
    SettingsBackend,
};
use crate::settings_service::errors::SettingsError;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

// Longest wait for a connection: the delay before the thread sees the stop flag. 200 ms keeps
// quitting prompt for about five idle wake-ups per second, each a few metadata reads.
pub(crate) const ACCEPT_WAIT: Duration = Duration::from_millis(200);
// Backoff after a listener failure: 50 ms doubling up to 2 s, so a lasting failure costs one
// attempt every 2 s while a transient one (descriptor exhaustion) recovers quickly.
const RETRY_FIRST: Duration = Duration::from_millis(50);
const RETRY_MAX: Duration = Duration::from_secs(2);
// Exit never waits longer than this for the serving thread (one wait plus closing the name);
// after it, the thread is detached and ends with the process.
pub(crate) const CLOSE_WAIT: Duration = Duration::from_secs(2);

/// Published endpoint of a running instance; `close` removes it.
pub struct EndpointHandle {
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
    finished: mpsc::Receiver<()>,
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
    let listener = endpoint.publish().map_err(|_| unavailable())?;
    Ok(start(Arc::new(endpoint), listener, identity, backend))
}
/// Serve `listener` on its own thread; `publisher` republishes a lost name.
pub(crate) fn start(
    publisher: Arc<dyn Publish>,
    listener: Box<dyn Accept>,
    identity: String,
    backend: Arc<dyn SettingsBackend + Send + Sync>,
) -> EndpointHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let (done, finished) = mpsc::channel::<()>();
    let server = Server {
        publisher,
        listener: Some(listener),
        retry: None,
        identity,
        backend,
    };
    let flag = Arc::clone(&stop);
    let worker = std::thread::spawn(move || {
        server.run(&flag);
        drop(done);
    });
    EndpointHandle {
        stop,
        worker: Some(worker),
        finished,
    }
}
impl EndpointHandle {
    /// Stop accepting, then remove the endpoint name while its lease is still held. Returns
    /// within `CLOSE_WAIT` even if the serving thread is stuck; that thread is then detached.
    pub fn close(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let Some(worker) = self.worker.take() else {
            return;
        };
        worker.thread().unpark();
        match self.finished.recv_timeout(CLOSE_WAIT) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                log::warn!("Settings CLI endpoint thread did not stop in time; detached");
            }
            // Finished (its sender is dropped last): the join returns at once.
            _ => {
                let _ = worker.join();
            }
        }
    }
}

/// State of the serving thread.
struct Server {
    publisher: Arc<dyn Publish>,
    listener: Option<Box<dyn Accept>>,
    /// Current backoff while failing; `None` while healthy.
    retry: Option<Duration>,
    identity: String,
    backend: Arc<dyn SettingsBackend + Send + Sync>,
}
impl Server {
    fn run(mut self, stop: &AtomicBool) {
        while !stop.load(Ordering::SeqCst) {
            match self.listener.take() {
                Some(listener) => self.serve_once(listener, stop),
                None => self.republish(stop),
            }
        }
        if let Some(listener) = self.listener.take() {
            if let Err(error) = listener.close() {
                log::warn!("Settings CLI endpoint could not be removed: {error}");
            }
        }
    }
    /// One bounded wait; the listener is kept unless its published name was lost.
    fn serve_once(&mut self, mut listener: Box<dyn Accept>, stop: &AtomicBool) {
        match listener.accept(ACCEPT_WAIT) {
            Ok(Incoming::Peer(stream)) => {
                self.recovered();
                let (identity, backend) = (self.identity.clone(), Arc::clone(&self.backend));
                std::thread::spawn(move || answer_connection(stream, &identity, backend.as_ref()));
            }
            // A refused peer was closed before any byte; the next connection is served at once.
            Ok(Incoming::Refused | Incoming::Idle) => self.recovered(),
            Err(error) => self.failed(stop, "accept", &error),
        }
        if listener.published() {
            self.listener = Some(listener);
            return;
        }
        log::warn!("Settings CLI endpoint name was removed or replaced; publishing it again");
        // The name is gone: dropping the listener releases its lease for the republication.
        let _ = listener.close();
    }
    /// Publish a new listener after its name was lost, with the same backoff on failure.
    fn republish(&mut self, stop: &AtomicBool) {
        match self.publisher.publish() {
            Ok(listener) => {
                log::info!("Settings CLI endpoint published again");
                self.retry = None;
                self.listener = Some(listener);
            }
            Err(error) => self.failed(stop, "publication", &error),
        }
    }
    /// Log the first failure of a streak, then wait the next backoff delay (or until stopped).
    fn failed(&mut self, stop: &AtomicBool, what: &str, error: &std::io::Error) {
        let delay = self
            .retry
            .map_or(RETRY_FIRST, |delay| (delay * 2).min(RETRY_MAX));
        if self.retry.is_none() {
            log::warn!("Settings CLI endpoint {what} failed: {error}; retrying with backoff");
        }
        self.retry = Some(delay);
        pause(stop, delay);
    }
    fn recovered(&mut self) {
        if self.retry.take().is_some() {
            log::info!("Settings CLI endpoint serves again");
        }
    }
}
/// Sleep `delay`, returning early once `stop` is set (`close` unparks this thread).
fn pause(stop: &AtomicBool, delay: Duration) {
    let until = Instant::now() + delay;
    while !stop.load(Ordering::SeqCst) {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return;
        }
        std::thread::park_timeout(left);
    }
}
#[cfg(test)]
#[path = "ipc_server_tests.rs"]
mod tests;
