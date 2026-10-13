//! Route one CLI operation: to the running instance over its private channel, or to the store
//! itself when no instance owns it. A busy, starting, older or uncertain owner never permits an
//! offline write.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    ipc::{self, Connection},
    locator::StoreLocator,
    operations::{execute, Reply, SettingsBackend},
    ownership::{FileLock, LegacyPathProbe, NativeOwnerProbe, OwnerProbe, OwnerState},
    protocol::{Revision, SettingsOperation},
    store::{SettingsRead, SettingsRepository},
    transaction::{INSTANCE_LOCK, TRANSACTION_LOCK},
    validation::SettingsMutation,
};
use std::path::Path;
use std::time::{Duration, Instant};

// A starting instance publishes its endpoint right after taking the lease; 20 ms polling keeps
// that wait short without busy-looping over the 5 s deadline.
const ENDPOINT_RETRY: Duration = Duration::from_millis(20);

/// Debug builds only: absolute pathname that replaces the passive legacy-owner probe, so process
/// tests never depend on a Handy the developer is running. Release builds ignore it.
pub const TEST_LEGACY_SOCKET_ENV: &str = "HANDY_SETTINGS_TEST_LEGACY_SOCKET";

/// Run `operation` where the settings live; every refusal is a failure reply.
// @spec FR-004: Authenticated live and offline routing — .specs/features/008-settings-cli/spec.md#fr-004
pub fn route(operation: &SettingsOperation) -> Reply {
    match StoreLocator::current().and_then(|store| route_store(&store, operation)) {
        Ok(reply) => reply,
        Err(error) => Reply::failure(error),
    }
}
/// Strategy: an authenticated instance endpoint always wins; without one, a held startup lease
/// means an instance is starting or busy, so the endpoint is retried until the deadline and the
/// store is never written behind it; only a free lease allows the offline path.
fn route_store(store: &Path, operation: &SettingsOperation) -> Result<Reply, SettingsError> {
    let identity = ipc::store_identity(store)?;
    let endpoint = ipc::endpoint(&identity)?;
    let deadline = Instant::now() + ipc::DEADLINE;
    loop {
        match endpoint.connect() {
            Connection::Connected(stream) => {
                let request = ipc::request(&identity, operation.clone());
                return Ok(ipc::exchange(stream, &request, deadline));
            }
            Connection::Refused => return Err(unavailable()),
            Connection::NotListening => {}
        }
        if !instance_running(store)? {
            return offline(store, operation);
        }
        // Exit when the deadline passes: the owner never answered, nothing was sent.
        if Instant::now() >= deadline {
            return Err(unavailable());
        }
        std::thread::sleep(ENDPOINT_RETRY);
    }
}
fn unavailable() -> SettingsError {
    SettingsError::new(SettingsErrorCode::Unavailable)
}
/// True while a Handy instance holds this store's startup lease (running or still starting).
///
/// Order matters: an offline writer also holds the lease, but only inside the transaction lock,
/// so the lease is tested under that lock and a held lease then means an instance. Both locks
/// are released at once; a missing lease file is never created here.
pub(super) fn instance_running(store: &Path) -> Result<bool, SettingsError> {
    let lease = store.with_file_name(INSTANCE_LOCK);
    match std::fs::symlink_metadata(&lease) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        _ => {}
    }
    let _transaction = FileLock::acquire(store.with_file_name(TRANSACTION_LOCK), true)?;
    match FileLock::acquire(lease, false) {
        Ok(_released_on_drop) => Ok(false),
        Err(error) if error.code() == SettingsErrorCode::Unavailable => Ok(true),
        Err(error) => Err(error),
    }
}
/// No new instance owns the store. An older Handy without the settings channel may still hold
/// its own copy, so a detected one refuses reads too; writes additionally require the probe to
/// prove absence, which the repository checks again under the lock before replacing the store.
fn offline(store: &Path, operation: &SettingsOperation) -> Result<Reply, SettingsError> {
    let probe = CliProbe::current();
    if probe.inspect() == OwnerState::Occupied {
        return Err(unavailable());
    }
    let backend = OfflineBackend {
        repository: SettingsRepository::new(store.to_path_buf()),
        probe,
    };
    Ok(execute(operation, &backend))
}

/// Passive legacy-owner evidence of the CLI process.
enum CliProbe {
    Native(NativeOwnerProbe),
    Isolated(LegacyPathProbe),
}
impl CliProbe {
    fn current() -> Self {
        #[cfg(debug_assertions)]
        if let Some(path) = std::env::var_os(TEST_LEGACY_SOCKET_ENV) {
            return Self::Isolated(LegacyPathProbe::new(path.into()));
        }
        Self::Native(NativeOwnerProbe)
    }
}
impl OwnerProbe for CliProbe {
    fn inspect(&self) -> OwnerState {
        match self {
            Self::Native(probe) => probe.inspect(),
            Self::Isolated(probe) => probe.inspect(),
        }
    }
}

/// The store itself, written as an offline writer under the shared lock and validation.
struct OfflineBackend {
    repository: SettingsRepository,
    probe: CliProbe,
}
impl SettingsBackend for OfflineBackend {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        self.repository.read_snapshot()
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        self.repository
            .commit_with_probe(mutation, expected, &self.probe, &())
    }
}
