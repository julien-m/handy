//! Scoped owner uncertainty and private Windows pipe identity tests.

use super::{instance_probe, platform_windows};

#[test]
fn uncertainty_does_not_authorize_offline_operation() {
    use instance_probe::{classify_owner, LegacyState, ProbeScope};
    assert_eq!(
        classify_owner(Ok(false), ProbeScope::SameUser),
        LegacyState::Absent
    );
    assert_eq!(
        classify_owner(Ok(true), ProbeScope::CurrentSession),
        LegacyState::Occupied
    );
    assert_eq!(
        classify_owner(
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            ProbeScope::SameUser
        ),
        LegacyState::Unavailable
    );
}

#[test]
fn session_negative_alone_cannot_prove_other_sessions_absent() {
    use instance_probe::{classify_owner, LegacyState, ProbeScope};
    assert_eq!(
        classify_owner(Ok(false), ProbeScope::CurrentSession),
        LegacyState::Uncertain
    );
}

/// Decision 008 on Windows: without the old plugin's mutex and window, the 008 instance lease is
/// the authority, so the offline write is allowed; any found resource is an older Handy and a
/// probe error stays refused.
#[test]
fn windows_missing_mutex_and_window_is_absence_and_probe_errors_stay_refused() {
    use instance_probe::{classify_windows_legacy, LegacyState, MutexObservation};
    let failure = || MutexObservation::Failed(std::io::Error::from_raw_os_error(5));
    let cases = [
        (MutexObservation::NotFound, false, LegacyState::Absent),
        (MutexObservation::NotFound, true, LegacyState::Occupied),
        (MutexObservation::Open, false, LegacyState::Occupied),
        (MutexObservation::Open, true, LegacyState::Occupied),
        (failure(), true, LegacyState::Occupied),
        (failure(), false, LegacyState::Unavailable),
    ];
    for (mutex, window_found, expected) in cases {
        let label = format!("{mutex:?} window={window_found}");
        assert_eq!(
            classify_windows_legacy(mutex, window_found),
            expected,
            "{label}"
        );
    }
}

/// Linux decision 008: the session bus only detects an older Handy; the instance lease is the
/// authority for 008+ instances, so a free bus is absence (the lease is checked separately).
#[test]
fn lease_backed_session_bus_absence_is_absence_and_errors_stay_refused() {
    use instance_probe::{classify_owner, LegacyState, ProbeScope};
    assert_eq!(
        classify_owner(Ok(false), ProbeScope::LeaseBackedSession),
        LegacyState::Absent
    );
    assert_eq!(
        classify_owner(Ok(true), ProbeScope::LeaseBackedSession),
        LegacyState::Occupied
    );
    assert_eq!(
        classify_owner(
            Err(std::io::Error::from(std::io::ErrorKind::NotConnected)),
            ProbeScope::LeaseBackedSession
        ),
        LegacyState::Unavailable
    );
}

#[test]
fn pipe_identity_rejects_injection_and_isolates_user_and_store() -> std::io::Result<()> {
    use platform_windows::pipe_name;
    let first = pipe_name("S-1-5-5-1-2", &"a".repeat(64))?;
    assert_ne!(first, pipe_name("S-1-5-5-1-3", &"a".repeat(64))?);
    assert_ne!(first, pipe_name("S-1-5-5-1-2", &"b".repeat(64))?);
    for sid in ["S-1-", "S-1---", "S-1-5)(A;;GA;;;WD)", "S-1-5/path"] {
        assert!(pipe_name(sid, &"a".repeat(64)).is_err());
    }
    assert!(pipe_name("S-1-5-5-1-2", "../other").is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejected_pipe_authentication_never_returns_resource_and_closes_peer() -> std::io::Result<()> {
    use std::io::Read;
    let (mut peer, stream) = std::os::unix::net::UnixStream::pair()?;
    peer.set_nonblocking(true)?;
    let owned: std::os::fd::OwnedFd = stream.into();
    let file = std::fs::File::from(owned);
    let error = platform_windows::authenticate_owned_pipe(file, |_| {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    })
    .expect_err("authentication refusal must not publish an owned resource");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(peer.read(&mut [0; 1])?, 0);
    Ok(())
}

#[test]
fn successful_pipe_authentication_precedes_resource_transfer() -> std::io::Result<()> {
    let verified = std::cell::Cell::new(false);
    let file = tempfile::tempfile()?;
    let owned = platform_windows::authenticate_owned_pipe(file, |resource| {
        resource.metadata()?;
        verified.set(true);
        Ok(())
    })?;
    assert!(verified.get());
    assert!(owned.metadata()?.is_file());
    Ok(())
}
