//! Tauri adapter of the live settings service: installed at startup, it serves the published
//! snapshot to every reader and commits every UI and manager write, then applies live effects and
//! notifies the UI, and publishes the private endpoint that answers routed CLI operations.
use super::{
    command_error::SettingsCommandError,
    domains::{live_operation::LiveContext, DomainChange},
    edit::SettingsEditFn,
    errors::{SettingsError, SettingsErrorCode},
    ipc::EndpointHandle,
    live::{ClaimedSingleInstance, LiveCommit, LiveSettings},
    locator::StoreLocator,
    native_changes,
    operations::SettingsBackend,
    ownership::NativeOwnerProbe,
    protocol::{Revision, SettingsSnapshot},
    sequence::{self, LiveEffectsPort, PreparedChange},
    store::{SettingsRead, SettingsRepository},
    ui_value::UiValue,
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
/// A store this process cannot own (a link, a hard link, unreadable file or lock permissions)
/// leaves the app running read-only on its best read, with a warning, instead of failing startup.
///
/// Side effects: may create the private data directory, the lock files and a normalized store.
///
/// # Errors
/// `unavailable` only when another instance holds this store's lease (a real conflict).
// @spec FR-004: Instance startup lease and exclusive store ownership — .specs/features/008-settings-cli/spec.md#fr-004
pub fn install_owned(app: &AppHandle) -> Result<(), SettingsError> {
    let live = match StoreLocator::current() {
        Ok(path) => LiveSettings::start_owned_or_read_only(
            SettingsRepository::new(path),
            &ClaimedSingleInstance,
        )?,
        Err(error) => read_only_service(error),
    };
    install(app, live);
    Ok(())
}
/// Install the settings of a headless run, which is not the instance: fresh read now, offline
/// writes later (refused while an instance owns the store); read-only if the store cannot be read.
pub fn install_offline(app: &AppHandle) {
    let started = StoreLocator::current().and_then(|path| {
        LiveSettings::start_offline(SettingsRepository::new(path), Box::new(NativeOwnerProbe))
    });
    install(app, started.unwrap_or_else(read_only_service));
}
/// Read-only service for a store that cannot be owned. Without any store pathname (unknown
/// executable or data directory) the empty path yields the defaults.
fn read_only_service(error: SettingsError) -> LiveSettings {
    let path = StoreLocator::current_unchecked().unwrap_or_default();
    LiveSettings::start_read_only(path, error.code())
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
/// True when this process runs read-only because it could not own the settings store.
pub fn read_only(app: &AppHandle) -> bool {
    app.try_state::<LiveSettings>()
        .is_some_and(|live| live.read_only())
}
fn live(app: &AppHandle) -> Result<tauri::State<'_, LiveSettings>, SettingsError> {
    app.try_state::<LiveSettings>()
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::Unavailable))
}
/// Live effects and UI event of committed settings, for the ordered write sequence.
struct TauriEffects<'a>(&'a AppHandle);
impl LiveEffectsPort for TauriEffects<'_> {
    fn apply(&self, commit: &LiveCommit, skip: &[&str]) -> Result<(), Vec<&'static str>> {
        super::effects::apply_changed(self.0, commit, skip)
    }
    fn notify(&self, revision: Revision, keys: &[&'static str]) {
        let event = SettingsChanged {
            revision: revision.0,
            keys: keys.to_vec(),
        };
        if let Err(error) = self.0.emit("settings-changed", event) {
            log::warn!("Could not notify the UI of a settings change: {error}");
        }
    }
}
/// UI scalar write in the write order: shared validation, durable commit, live effects, then the
/// UI event (see `sequence`).
///
/// # Errors
/// Validation or commit refusals (nothing changed), or `internal_error` with outcome
/// `committed` when a live effect failed after the durable write.
pub fn set(
    app: &AppHandle,
    key: &str,
    value: serde_json::Value,
) -> Result<LiveCommit, SettingsCommandError> {
    Ok(sequence::write_value(
        live(app)?.inner(),
        &TauriEffects(app),
        key,
        value,
    )?)
}
/// Settings UI command: write the raw argument on the blocking pool, never on the main thread
/// (a durable write and the wait for an ordered write in progress can take time, and live
/// effects may need the main thread).
///
/// # Errors
/// Same as `set`, or `internal_error` if the worker could not run.
pub async fn write<T>(
    app: AppHandle,
    key: &'static str,
    value: UiValue<T>,
) -> Result<(), SettingsCommandError> {
    let value = value.into_json();
    blocking(move || set(&app, key, value).map(|_| ())).await
}
/// Run a settings command body on the blocking pool (see `write`).
///
/// # Errors
/// The body's error, or `internal_error` if the worker could not run.
pub async fn blocking<T: Send + 'static>(
    body: impl FnOnce() -> Result<T, SettingsCommandError> + Send + 'static,
) -> Result<T, SettingsCommandError> {
    tauri::async_runtime::spawn_blocking(body)
        .await
        .map_err(|error| {
            log::error!("Settings command worker failed: {error}");
            SettingsCommandError::from(SettingsError::new(SettingsErrorCode::InternalError))
        })?
}
/// Typed UI or manager edit applied to the latest settings, in the write order.
///
/// # Errors
/// Same as `set`, plus the edit's own refusal.
pub fn edit(app: &AppHandle, edit: SettingsEditFn<'_>) -> Result<LiveCommit, SettingsCommandError> {
    edit_at(app, edit, None)
}
/// Typed edit committed only while the settings are still at `expected` (when given): an editor
/// that worked from a revision never overwrites a newer one.
///
/// # Errors
/// Same as `edit`, plus `revision_conflict` (with the current revision) when they moved on.
pub fn edit_at(
    app: &AppHandle,
    edit: SettingsEditFn<'_>,
    expected: Option<Revision>,
) -> Result<LiveCommit, SettingsCommandError> {
    Ok(sequence::write_edit(
        live(app)?.inner(),
        &TauriEffects(app),
        edit,
        expected,
    )?)
}
/// Domain change from the settings UI (a list edit built with the same validator as the CLI
/// domain), committed in the write order with the UI's base revision: the same edit, compare-and-
/// set, live effects and event as a routed CLI change (`sequence::write_change`).
///
/// # Errors
/// `revision_conflict` for a stale `expected`, the change's own refusal, otherwise same as `set`.
pub fn write_change(
    app: &AppHandle,
    change: &DomainChange,
    expected: Option<Revision>,
) -> Result<LiveCommit, SettingsCommandError> {
    Ok(sequence::write_change(
        live(app)?.inner(),
        &TauriEffects(app),
        change,
        expected,
    )?)
}
/// Scalar write whose fallible native change is prepared first (from the settings published when
/// the ordered write starts) and compensated when the commit is refused, so a refusal leaves both
/// the disk and the live state unchanged. The preparation of `key` comes from `native_changes`,
/// shared with routed CLI writes.
///
/// # Errors
/// `unavailable` naming `key` when preparation fails (nothing committed); otherwise same as `set`.
pub fn set_prepared(
    app: &AppHandle,
    key: &'static str,
    value: serde_json::Value,
) -> Result<LiveCommit, SettingsCommandError> {
    let native = native_changes::prepared_change(app, key)
        .unwrap_or_else(|| Err(SettingsError::new(SettingsErrorCode::InternalError)))?;
    set_prepared_with(app, key, value, native)
}
/// `set_prepared` with the native change built by the caller from the same `native_changes`
/// constructor (for example with a sink for the details only the settings UI shows).
///
/// # Errors
/// Same as `set_prepared`.
pub fn set_prepared_with(
    app: &AppHandle,
    key: &'static str,
    value: serde_json::Value,
    native: PreparedChange<'_>,
) -> Result<LiveCommit, SettingsCommandError> {
    Ok(sequence::write_prepared(
        live(app)?.inner(),
        &TauriEffects(app),
        key,
        value,
        native,
    )?)
}
/// Typed UI edit of `key` whose native change is prepared first and compensated when the commit
/// is refused (the shortcut lists), in the write order.
///
/// # Errors
/// The preparation's refusal (nothing committed), otherwise same as `edit`.
pub fn edit_prepared(
    app: &AppHandle,
    key: &'static str,
    apply: SettingsEditFn<'_>,
    native: PreparedChange<'_>,
) -> Result<LiveCommit, SettingsCommandError> {
    Ok(sequence::write_edit_prepared(
        live(app)?.inner(),
        &TauriEffects(app),
        key,
        apply,
        native,
    )?)
}
/// Run `body` as one ordered write (for commands that read, prepare and commit together).
///
/// # Errors
/// `unavailable` before the service is installed; otherwise the body's own result.
pub fn ordered<T>(
    app: &AppHandle,
    body: impl FnOnce() -> Result<T, SettingsCommandError>,
) -> Result<T, SettingsCommandError> {
    let live = live(app)?;
    sequence::ordered(&live, &TauriEffects(app), body)
}
/// Internal writer (startup fallback, manager repair): committed outside the write order with no
/// live effect (the caller's live state already reflects it); the UI is notified. A refusal is
/// logged, never raised. Returns true when the edit is committed (or was already in place).
pub fn repair(app: &AppHandle, what: &str, apply: SettingsEditFn<'_>) -> bool {
    let repaired = live(app).and_then(|live| sequence::repair(&live, &TauriEffects(app), apply));
    if let Err(error) = &repaired {
        log::warn!("Could not persist {what}: {}", error.message());
    }
    repaired.is_ok()
}

