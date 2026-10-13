//! Passive fail-closed legacy-owner inspection; never invokes historical callbacks.

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
#[cfg(unix)]
use std::path::Path;

/// Conservative inspection result; neither absence nor this spike authorizes writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyState {
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
    // A session bus cannot rule out the same user's Handy on another session bus.
    classify_owner(
        query().map_err(std::io::Error::other),
        ProbeScope::CurrentSession,
    )
}

/// The injected evidence seam preserves uncertainty instead of treating it as absence.
pub(crate) fn classify_owner(owner: std::io::Result<bool>, scope: ProbeScope) -> LegacyState {
    match owner {
        Ok(true) => LegacyState::Occupied,
        Ok(false) if scope == ProbeScope::CurrentSession => LegacyState::Uncertain,
        Ok(false) => LegacyState::Absent,
        Err(_) => LegacyState::Unavailable,
    }
}
