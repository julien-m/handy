//! Serving-thread robustness: listener failures back off instead of spinning, refused peers do
//! not, a stuck listener never blocks `close`, and a removed runtime directory is published again.
use super::*;
use crate::settings_service::{
    live::LiveSettings, ownership::LegacyPathProbe, store::SettingsRepository,
};
use std::io;
use std::sync::atomic::AtomicUsize;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn backend(
    directory: &tempfile::TempDir,
) -> Result<Arc<dyn SettingsBackend + Send + Sync>, SettingsError> {
    let live = LiveSettings::start_offline(
        SettingsRepository::new(directory.path().join("settings_store.json")),
        Box::new(LegacyPathProbe::new(directory.path().join("absent.sock"))),
    )?;
    Ok(Arc::new(live))
}

/// Scripted listener: counts its waits and answers each with `outcome` (`None`: fails).
struct Scripted {
    calls: Arc<AtomicUsize>,
    outcome: fn() -> Option<Incoming>,
    stuck: bool,
}
impl Accept for Scripted {
    fn accept(&mut self, _wait: Duration) -> io::Result<Incoming> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.stuck {
            // Ignores its bound, like a blocking accept that no wake-up reaches.
            std::thread::sleep(Duration::from_secs(30));
        }
        (self.outcome)().ok_or_else(|| io::Error::from(io::ErrorKind::PermissionDenied))
    }
    fn published(&self) -> bool {
        true
    }
    fn close(self: Box<Self>) -> io::Result<()> {
        Ok(())
    }
}
struct NeverRepublished;
impl Publish for NeverRepublished {
    fn publish(&self) -> io::Result<Box<dyn Accept>> {
        Err(io::ErrorKind::NotFound.into())
    }
}
fn scripted(
    directory: &tempfile::TempDir,
    outcome: fn() -> Option<Incoming>,
    stuck: bool,
) -> Result<(EndpointHandle, Arc<AtomicUsize>), SettingsError> {
    let calls = Arc::new(AtomicUsize::new(0));
    let listener = Scripted {
        calls: Arc::clone(&calls),
        outcome,
        stuck,
    };
    let handle = start(
        Arc::new(NeverRepublished),
        Box::new(listener),
        "identity".into(),
        backend(directory)?,
    );
    Ok((handle, calls))
}
fn timed_close(handle: EndpointHandle) -> Duration {
    let started = Instant::now();
    handle.close();
    started.elapsed()
}

#[test]
fn a_failing_listener_backs_off_instead_of_spinning() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (handle, calls) = scripted(&directory, || None, false)?;
    std::thread::sleep(Duration::from_secs(1));
    // 50 + 100 + 200 + 400 ms of backoff: about five attempts per second, not millions.
    let attempts = calls.load(Ordering::SeqCst);
    assert!(
        (1..=8).contains(&attempts),
        "{attempts} accept attempts in 1 s"
    );
    assert!(timed_close(handle) < Duration::from_millis(500));
    Ok(())
}

#[test]
fn a_refused_peer_is_skipped_without_waiting() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (handle, calls) = scripted(&directory, || Some(Incoming::Refused), false)?;
    std::thread::sleep(Duration::from_millis(200));
    let attempts = calls.load(Ordering::SeqCst);
    assert!(
        attempts > 100,
        "only {attempts} refused peers handled in 200 ms"
    );
    handle.close();
    Ok(())
}

#[test]
fn a_stuck_listener_never_blocks_close_beyond_its_bound() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (handle, calls) = scripted(&directory, || Some(Incoming::Idle), true)?;
    while calls.load(Ordering::SeqCst) == 0 {
        std::thread::sleep(Duration::from_millis(5));
    }
    let elapsed = timed_close(handle);
    assert!(elapsed >= CLOSE_WAIT && elapsed < CLOSE_WAIT + Duration::from_secs(1));
    Ok(())
}

#[cfg(unix)]
mod unix {
    use super::*;
    use crate::settings_service::{
        endpoint_unix::Endpoint,
        ipc::{store_identity, Connection},
    };
    use std::os::unix::fs::PermissionsExt;

    /// The real endpoint, counting the waits of every listener it publishes.
    struct Counted {
        endpoint: Endpoint,
        waits: Arc<AtomicUsize>,
    }
    struct CountedListener {
        inner: Box<dyn Accept>,
        waits: Arc<AtomicUsize>,
    }
    impl Publish for Counted {
        fn publish(&self) -> io::Result<Box<dyn Accept>> {
            Ok(Box::new(CountedListener {
                inner: self.endpoint.publish()?,
                waits: Arc::clone(&self.waits),
            }))
        }
    }
    impl Accept for CountedListener {
        fn accept(&mut self, wait: Duration) -> io::Result<Incoming> {
            self.waits.fetch_add(1, Ordering::SeqCst);
            self.inner.accept(wait)
        }
        fn published(&self) -> bool {
            self.inner.published()
        }
        fn close(self: Box<Self>) -> io::Result<()> {
            self.inner.close()
        }
    }
    struct Served {
        handle: EndpointHandle,
        client: Endpoint,
        runtime: std::path::PathBuf,
        waits: Arc<AtomicUsize>,
    }
    /// Serve the real endpoint in a private runtime directory inside `directory`.
    fn served(directory: &tempfile::TempDir) -> Result<Served, Box<dyn std::error::Error>> {
        let runtime = directory.path().canonicalize()?.join("hs");
        let identity = store_identity(&directory.path().join("settings_store.json"))?;
        let counted = Arc::new(Counted {
            endpoint: Endpoint::in_directory(runtime.clone(), &identity)?,
            waits: Arc::new(AtomicUsize::new(0)),
        });
        let waits = Arc::clone(&counted.waits);
        let listener = counted.publish()?;
        let handle = start(counted, listener, identity.clone(), backend(directory)?);
        let client = Endpoint::in_directory(runtime.clone(), &identity)?;
        Ok(Served {
            handle,
            client,
            runtime,
            waits,
        })
    }
    fn connects_within(client: &Endpoint, bound: Duration) -> bool {
        let deadline = Instant::now() + bound;
        while Instant::now() < deadline {
            if matches!(client.connect(), Connection::Connected(_)) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    #[test]
    fn a_removed_runtime_directory_is_published_again_and_close_stays_bounded() -> TestResult {
        let directory = tempfile::tempdir()?;
        let served = served(&directory)?;
        assert!(connects_within(&served.client, Duration::from_secs(1)));
        std::fs::remove_dir_all(&served.runtime)?;
        assert!(
            connects_within(&served.client, Duration::from_secs(3)),
            "the CLI finds the instance again"
        );
        let path = served.client.path();
        assert!(timed_close(served.handle) < Duration::from_millis(500));
        assert!(!path.exists(), "closing removes the republished name");
        Ok(())
    }

    #[test]
    fn a_runtime_directory_with_changed_permissions_does_not_spin() -> TestResult {
        let directory = tempfile::tempdir()?;
        let served = served(&directory)?;
        std::fs::set_permissions(&served.runtime, std::fs::Permissions::from_mode(0o755))?;
        std::thread::sleep(Duration::from_secs(1));
        // About five bounded waits per second while healthy, then backoff: never a busy loop.
        let waits = served.waits.load(Ordering::SeqCst);
        assert!(waits <= 12, "{waits} accept waits in 1 s");
        assert!(!matches!(served.client.connect(), Connection::Connected(_)));
        assert!(timed_close(served.handle) < Duration::from_millis(500));
        std::fs::set_permissions(&served.runtime, std::fs::Permissions::from_mode(0o700))?;
        Ok(())
    }
}
