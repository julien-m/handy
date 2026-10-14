//! `handy settings microphones` (feature 001): `list` reports the configured order, each
//! choice's availability and the effective input, answered by the running Handy because only its
//! audio manager knows them (live operation: without a running Handy it is refused with
//! `unavailable`, exit 5, empty details, and nothing is fabricated); `set-priority` commits the
//! complete order through the shared validation and transaction, offline or routed, with
//! `--dry-run` and `--expected-revision`. Zero names set the empty order (the system default).
use super::live_operation::{managed_state, LiveAnswer, LiveCall, LiveContext, LiveOperation};
use super::{Domain, DomainPlan, DomainRequest};
use crate::managers::audio::AudioRecordingManager;
use crate::microphones::priority::{priority_change, PRIORITY_KEY};
use crate::settings::AppSettings;
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use std::sync::Arc;

/// The microphones domain.
pub struct MicrophonesDomain;
impl Domain for MicrophonesDomain {
    fn name(&self) -> &'static str {
        "microphones"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["list", "set-priority"]
    }
    // @spec FR-008: microphones list and set-priority — .specs/features/001-microphone-priority/spec.md#fr-008
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        _settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        // `_settings` is unused: `list` reads them inside the instance and `set-priority` edits
        // the locked latest settings.
        match (request.action, request.arguments) {
            ("list", []) => Ok(DomainPlan::Live(Box::new(MicrophonesList))),
            ("set-priority", names) => Ok(DomainPlan::Change {
                change: priority_change(names.to_vec())?,
                keys: vec![PRIORITY_KEY],
            }),
            _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
        }
    }
}

/// `list`, run inside the instance.
struct MicrophonesList;
impl LiveOperation for MicrophonesList {
    /// `call` is unused: a read has no dry run and no compare-and-set.
    fn run(
        &self,
        context: &dyn LiveContext,
        _call: &LiveCall,
    ) -> Result<LiveAnswer, SettingsError> {
        let configured = context.read()?.snapshot.settings.microphone_priority;
        let manager = managed_state::<Arc<AudioRecordingManager>>(context)?;
        let status = manager.microphone_status(configured);
        let data = serde_json::to_value(status)
            .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
        Ok(LiveAnswer::Read(data))
    }
}

#[cfg(test)]
#[path = "microphones_tests.rs"]
mod tests;
