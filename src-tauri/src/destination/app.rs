//! Tauri glue of the destination feature: the managed services installed at startup and the
//! typed UI events. `destination-changed` carries a session snapshot and
//! `destination-support-changed` the runtime support; neither carries a transcript, a window
//! title or a settings revision.
use super::manager::DestinationManager;
use super::recovery::{OutcomeRegistry, RecoverySlot};
use super::session::DestinationSupport;
use super::snapshot::DestinationSnapshot;
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
    let manager = Arc::new(DestinationManager::new(
        super::native_workspace(),
        Arc::new(SystemClock),
        PlatformCapabilities::current(),
    ));
    app.manage(Arc::clone(&manager));
    app.manage(Arc::new(RecoverySlot::default()));
    app.manage(Arc::new(OutcomeRegistry::default()));
    let app = app.clone();
    std::thread::spawn(move || emit_support(&app, manager.refresh_support()));
}
/// The destination manager, once installed.
pub fn manager(app: &AppHandle) -> Option<Arc<DestinationManager>> {
    app.try_state::<Arc<DestinationManager>>()
        .map(|state| Arc::clone(state.inner()))
}
/// Tell the UI that the session changed.
pub fn emit_changed(app: &AppHandle, snapshot: &DestinationSnapshot) {
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
