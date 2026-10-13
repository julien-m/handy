//! Tauri adapter of the live settings service: installed at startup, it serves the published
//! snapshot to every reader and commits every UI and manager write, then applies live effects and
//! notifies the UI, and publishes the private endpoint that answers routed CLI operations.
use super::{
    command_error::SettingsCommandError,
    edit::SettingsEditFn,
    errors::{SettingsError, SettingsErrorCode},
    ipc::EndpointHandle,
    live::{ClaimedSingleInstance, LiveCommit, LiveSettings},
    locator::StoreLocator,
    operations::SettingsBackend,
    ownership::NativeOwnerProbe,
    protocol::{Revision, SettingsSnapshot},
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// UI notification: revision and changed key names only, never values (some are secret).
#[derive(Clone, serde::Serialize)]
struct SettingsChanged {
    revision: u64,
    keys: Vec<&'static str>,
}

/// Install the running instance's settings: take the startup lease, normalize once, retain it.
///
/// Side effects: may create the private data directory, the lock files and a normalized store.
///
/// # Errors
/// Any startup refusal of `LiveSettings::start_owned`.
// @spec FR-004: Instance startup lease and exclusive store ownership — .specs/features/008-settings-cli/spec.md#fr-004
pub fn install_owned(app: &AppHandle) -> Result<(), SettingsError> {
    let repository = SettingsRepository::new(StoreLocator::current()?);
    let live = LiveSettings::start_owned(repository, &ClaimedSingleInstance)?;
    install(app, live);
    Ok(())
}
/// Install the settings of a headless run, which is not the instance: fresh read now, offline
/// writes later (refused while an instance owns the store).
///
/// # Errors
/// Any read refusal of `LiveSettings::start_offline`.
pub fn install_offline(app: &AppHandle) -> Result<(), SettingsError> {
    let repository = SettingsRepository::new(StoreLocator::current()?);
    let live = LiveSettings::start_offline(repository, Box::new(NativeOwnerProbe))?;
    install(app, live);
    Ok(())
}
fn install(app: &AppHandle, live: LiveSettings) {
    if live.recovered() {
        log::warn!("Settings store was salvaged or unreadable; recovered values are in use");
    }
    app.manage(live);
}
/// Published settings of this process.
pub fn current(app: &AppHandle) -> AppSettings {
    match app.try_state::<LiveSettings>() {
        Some(live) => live.settings(),
        None => {
            // Only reachable if a reader runs before `setup` installed the service; defaults
            // keep it alive and nothing is written from them.
            log::error!("Settings read before the settings service started; using defaults");
            crate::settings::get_default_settings()
        }
    }
}
/// Published revision and settings, for UI refreshes ordered by revision; before the service is
/// installed, the defaults at revision 0 (same fallback as `current`).
pub fn view(app: &AppHandle) -> (Revision, AppSettings) {
    match app.try_state::<LiveSettings>() {
        Some(live) => {
            let snapshot = live.snapshot();
            (snapshot.revision, snapshot.settings)
        }
        None => (Revision(0), current(app)),
    }
}
fn live(app: &AppHandle) -> Result<tauri::State<'_, LiveSettings>, SettingsError> {
    app.try_state::<LiveSettings>()
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::Unavailable))
}
/// UI scalar write: shared validation, durable commit, live effects, then the UI event.
///
/// # Errors
/// Validation or commit refusals (nothing changed), or `internal_error` with outcome
/// `committed` when a live effect failed after the durable write.
pub fn set(
    app: &AppHandle,
    key: &str,
    value: serde_json::Value,
) -> Result<LiveCommit, SettingsCommandError> {
    let commit = live(app)?.commit_ui_value(key, value)?;
    finish(app, &commit, &[])?;
    Ok(commit)
}
/// `set` for a typed command argument; the value is serialized exactly as it is stored.
///
/// # Errors
/// Same as `set`.
pub fn set_value(
    app: &AppHandle,
    key: &str,
    value: impl serde::Serialize,
) -> Result<(), SettingsCommandError> {
    set(app, key, to_json(value)?).map(|_| ())
}
fn to_json(value: impl serde::Serialize) -> Result<serde_json::Value, SettingsCommandError> {
    serde_json::to_value(value)
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError).into())
}
/// Typed UI or manager edit applied to the latest settings, then live effects and the UI event.
///
/// # Errors
/// Same as `set`, plus the edit's own refusal.
pub fn edit(app: &AppHandle, edit: SettingsEditFn<'_>) -> Result<LiveCommit, SettingsCommandError> {
    let commit = live(app)?.commit_edit(edit, None)?;
    finish(app, &commit, &[])?;
    Ok(commit)
}
/// Scalar write whose fallible native change is prepared first and compensated when the commit
/// is refused, so a refusal leaves both the disk and the live state unchanged.
///
/// # Errors
/// `unavailable` when preparation fails (nothing committed); otherwise same as `set`.
pub fn set_prepared(
    app: &AppHandle,
    key: &'static str,
    value: impl serde::Serialize,
    prepare: impl FnOnce() -> Result<(), String>,
    compensate: impl FnOnce(),
) -> Result<LiveCommit, SettingsCommandError> {
    let live = live(app)?;
    let mutation = live.validate_ui_value(key, to_json(value)?)?;
    if let Err(detail) = prepare() {
        log::warn!("Live change of setting '{key}' was refused: {detail}");
        return Err(SettingsCommandError::unavailable(key));
    }
    match live.commit_scalar(mutation, None) {
        Ok(commit) => finish(app, &commit, &[key]).map(|()| commit),
        Err(error) => {
            compensate();
            Err(error.into())
        }
    }
}
/// Internal writer (startup fallback, manager repair): a refusal is logged, never raised.
/// Returns true when the edit is committed (or was already in place).
pub fn repair(app: &AppHandle, what: &str, apply: SettingsEditFn<'_>) -> bool {
    match edit(app, apply) {
        Ok(_) => true,
        Err(error) => {
            log::warn!("Could not persist {what}: {}", error.message);
            false
        }
    }
}
/// `repair` for a manager that already applied the live state itself (for example the audio
/// manager after a device fallback): the UI is notified, no live effect runs again.
pub fn repair_without_effects(app: &AppHandle, what: &str, apply: SettingsEditFn<'_>) {
    let committed = live(app)
        .and_then(|live| live.commit_edit(apply, None))
        .map_err(SettingsCommandError::from)
        .and_then(|commit| finish(app, &commit, &commit.changed));
    if let Err(error) = committed {
        log::warn!("Could not persist {what}: {}", error.message);
    }
}
/// Order matters: live effects complete before the event, and both before success is returned.
/// A no-op neither applies effects nor notifies.
///
/// # Errors
/// `internal_error` with outcome `committed` for the first key whose live effect failed.
pub fn finish(
    app: &AppHandle,
    commit: &LiveCommit,
    skip: &[&str],
) -> Result<(), SettingsCommandError> {
    apply_and_notify(app, commit, skip).map_err(SettingsCommandError::from)
}
fn apply_and_notify(
    app: &AppHandle,
    commit: &LiveCommit,
    skip: &[&str],
) -> Result<(), SettingsError> {
    if commit.changed.is_empty() {
        return Ok(());
    }
    let applied = super::effects::apply_changed(app, commit, skip);
    let event = SettingsChanged {
        revision: commit.revision.0,
        keys: commit.changed.clone(),
    };
    if let Err(error) = app.emit("settings-changed", event) {
        log::warn!("Could not notify the UI of a settings change: {error}");
    }
    applied.map_err(|key| SettingsError::committed_effect_failed(commit.revision, key))
}

