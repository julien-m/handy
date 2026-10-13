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
    /// Linux session bus (decision 008, 2026-10-13): Handy 008 and later instances hold the
    /// store's instance lease, which every offline writer must take under the transaction lock,
    /// whatever their session; the bus only detects an older Handy in the current session. Its
    /// absence there is accepted as absence; an older Handy in another session of the same user
    /// is the accepted residual risk.
    // Constructed only by the Linux probe and the platform tests.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
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

/// Inspect the original plugin's local mutex/window without messaging or reserving it.
#[cfg(windows)]
pub(crate) fn inspect_legacy_windows() -> LegacyState {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_FILE_NOT_FOUND},
        System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE},
        UI::WindowsAndMessaging::FindWindowW,
    };
    // Original tauri-plugin-single-instance 2.4 resources; absence covers only the current session.
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let mutex = wide("com.pais.handy-sim");
    let class = wide("com.pais.handy-sic");
    let window = wide("com.pais.handy-siw");
    // SAFETY: names are terminated, permissions only query existing ownership; no resource is created.
    let handle = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, mutex.as_ptr()) };
    if !handle.is_null() {
        // SAFETY: this owned successful OpenMutex handle is released exactly once.
        unsafe { CloseHandle(handle) };
        return LegacyState::Occupied;
    }
    // SAFETY: GetLastError snapshots the preceding failed call; FindWindow only observes existing identity.
    let error = unsafe { GetLastError() };
    let target = unsafe { FindWindowW(class.as_ptr(), window.as_ptr()) };
    if !target.is_null() {
        return LegacyState::Occupied;
    }
    classify_owner(
        if error == ERROR_FILE_NOT_FOUND {
            Ok(false)
        } else {
            Err(std::io::Error::from_raw_os_error(error as i32))
        },
        ProbeScope::CurrentSession,
    )
}
