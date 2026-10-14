//! Passive fail-closed legacy-owner inspection; never invokes historical callbacks.

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
#[cfg(unix)]
use std::path::Path;

/// Conservative inspection result; neither absence nor this spike authorizes writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyState {
    Absent,
    Occupied,
    Uncertain,
    Unavailable,
}

/// Scope actually observed by a passive probe; absence needs same-user coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProbeScope {
    SameUser,
    CurrentSession,
    /// Linux session bus and Windows session objects (decision 008, 2026-10-13): Handy 008 and
    /// later instances hold the store's instance lease, which every offline writer must take
    /// under the transaction lock, whatever their session; the session probe only detects an
    /// older Handy in the current session. Its absence there is accepted as absence; an older
    /// Handy in another session of the same user is the accepted residual risk.
    LeaseBackedSession,
}

/// Inspect owner/type without connecting, invoking a callback or deleting anything.
/// Every same-user socket is occupied, including stale sockets: liveness is unproven.
// @spec FR-004: Non-notifying legacy inspection — .specs/features/008-settings-cli/spec.md#fr-004
#[cfg(unix)]
pub(crate) fn inspect_legacy_path(path: &Path, expected_uid: u32) -> LegacyState {
    if !path.is_absolute() {
        return LegacyState::Unavailable;
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() && metadata.uid() == expected_uid => {
            LegacyState::Occupied
        }
        Ok(_) => LegacyState::Unavailable,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => LegacyState::Absent,
        Err(_) => LegacyState::Unavailable,
    }
}

/// Linux asks only the session bus daemon about ownership, never the app callback.
#[cfg(target_os = "linux")]
pub(crate) fn inspect_legacy_dbus() -> LegacyState {
    let query = || -> Result<bool, zbus::Error> {
        let connection = zbus::blocking::connection::Builder::session()?
            .method_timeout(std::time::Duration::from_secs(5))
            .build()?;
        let proxy = zbus::blocking::fdo::DBusProxy::new(&connection)?;
        let name = zbus::names::BusName::try_from("com.pais.handy.SingleInstance")?;
        Ok(proxy.name_has_owner(name)?)
    };
    // The bus covers only this session; the instance lease covers the others (see the scope).
    classify_owner(
        query().map_err(std::io::Error::other),
        ProbeScope::LeaseBackedSession,
    )
}

/// The injected evidence seam preserves uncertainty instead of treating it as absence, except
/// where another authority covers the unobserved scope (`LeaseBackedSession`).
pub(crate) fn classify_owner(owner: std::io::Result<bool>, scope: ProbeScope) -> LegacyState {
    match owner {
        Ok(true) => LegacyState::Occupied,
        Ok(false) if scope == ProbeScope::CurrentSession => LegacyState::Uncertain,
        Ok(false) => LegacyState::Absent,
        Err(_) => LegacyState::Unavailable,
    }
}

/// What the passive Windows probe saw of the original plugin's named mutex.
#[derive(Debug)]
// Built by the Windows probe; the platform tests build it everywhere.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) enum MutexObservation {
    /// The mutex exists: an older Handy runs in this session.
    Open,
    /// `OpenMutexW` failed with `ERROR_FILE_NOT_FOUND`: no such mutex in this session.
    NotFound,
    /// Any other failure: the observation is not trustworthy.
    Failed(std::io::Error),
}

/// Decision 008 applied to Windows (same rule as the Linux bus): the unprefixed mutex and
/// `FindWindowW` cover only the current session, and the instance lease of Handy 008 and later
/// covers every session (the offline writer must take it under the transaction lock), so a missing
/// mutex and window is absence; any found resource is an older Handy; a probe error stays
/// refused. An older Handy in another session of the same user is the accepted residual risk.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn classify_windows_legacy(mutex: MutexObservation, window_found: bool) -> LegacyState {
    let owner = match mutex {
        MutexObservation::Open => Ok(true),
        _ if window_found => Ok(true),
        MutexObservation::NotFound => Ok(false),
        MutexObservation::Failed(error) => Err(error),
    };
    classify_owner(owner, ProbeScope::LeaseBackedSession)
}

/// Inspect the original plugin's local mutex/window without messaging or reserving it.
// @spec FR-004: Passive Windows legacy detection — .specs/features/008-settings-cli/spec.md#fr-004
#[cfg(windows)]
pub(crate) fn inspect_legacy_windows() -> LegacyState {
    use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;
    // Original tauri-plugin-single-instance 2.4 resources, looked up in the current session.
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let (class, window) = (wide("com.pais.handy-sic"), wide("com.pais.handy-siw"));
    let observation = observe_mutex(&wide("com.pais.handy-sim"));
    // SAFETY: terminated names; FindWindowW only observes an existing class/window identity.
    let window_found = !unsafe { FindWindowW(class.as_ptr(), window.as_ptr()) }.is_null();
    classify_windows_legacy(observation, window_found)
}
/// Open the named mutex `name` (terminated UTF-16) only to learn whether it exists.
#[cfg(windows)]
fn observe_mutex(name: &[u16]) -> MutexObservation {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_FILE_NOT_FOUND},
        System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE},
    };
    // SAFETY: terminated name; SYNCHRONIZE only opens an existing mutex (no wait, no creation),
    // GetLastError snapshots that call before any other OS call, and an opened handle is
    // closed exactly once.
    unsafe {
        let handle = OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr());
        let error = GetLastError();
        if !handle.is_null() {
            CloseHandle(handle);
            return MutexObservation::Open;
        }
        if error == ERROR_FILE_NOT_FOUND {
            return MutexObservation::NotFound;
        }
        MutexObservation::Failed(std::io::Error::from_raw_os_error(error as i32))
    }
}
