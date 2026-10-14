//! Live domain operations: an answer or an action that only the running instance can give
//! (native state, a manager), planned by a domain and run inside the instance reached through the
//! private channel, never in the CLI process. Without a running instance the framework refuses it
//! with the live-only refusal (`unavailable`, exit 5, empty details). An operation reads native
//! state through `managed_state` and writes settings only through `LiveContext::commit_change`.
use super::DomainChange;
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode},
    protocol::Revision,
    store::SettingsRead,
};
use serde_json::Value;

/// Flags of the call a live operation runs for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveCall {
    /// Preview only: an operation that acts must not act (it returns a preview or `usage_error`).
    pub dry_run: bool,
    /// Compare-and-set revision for an operation that writes settings.
    pub expected: Option<Revision>,
}

/// What a live operation answered.
pub enum LiveAnswer {
    /// Data read at the revision read before planning; nothing was written.
    Read(Value),
    /// Settings were committed through `LiveContext::commit_change`; `read` is that commit
    /// (boxed: a settings read is large and most answers are plain reads).
    Committed {
        read: Box<SettingsRead>,
        data: Value,
    },
}

/// The running instance, as a live operation sees it.
pub trait LiveContext {
    /// Tauri handle of the instance (`None` in a context without one, such as a test).
    fn app(&self) -> Option<&tauri::AppHandle>;
    /// Published settings and revision.
    ///
    /// # Errors
    /// `unavailable` before the settings service is installed.
    fn read(&self) -> Result<SettingsRead, SettingsError>;
    /// Commit `change` in the instance's write order, with its live effects and UI event.
    ///
    /// # Errors
    /// Every refusal of the shared commit path.
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError>;
}

/// An operation planned by a domain and run by the framework inside the instance.
// @spec FR-006: Shared live operations for the settings CLI — .specs/features/006-destination-application/spec.md#fr-006
pub trait LiveOperation: Send + Sync {
    /// Run against `context` for `call`.
    ///
    /// # Errors
    /// `unavailable` when native state is missing; the operation's own refusals.
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError>;
}

/// Manager `T` of the instance, or `unavailable` when there is no instance or no such manager.
///
/// # Errors
/// `unavailable`; nothing is read or written.
pub fn managed_state<T: Send + Sync + 'static>(
    context: &dyn LiveContext,
) -> Result<tauri::State<'_, T>, SettingsError> {
    use tauri::Manager;
    context
        .app()
        .and_then(|app| app.try_state::<T>())
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::Unavailable))
}
