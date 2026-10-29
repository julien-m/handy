//! Tauri glue of the destination feature: the managed services installed at startup and the
//! typed UI events. `destination-changed` carries a session snapshot and
//! `destination-support-changed` the runtime support; neither carries a transcript, a window
//! title or a settings revision.
use super::manager::DestinationManager;
use super::recovery::{OutcomeRegistry, RecoverySlot};
use super::session::DestinationSupport;
use super::snapshot::{DestinationSnapshot, SelectionView};
use super::workspace::SystemClock;
use crate::settings_service::platform_capability::PlatformCapabilities;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// UI event of a session change (start, badge choice, freeze, cancel).
pub const DESTINATION_CHANGED: &str = "destination-changed";
/// UI event of a runtime support change (private, no settings revision).
pub const DESTINATION_SUPPORT_CHANGED: &str = "destination-support-changed";

/// Install the destination manager, the recovery slot and the outcome registry, then read the
/// foreground once on a worker thread to publish the runtime support.
///
/// Side effects: manages three Tauri states and spawns one short-lived thread.
pub fn install(app: &AppHandle) {
    let manager = DestinationManager::new(
        super::native_workspace(),
        Arc::new(SystemClock),
        PlatformCapabilities::current(),
    );
    let manager = Arc::new(manager.with_field_access(super::native_field_access()));
    app.manage(Arc::clone(&manager));
    app.manage(Arc::new(RecoverySlot::default()));
    app.manage(Arc::new(OutcomeRegistry::default()));
    let app = app.clone();
    std::thread::spawn(move || emit_support(&app, manager.refresh_support()));
}
/// Destination read at a starting recording, committed once the recording really starts.
pub struct PendingCapture {
    manager: Arc<DestinationManager>,
    capture: super::manager::TentativeCapture,
}
// @spec FR-001: Capture at the actual recording start — .specs/features/006-destination-application/spec.md#fr-001
/// Read the destination before any Handy surface is shown (none before installation, and none
/// while the rollout gate is closed: the dictation then inserts as before 006).
pub fn capture_at_start(app: &AppHandle, return_enabled: bool) -> Option<PendingCapture> {
    if !super::rollout::Rollout::CURRENT.opens_sessions() {
        return None;
    }
    let manager = manager(app)?;
    let capture = manager.capture(return_enabled);
    log::debug!("Destination capture took {:?}", capture.elapsed);
    Some(PendingCapture { manager, capture })
}
impl PendingCapture {
    /// The recording started: commit the session, start 011's read of the destination's focused
    /// input on its own worker (never waited for), and tell the UI (support too, if it changed).
    /// Returns the committed snapshot (014 starts the dictation's key observer with it).
    pub fn commit(self, app: &AppHandle) -> DestinationSnapshot {
        let previous = self.manager.support();
        let snapshot = self.manager.commit_start(self.capture);
        let (id, generation) = (snapshot.dictation_id, snapshot.destination_generation);
        let target = self.manager.field_target(id, generation);
        // Detached on purpose: the read is bounded and the recording never waits for it.
        drop(super::field_capture::spawn_start_read(
            &self.manager,
            target,
        ));
        if snapshot.support != previous {
            emit_support(app, snapshot.support);
        }
        emit_changed(app, &snapshot);
        snapshot
    }
}
/// The pre-insertion check of 010, managed only when the recovery bubble is installed.
struct InstalledRecovery(Arc<dyn super::context::RecoveryPort>);

/// Install 010's recovery bubble: its pre-insertion check for the guard and its outcome
/// consumer in the registry (rollout gate open only; see `recovery::app`).
pub fn install_recovery(
    app: &AppHandle,
    check: Arc<dyn super::context::RecoveryPort>,
    consumer: Arc<dyn super::recovery::OutcomeConsumer>,
) {
    app.manage(InstalledRecovery(check));
    if let Some(registry) = app.try_state::<Arc<OutcomeRegistry>>() {
        registry.register(consumer);
    }
}
/// The pre-insertion check: 010's when installed, otherwise the one that always inserts.
pub fn recovery_port(app: &AppHandle) -> Arc<dyn super::context::RecoveryPort> {
    match app.try_state::<InstalledRecovery>() {
        Some(installed) => Arc::clone(&installed.0),
        None => Arc::new(super::context::NoRecoveryConsumer),
    }
}
/// The destination manager, once installed.
pub fn manager(app: &AppHandle) -> Option<Arc<DestinationManager>> {
    app.try_state::<Arc<DestinationManager>>()
        .map(|state| Arc::clone(state.inner()))
}
// @spec AC-009: Indicator stays on its start monitor — .specs/features/006-destination-application/spec.md#ac-009
/// Tell the UI that the session changed. A dictation with a destination (captured, pinned or
/// chosen from the badge) keeps the indicator on its monitor until the next dictation starts.
pub fn emit_changed(app: &AppHandle, snapshot: &DestinationSnapshot) {
    if matches!(snapshot.selection, SelectionView::Application { .. }) {
        crate::overlay::hold_placement_for_destination();
    }
    if let Err(error) = app.emit(DESTINATION_CHANGED, snapshot) {
        log::warn!("Could not notify the UI of a destination change: {error}");
    }
}
/// Tell the UI the runtime support of destination return.
pub fn emit_support(app: &AppHandle, support: DestinationSupport) {
    if let Err(error) = app.emit(DESTINATION_SUPPORT_CHANGED, support) {
        log::warn!("Could not notify the UI of destination support: {error}");
    }
}