/// CLI operations routed to this instance: the published snapshot, then the same ordered write,
/// live effects and UI event as a settings-UI scalar write.
struct RoutedBackend {
    app: AppHandle,
}
impl SettingsBackend for RoutedBackend {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        live(&self.app)?.read()
    }
    /// A key the settings UI prepares (`native_changes`) is prepared the same way here, so the
    /// CLI gets the UI's decision (for example `unavailable`, nothing written, during a recording).
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let live = live(&self.app)?;
        let effects = TauriEffects(&self.app);
        let commit = match native_changes::prepared_change(&self.app, mutation.key()) {
            Some(native) => {
                sequence::write_scalar_prepared(&live, &effects, mutation, expected, native?)?
            }
            None => sequence::write_scalar(&live, &effects, mutation, expected)?,
        };
        Ok(routed_read(&live, commit))
    }
    // @spec AC-006: Deleting a prompt releases its shortcuts first — .specs/features/005-custom-ai-prompts/spec.md#ac-006
    /// A domain change that also changes the shortcut lists (deleting a prompt with shortcuts,
    /// 005) gets the settings UI's `bindings` preparation: released before the commit,
    /// registered again when the commit is refused, refused during a dictation session. Any
    /// other change skips the preparation (a no-op for `bindings`) and commits as before.
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let live = live(&self.app)?;
        let effects = TauriEffects(&self.app);
        let commit = match native_changes::prepared_change(&self.app, "bindings") {
            Some(native) => sequence::write_change_prepared(
                &live,
                &effects,
                ("bindings", change),
                expected,
                native?,
            )?,
            None => sequence::write_change(&live, &effects, change, expected)?,
        };
        Ok(routed_read(&live, commit))
    }
    /// A live-only domain change of `key` gets the native preparation of the settings UI.
    fn commit_live_change(
        &self,
        key: &'static str,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let live = live(&self.app)?;
        let effects = TauriEffects(&self.app);
        let commit = match native_changes::prepared_change(&self.app, key) {
            Some(native) => {
                sequence::write_change_prepared(&live, &effects, (key, change), expected, native?)?
            }
            None => sequence::write_change(&live, &effects, change, expected)?,
        };
        Ok(routed_read(&live, commit))
    }
    fn is_live(&self) -> bool {
        true
    }
    fn live_context(&self) -> Option<&dyn LiveContext> {
        Some(self)
    }
}
/// Live domain operations of routed CLI calls run here, inside the instance: the published
/// snapshot, and the same ordered write, live effects and `settings-changed` event as the UI.
impl LiveContext for RoutedBackend {
    fn app(&self) -> Option<&AppHandle> {
        Some(&self.app)
    }
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        live(&self.app)?.read()
    }
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self, change, expected)
    }
}
fn routed_read(live: &LiveSettings, commit: LiveCommit) -> SettingsRead {
    SettingsRead {
        snapshot: SettingsSnapshot {
            settings: commit.after,
            revision: commit.revision,
        },
        warning: live.recovered(),
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
