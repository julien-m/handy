//! Session pin of 009 (PIN mode): one pinned application identity kept in the destination
//! manager's RAM. It becomes the destination of the current dictation (before insertion) and of
//! every following one, whatever is frontmost at their start, until it is unpinned or Handy
//! quits; it is never persisted or restored. 006's capture at start is still made and kept, so an
//! Unpin before insertion restores the route 006 gave the dictation. Badge choices and the 006
//! cross change the current dictation only and keep the pin.
use super::context::{ApplicationIdentity, DestinationSource, DictationId};
use super::manager::{DestinationManager, ManagerState};
use super::session::{Destination, Selection, SelectionRefusal};
use super::snapshot::DestinationSnapshot;
use super::workspace::RunningApp;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

/// The pinned application. `pid_hint` is the instance pinned, passed to 006's instance
/// resolution as its captured process: the pinned instance while it runs, otherwise the only
/// running instance of the identity (several others are ambiguous: `activation_failed`).
#[derive(Clone, Debug, PartialEq)]
pub struct PinState {
    pub identity: ApplicationIdentity,
    pub display_name: String,
    pub icon_ref: Option<String>,
    pub pid_hint: u32,
    pub pinned_at: Instant,
    pub pin_version: u64,
}

/// What the badge shows of the pin (name and icon reference only).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct PinView {
    pub display_name: String,
    pub icon_ref: Option<String>,
}

/// The pinned application as `handy settings destination pin|status` reports it: 006's stable
/// identity (`{kind, value}`) and its display name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PinnedApp {
    pub identity: ApplicationIdentity,
    pub display_name: String,
}

/// A pin accepted from the CLI: the pinned application and, when a dictation exists, its new
/// snapshot (for the badge).
#[derive(Clone, Debug, PartialEq)]
pub struct PinChange {
    pub pinned: PinnedApp,
    pub snapshot: Option<DestinationSnapshot>,
}

/// No running application (Handy excluded) has the identity given to `destination pin`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoRunningApplication;
impl std::fmt::Display for NoRunningApplication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("no running application has this identity")
    }
}
impl std::error::Error for NoRunningApplication {}

impl PinState {
    /// The destination of a pinned dictation (source `pinned`).
    pub fn destination(&self) -> Destination {
        Destination {
            identity: self.identity.clone(),
            display_name: self.display_name.clone(),
            icon_ref: self.icon_ref.clone(),
            captured_pid: self.pid_hint,
            source: DestinationSource::Pinned,
        }
    }
    /// What the badge shows.
    pub fn view(&self) -> PinView {
        PinView {
            display_name: self.display_name.clone(),
            icon_ref: self.icon_ref.clone(),
        }
    }
    /// What the CLI reports.
    pub fn pinned_app(&self) -> PinnedApp {
        PinnedApp {
            identity: self.identity.clone(),
            display_name: self.display_name.clone(),
        }
    }
}

