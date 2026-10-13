//! Ordered writes of the running process. Each write's native preparation, durable commit, live
//! effects and UI event complete before the next write starts, for the settings UI and for CLI
//! operations routed to the instance alike, so the live (native) state always ends equal to the
//! last committed value. Live effects that failed after their commit are retried before the next
//! write. Internal repairs stay outside this order (see `repair`).
use super::{
    edit::{SettingsEdit, SettingsEditFn},
    errors::{Outcome, SettingsError},
    live::{LiveCommit, LiveSettings},
    protocol::Revision,
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use std::{
    collections::BTreeMap,
    sync::{Mutex, MutexGuard, PoisonError},
    thread::ThreadId,
};

/// Native side of committed settings, injected so that the order is testable without Tauri.
pub(crate) trait LiveEffectsPort {
    /// Apply the live effect of every changed key of `commit` except `skip`; every key runs even
    /// after a failure.
    ///
    /// # Errors
    /// The keys whose live effect failed, in the order they were applied.
    fn apply(&self, commit: &LiveCommit, skip: &[&str]) -> Result<(), Vec<&'static str>>;
    /// Notify the UI that `keys` changed at `revision` (names only, never values).
    fn notify(&self, revision: Revision, keys: &[&'static str]);
}

/// The turn of the write in progress and the keys whose effect failed after their commit.
#[derive(Default)]
pub(crate) struct WriteSequence {
    turn: Mutex<()>,
    /// Thread holding `turn`: a live effect may write again on that thread (reentrant turn).
    holder: Mutex<Option<ThreadId>>,
    /// Key → settings its failed effect started from, retried before the next write.
    unapplied: Mutex<BTreeMap<&'static str, AppSettings>>,
}
impl WriteSequence {
    /// Run `body` in the write order; immediately when this thread already holds the turn.
    fn run<T>(&self, body: impl FnOnce() -> T) -> T {
        let me = std::thread::current().id();
        if *lock(&self.holder) == Some(me) {
            return body();
        }
        // Order matters: the turn is taken before the holder is recorded, and the holder is
        // cleared (drop order: `_holder` first) before the turn is released.
        let _turn = lock(&self.turn);
        let _holder = Holder::claim(&self.holder, me);
        body()
    }
    #[cfg(test)]
    /// Keys whose live effect is still not applied after its commit.
    pub(crate) fn unapplied_keys(&self) -> Vec<&'static str> {
        lock(&self.unapplied).keys().copied().collect()
    }
}
/// Records the thread holding the turn; cleared on drop, even when the body panics.
struct Holder<'a>(&'a Mutex<Option<ThreadId>>);
impl<'a> Holder<'a> {
    fn claim(slot: &'a Mutex<Option<ThreadId>>, thread: ThreadId) -> Self {
        *lock(slot) = Some(thread);
        Self(slot)
    }
}
impl Drop for Holder<'_> {
    fn drop(&mut self) {
        *lock(self.0) = None;
    }
}
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Run `body` as one ordered write: live effects still unapplied are retried first (live state
/// is reconciled before another mutation is accepted), then `body` reads, prepares and commits
/// without any other ordered write interleaving.
pub(crate) fn ordered<T>(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    body: impl FnOnce() -> T,
) -> T {
    live.sequence().run(|| {
        retry_unapplied(live, port);
        body()
    })
}
/// Settings-UI scalar write: shared validation, commit, live effects, then the UI event.
///
/// # Errors
/// Validation or commit refusals (nothing changed), or `internal_error` with outcome `committed`
/// when a live effect failed after the durable write.
pub(crate) fn write_value(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    key: &str,
    value: serde_json::Value,
) -> Result<LiveCommit, SettingsError> {
    ordered(live, port, || {
        let mutation = live.validate_ui_value(key, value)?;
        commit_and_apply(live, port, &SettingsEdit::Scalar(mutation), None)
    })
}
/// CLI scalar routed to the instance: same order and effects as a settings-UI write.
///
/// # Errors
/// Same as `write_value`, plus `revision_conflict` for a stale `expected` revision.
pub(crate) fn write_scalar(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    mutation: SettingsMutation,
    expected: Option<Revision>,
) -> Result<LiveCommit, SettingsError> {
    ordered(live, port, || {
        commit_and_apply(live, port, &SettingsEdit::Scalar(mutation), expected)
    })
}
/// Domain change routed to the instance (for example `handy settings output set`): same order,
/// effects and event as a settings-UI write.
///
/// # Errors
/// Same as `write_scalar`, plus the change's own refusal.
pub(crate) fn write_change(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    change: &super::domains::DomainChange,
    expected: Option<Revision>,
) -> Result<LiveCommit, SettingsError> {
    ordered(live, port, || {
        commit_and_apply(live, port, &change.edit(), expected)
    })
}
/// Typed UI or manager edit applied to the latest settings, in the write order.
///
/// # Errors
/// Same as `write_value`, plus the edit's own refusal.
pub(crate) fn write_edit(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    edit: SettingsEditFn<'_>,
) -> Result<LiveCommit, SettingsError> {
    ordered(live, port, || {
        commit_and_apply(live, port, &SettingsEdit::Apply(edit), None)
    })
}
/// Scalar write whose fallible native change is prepared from the current settings before the
/// commit (skipped for a no-op), then compensated if the commit is refused.
///
/// # Errors
/// The preparation's refusal (nothing committed), otherwise same as `write_value`.
pub(crate) fn write_prepared(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    key: &'static str,
    value: serde_json::Value,
    native: PreparedChange<'_>,
) -> Result<LiveCommit, SettingsError> {
    ordered(live, port, || {
        let mutation = live.validate_ui_value(key, value)?;
        let before = live.settings();
        let edit = SettingsEdit::Scalar(mutation);
        if !changes_key(&before, &edit, key)? {
            return commit_and_apply(live, port, &edit, None);
        }
        (native.prepare)(&before)?;
        commit_prepared(live, port, key, &edit, &before, || {
            (native.compensate)(&before)
        })
    })
}
/// Native preparation from the settings published when the ordered write started.
pub(crate) type Prepare<'a> = Box<dyn FnOnce(&AppSettings) -> Result<(), SettingsError> + 'a>;
/// Undo of a preparation, from the same settings.
pub(crate) type Compensate<'a> = Box<dyn FnOnce(&AppSettings) + 'a>;
/// A reversible native change: applied before the commit, undone if the commit is refused.
pub(crate) struct PreparedChange<'a> {
    pub prepare: Prepare<'a>,
    pub compensate: Compensate<'a>,
}
/// True when `edit` would change `key` of `settings` (the edit is pure, so it may be evaluated
/// here and again under the transaction lock).
fn changes_key(
    settings: &AppSettings,
    edit: &SettingsEdit<'_>,
    key: &str,
) -> Result<bool, SettingsError> {
    let candidate = super::edit::edited_settings(settings, edit)?;
    Ok(key_differs(settings, &candidate, key))
}
fn key_differs(before: &AppSettings, after: &AppSettings, key: &str) -> bool {
    let value = |settings: &AppSettings| {
        serde_json::to_value(settings)
            .ok()
            .and_then(|object| object.get(key).cloned())
    };
    value(before) != value(after)
}
/// Commit `edit`, then apply its live effects and notify: the tail of every ordered write.
///
/// # Errors
/// The commit refusal, or `internal_error` with outcome `committed` after a failed effect.
pub(crate) fn commit_and_apply(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    edit: &SettingsEdit<'_>,
    expected: Option<Revision>,
) -> Result<LiveCommit, SettingsError> {
    let before = live.settings();
    match live.commit(edit, expected) {
        Ok(commit) => finish(live, port, &commit, &[]).map(|()| commit),
        Err(error) => Err(after_failed_commit(live, port, &before, error, &[])),
    }
}
/// Commit a change whose native side was prepared in this ordered write: the effect of `key` is
/// skipped, a refused commit runs `compensate`, and an unknown outcome keeps the preparation only
/// when the reread store holds the new value of `key`.
///
/// # Errors
/// Same as `commit_and_apply`.
pub(crate) fn commit_prepared(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    key: &'static str,
    edit: &SettingsEdit<'_>,
    before: &AppSettings,
    compensate: impl FnOnce(),
) -> Result<LiveCommit, SettingsError> {
    match live.commit(edit, None) {
        Ok(commit) => finish(live, port, &commit, &[key]).map(|()| commit),
        Err(error) => {
            // Business rule: the store is authoritative. Without an unknown outcome nothing was
            // written; with one, the reread store tells whether the new value landed.
            let landed = error.outcome() == Some(Outcome::Unknown)
                && key_differs(before, &live.settings(), key);
            if !landed {
                compensate();
            }
            let skip: &[&str] = if landed { &[key] } else { &[] };
            Err(after_failed_commit(live, port, before, error, skip))
        }
    }
}
/// After an unknown outcome the published snapshot was reread from disk: apply the effects of
/// whatever it changed and notify, so live state and UI follow the store. The error is kept.
fn after_failed_commit(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    before: &AppSettings,
    error: SettingsError,
    skip: &[&str],
) -> SettingsError {
    if error.outcome() != Some(Outcome::Unknown) {
        return error;
    }
    match live.changes_since(before) {
        Ok(reread) => {
            // The unknown outcome is what the caller reports; an effect failure is recorded.
            let _ = finish(live, port, &reread, skip);
        }
        Err(reread_error) => log::warn!("Settings reread after an unknown outcome: {reread_error}"),
    }
    error
}
/// Order matters: live effects, then the UI event, both before the result is returned. A no-op
/// neither applies effects nor notifies.
///
/// # Errors
/// `internal_error` with outcome `committed` naming the first key whose live effect failed.
fn finish(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    commit: &LiveCommit,
    skip: &[&str],
) -> Result<(), SettingsError> {
    if commit.changed.is_empty() {
        return Ok(());
    }
    let applied = port.apply(commit, skip);
    record_effects(live.sequence(), commit, applied.as_ref().err());
    port.notify(commit.revision, &commit.changed);
    applied.map_err(|failed| {
        let key = failed.first().copied().unwrap_or_default();
        SettingsError::committed_effect_failed(live.snapshot().revision, key)
    })
}
/// Remember failed keys (with the settings their effect started from); forget applied ones.
fn record_effects(sequence: &WriteSequence, commit: &LiveCommit, failed: Option<&Vec<&str>>) {
    let mut unapplied = lock(&sequence.unapplied);
    for key in commit.changed.iter().copied() {
        if failed.is_some_and(|failed| failed.contains(&key)) {
            unapplied
                .entry(key)
                .or_insert_with(|| commit.before.clone());
        } else {
            unapplied.remove(key);
        }
    }
}
/// Reapply the effects that failed after their commit, toward the current published settings.
/// A key that fails again stays recorded; the write that follows is accepted either way.
fn retry_unapplied(live: &LiveSettings, port: &dyn LiveEffectsPort) {
    let pending = std::mem::take(&mut *lock(&live.sequence().unapplied));
    if pending.is_empty() {
        return;
    }
    let current = live.snapshot();
    for (key, before) in pending {
        let retry = LiveCommit {
            before,
            after: current.settings.clone(),
            revision: current.revision,
            changed: vec![key],
        };
        if port.apply(&retry, &[]).is_err() {
            log::warn!("Live effect of setting '{key}' still not applied");
            lock(&live.sequence().unapplied).insert(key, retry.before);
        }
    }
}
/// Internal repair (startup fallback, manager repair): committed outside the write order, because
/// it runs from manager code that may hold locks an ordered write's effect waits for. It never
/// applies live effects (its caller's live state already reflects it); the UI is notified.
///
/// # Errors
/// Every refusal of the commit or of the edit itself.
pub(crate) fn repair(
    live: &LiveSettings,
    port: &dyn LiveEffectsPort,
    edit: SettingsEditFn<'_>,
) -> Result<LiveCommit, SettingsError> {
    let commit = live.commit_edit(edit, None)?;
    if !commit.changed.is_empty() {
        port.notify(commit.revision, &commit.changed);
    }
    Ok(commit)
}
#[cfg(test)]
#[path = "sequence_tests.rs"]
mod tests;
