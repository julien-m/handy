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
    route_until(operation, Instant::now() + ipc::DEADLINE)
}
/// Same as `route`, within `deadline`: the several requests of one CLI operation (a paged read,
/// a staged import) share one total deadline instead of one each.
pub fn route_until(operation: &SettingsOperation, deadline: Instant) -> Reply {
    match StoreLocator::current().and_then(|store| route_store(&store, operation, deadline)) {
        Ok(reply) => reply,
        Err(error) => Reply::failure(error),
    }
}
/// Strategy: an authenticated instance endpoint always wins; without one, a held startup lease
/// means an instance is starting or busy, so the endpoint is retried until the deadline and the
/// store is never written behind it; only a free lease allows the offline path. The request is
/// encoded first, so one too large for a frame is refused (`limit_exceeded`) whatever the route,
/// before anything is sent or written.
fn route_store(
    store: &Path,
    operation: &SettingsOperation,
    deadline: Instant,
) -> Result<Reply, SettingsError> {
    let identity = ipc::store_identity(store)?;
    let request = ipc::encode(&ipc::request(&identity, operation.clone()))?;
    let endpoint = ipc::endpoint(&identity)?;
    loop {
        match endpoint.connect() {
            Connection::Connected(stream) => {
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
    fn commit_change(
        &self,
        change: &super::domains::DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        self.repository
            .commit_edit_with_probe(&change.edit(), expected, &self.probe)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_oversized_request_is_refused_before_any_route() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let store = directory.path().join("settings_store.json");
        let operation = SettingsOperation::Domain {
            domain: "output".into(),
            action: "set".into(),
            arguments: vec!["paste_method".into(), "direct".into()],
            input: Some("\u{1}".repeat(ipc::MAX_FRAME_BYTES / 3)),
            dry_run: false,
            expected_revision: None,
        };
        let refused = route_store(&store, &operation, Instant::now() + ipc::DEADLINE)
            .err()
            .ok_or("oversized request routed")?;
        assert_eq!(
            (refused.code(), refused.outcome()),
            (SettingsErrorCode::LimitExceeded, None)
        );
        Ok(())
    }
}
