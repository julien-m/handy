//! Stable sibling locks and passive legacy ownership are independent of JSON generations.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    store::persistence_failed,
};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[path = "instance_probe.rs"]
pub(super) mod probe;
pub use probe::LegacyState as OwnerState;
// The future IPC technical deadline is five seconds; transaction lock waiting is bounded here too.
const LOCK_DEADLINE: Duration = Duration::from_secs(5);
// fs2 offers no portable timed lock, so a waiter polls. Transactions hold the lock for a few
// milliseconds, so 2 ms keeps added latency well below the 2 s read budget while capping a
// blocked waiter at about 2 500 wake-ups over the 5 s deadline, a negligible CPU cost.
const LOCK_RECHECK: Duration = Duration::from_millis(2);

/// Effective UID is obtained only at the native platform inspection boundary.
#[cfg(unix)]
pub(super) fn current_uid() -> u32 {
    // SAFETY: geteuid has no pointers and does not mutate Rust memory.
    unsafe { libc::geteuid() }
}
/// Passive owner port; injections inspect real isolated OS resources in integration tests.
pub trait OwnerProbe {
    /// Observe current native ownership without connecting, messaging, deleting or reserving resources.
    fn inspect(&self) -> OwnerState;
}
/// Default platform observation never invokes the historic singleton callback.
pub struct NativeOwnerProbe;
impl OwnerProbe for NativeOwnerProbe {
    fn inspect(&self) -> OwnerState {
        #[cfg(target_os = "macos")]
        {
            probe::inspect_legacy_path(Path::new("/tmp/com_pais_handy_si.sock"), current_uid())
        }
        #[cfg(target_os = "linux")]
        {
            probe::inspect_legacy_dbus()
        }
        #[cfg(windows)]
        {
            probe::inspect_legacy_windows()
        }
    }
}
/// Isolated passive pathname probe; absence must cover the same-user scope to permit a write.
pub struct LegacyPathProbe {
    path: PathBuf,
}
impl LegacyPathProbe {
    /// Bind an isolated absolute legacy pathname without filesystem side effects.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
impl OwnerProbe for LegacyPathProbe {
    fn inspect(&self) -> OwnerState {
        #[cfg(unix)]
        {
            let state = probe::inspect_legacy_path(&self.path, current_uid());
            match state {
                OwnerState::Absent | OwnerState::Occupied => probe::classify_owner(
                    Ok(state == OwnerState::Occupied),
                    probe::ProbeScope::SameUser,
                ),
                OwnerState::Uncertain => {
                    probe::classify_owner(Ok(false), probe::ProbeScope::CurrentSession)
                }
                OwnerState::Unavailable => probe::classify_owner(
                    Err(std::io::Error::other("owner unavailable")),
                    probe::ProbeScope::SameUser,
                ),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = &self.path;
            OwnerState::Uncertain
        }
    }
}
/// Pinned stable lock handle; drop releases the lock without unlinking its pathname.
pub(super) struct FileLock {
    #[cfg(windows)]
    parent: super::windows_store::PinnedDirectory,
    file: File,
    path: PathBuf,
}
impl FileLock {
    /// Open a stable private lock and acquire nonblocking or bounded worker ownership; drop retains its pathname.
    pub fn acquire(path: PathBuf, wait: bool) -> Result<Self, SettingsError> {
        let lock = open_lock(path)?;
        lock.validate()?;
        let deadline = Instant::now() + LOCK_DEADLINE;
        loop {
            match fs2::FileExt::try_lock_exclusive(&lock.file) {
                Ok(()) => {
                    lock.validate()?;
                    return Ok(lock);
                }
                Err(error)
                    if error.raw_os_error().is_some_and(|code| {
                        Some(code) == fs2::lock_contended_error().raw_os_error()
                    }) && wait
                        && Instant::now() < deadline =>
                {
                    std::thread::park_timeout(LOCK_RECHECK)
                }
                Err(_) => return Err(SettingsError::new(SettingsErrorCode::Unavailable)),
            }
        }
    }
    /// Pathname this lock was opened at, used to bind a retained lease to one repository.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Compare the held file and current pathname identity; refuse substitutes without deleting them.
    pub fn validate(&self) -> Result<(), SettingsError> {
        let path = std::fs::symlink_metadata(&self.path).map_err(|_| persistence_failed())?;
        let held = self.file.metadata().map_err(|_| persistence_failed())?;
        super::locator::validate_owner(&path)?;
        if !path.is_file() {
            return Err(persistence_failed());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (path.dev(), path.ino(), path.uid()) != (held.dev(), held.ino(), held.uid())
                || path.nlink() != 1
                || path.mode() & 0o777 != 0o600
            {
                return Err(persistence_failed());
            }
        }
        #[cfg(windows)]
        {
            self.parent.validate().map_err(|_| persistence_failed())?;
            let current = super::windows_store::open_private(&self.path, true)
                .map_err(|_| persistence_failed())?;
            if super::windows_store::identity(&current).map_err(|_| persistence_failed())?
                != super::windows_store::identity(&self.file).map_err(|_| persistence_failed())?
            {
                return Err(persistence_failed());
            }
            let _ = held;
        }
        Ok(())
    }
}
fn open_lock(path: PathBuf) -> Result<FileLock, SettingsError> {
    #[cfg(windows)]
    let parent = super::windows_store::pin_parent(path.parent().ok_or_else(persistence_failed)?)
        .map_err(|_| persistence_failed())?;
    #[cfg(windows)]
    let file = match super::windows_store::create_private_lock(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            super::windows_store::open_private(&path, true).map_err(|_| persistence_failed())?
        }
        Err(_) => return Err(persistence_failed()),
    };
    #[cfg(unix)]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;
        // Stable lock contents/inode are deliberately retained; never truncate an existing lock.
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|_| persistence_failed())?
    };
    let lock = FileLock {
        file,
        path,
        #[cfg(windows)]
        parent,
    };
    Ok(lock)
}
/// Retain startup lease until shutdown; ownership cannot be inferred from a busy lease.
///
/// The running instance passes it to `SettingsRepository::commit_owned` to write through the same
/// locked repository while every offline writer stays refused.
pub struct StartupLease {
    pub(super) lock: FileLock,
}
impl StartupLease {
    /// Revalidate the pinned lease; dropping it releases ownership without deleting the lock.
    pub fn validate(&self) -> Result<(), SettingsError> {
        self.lock.validate()
    }
}
/// Fail closed on occupied, unavailable and session-limited evidence.
pub(super) fn require_absent(probe: &impl OwnerProbe) -> Result<(), SettingsError> {
    if probe.inspect() != OwnerState::Absent {
        return Err(SettingsError::new(SettingsErrorCode::Unavailable));
    }
    Ok(())
}