// @spec FR-001: Session pin, current and next dictations — .specs/features/009-pin-mode-and-enter-rule/spec.md#fr-001
impl DestinationManager {
    /// The pinned application, if any.
    pub fn pinned(&self) -> Option<PinnedApp> {
        self.lock().pin.as_ref().map(PinState::pinned_app)
    }
    /// Badge "Pin <destination>": pin the current destination of the exact dictation and
    /// version, which becomes its pinned route (a badge choice made before is replaced).
    ///
    /// # Errors
    /// `Stale`, `Frozen`, or `Unavailable` when the dictation has no application destination.
    pub fn pin_current(
        &self,
        dictation_id: DictationId,
        version: u64,
    ) -> Result<DestinationSnapshot, SelectionRefusal> {
        let mut state = self.lock();
        let session = state.session.as_ref().ok_or(SelectionRefusal::Stale)?;
        if session.dictation_id != dictation_id || session.selection_version != version {
            return Err(SelectionRefusal::Stale);
        }
        if !session.accepts_choices() {
            return Err(SelectionRefusal::Frozen);
        }
        let Selection::Application(current) = &session.selection else {
            return Err(SelectionRefusal::Unavailable);
        };
        let app = RunningApp {
            pid: current.captured_pid,
            identity: current.identity.clone(),
            display_name: current.display_name.clone(),
            icon_ref: current.icon_ref.clone(),
        };
        let now = self.clock.now();
        install(&mut state, app, now).ok_or(SelectionRefusal::Stale)
    }
    /// `destination pin <identifier>`: pin the running application whose stable identity value
    /// (bundle identifier; AUMID, otherwise executable path) is `identifier`. Same effect as the
    /// badge: before insertion it changes the current dictation, later only the next ones.
    ///
    /// # Errors
    /// [`NoRunningApplication`] for an identity no running application other than Handy has;
    /// nothing changes.
    pub fn pin_identifier(&self, identifier: &str) -> Result<PinChange, NoRunningApplication> {
        // Native reads outside the session lock.
        let app = running_application(self, identifier).ok_or(NoRunningApplication)?;
        let now = self.clock.now();
        let mut state = self.lock();
        let snapshot = install(&mut state, app, now);
        let pinned = state
            .pin
            .as_ref()
            .map(PinState::pinned_app)
            .ok_or(NoRunningApplication)?;
        Ok(PinChange { pinned, snapshot })
    }
    // @spec AC-002: Unpin restores the route of the start — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-002
    /// Unpin (badge or CLI). Before insertion the current dictation gets back the route 006 gave
    /// it at its start (any badge choice made while pinned is dropped); after the freeze only
    /// the next dictations change. Unpinning with nothing pinned is a no-op. Returns the snapshot
    /// of the current dictation, if any.
    pub fn unpin(&self) -> Option<DestinationSnapshot> {
        unpin_locked(&mut self.lock())
    }
    /// Badge "Unpin": the same unpin, offered only for the exact dictation and version.
    ///
    /// # Errors
    /// `Stale` or `Frozen`; nothing changes.
    pub fn unpin_from_badge(
        &self,
        dictation_id: DictationId,
        version: u64,
    ) -> Result<DestinationSnapshot, SelectionRefusal> {
        let mut state = self.lock();
        let session = state.session.as_ref().ok_or(SelectionRefusal::Stale)?;
        if session.dictation_id != dictation_id || session.selection_version != version {
            return Err(SelectionRefusal::Stale);
        }
        if !session.accepts_choices() {
            return Err(SelectionRefusal::Frozen);
        }
        let unchanged = DestinationSnapshot::from(session);
        Ok(unpin_locked(&mut state).unwrap_or(unchanged))
    }
}

/// See [`DestinationManager::unpin`].
fn unpin_locked(state: &mut ManagerState) -> Option<DestinationSnapshot> {
    state.pin.take()?;
    let session = state.session.as_mut()?;
    session.pin = None;
    if session.accepts_choices() && session.selection != session.start_selection {
        session.selection = session.start_selection.clone();
        session.selection_version += 1;
    }
    Some(DestinationSnapshot::from(&*session))
}

/// Store `app` as the pin and make it the route of a dictation still open to choices. Pinning
/// the same identity again keeps the pin version (a duplicate pin is a no-op for the pin).
fn install(state: &mut ManagerState, app: RunningApp, now: Instant) -> Option<DestinationSnapshot> {
    let same = state
        .pin
        .as_ref()
        .is_some_and(|pin| pin.identity == app.identity);
    if !same {
        state.pin_version += 1;
    }
    let pin = PinState {
        identity: app.identity,
        display_name: app.display_name,
        icon_ref: app.icon_ref,
        pid_hint: app.pid,
        pinned_at: now,
        pin_version: state.pin_version,
    };
    let route = Selection::Application(pin.destination());
    let view = pin.view();
    state.pin = Some(pin);
    let session = state.session.as_mut()?;
    session.pin = Some(view);
    if session.accepts_choices() && session.selection != route {
        session.selection = route;
        session.selection_version += 1;
    }
    Some(DestinationSnapshot::from(&*session))
}

/// The running instance named by `identifier`, Handy's own process excluded. Business rule: the
/// identifier is 006's stable identity value, tried as a bundle identifier, then an AUMID, then
/// an executable path; a display name or a PID never matches and nothing is launched.
fn running_application(manager: &DestinationManager, identifier: &str) -> Option<RunningApp> {
    if identifier.trim().is_empty() {
        return None;
    }
    let own = std::process::id();
    let candidates = [
        ApplicationIdentity::BundleId(identifier.to_owned()),
        ApplicationIdentity::Aumid(identifier.to_owned()),
        ApplicationIdentity::ExecutablePath(PathBuf::from(identifier)),
    ];
    candidates.iter().find_map(|identity| {
        manager
            .workspace
            .instances(identity)
            .into_iter()
            .find(|app| app.pid != own && app.identity == *identity)
    })
}

#[cfg(test)]
#[path = "pin_route_tests.rs"]
mod route_tests;
#[cfg(test)]
#[path = "pin_tests.rs"]
mod tests;