/// CLI operations routed to this instance: the published snapshot, then the same commit, live
/// effects and UI event as a settings-UI scalar write. Like that UI path, the effects run after the
/// commit lock is released (see the Step 4 audit finding on unserialized live effects).
struct RoutedBackend {
    app: AppHandle,
}
impl SettingsBackend for RoutedBackend {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        live(&self.app)?.read()
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let live = live(&self.app)?;
        let commit = live.commit_scalar(mutation, expected)?;
        apply_and_notify(&self.app, &commit, &[])?;
        Ok(SettingsRead {
            snapshot: SettingsSnapshot {
                settings: commit.after,
                revision: commit.revision,
            },
            warning: live.recovered(),
        })
    }
}
/// Published CLI endpoint of this instance, removed at exit.
struct PublishedEndpoint(std::sync::Mutex<Option<EndpointHandle>>);

/// Publish the private CLI endpoint once the settings service owns the store. A failure is
/// logged: CLI calls then report `unavailable` while this instance holds the lease.
pub fn publish_endpoint(app: &AppHandle) {
    let backend = Arc::new(RoutedBackend { app: app.clone() });
    match StoreLocator::current().and_then(|store| super::ipc::serve(&store, backend)) {
        Ok(handle) => {
            app.manage(PublishedEndpoint(std::sync::Mutex::new(Some(handle))));
        }
        Err(error) => log::error!("Settings CLI endpoint could not start: {error}"),
    }
}
/// Remove the CLI endpoint at exit, while the startup lease is still held.
pub fn close_endpoint(app: &AppHandle) {
    let handle = app.try_state::<PublishedEndpoint>().and_then(|published| {
        published
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    });
    if let Some(handle) = handle {
        handle.close();
    }
}
