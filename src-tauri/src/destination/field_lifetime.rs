//! Lifetime of 011's remembered input inside 006's session (plan "Lifetime and destination
//! match"): one field for the current dictation, attached by the start read only while that
//! session is current, open to choices and still names the same application instance, and
//! answered within `FIELD_CAPTURE_BUDGET`; dropped at once by a destination change, a
//! cancellation, a new dictation or the end of the session; moved out at the insertion freeze
//! only for the same instance. Every rule runs under the manager lock.
use super::context::DictationId;
use super::field::{FieldAccess, FieldTarget, RememberedField, FIELD_CAPTURE_BUDGET};
use super::manager::{DestinationManager, FrozenDestination, ManagerState};
use super::session::{DispositionRefusal, Selection};
use std::sync::Arc;

impl DestinationManager {
    /// The platform field port this manager reads through (011).
    pub fn field_access(&self) -> Arc<dyn FieldAccess> {
        Arc::clone(&self.field_access)
    }
    #[must_use]
    /// Install `access` as the field port (startup and tests; unsupported until then).
    pub fn with_field_access(mut self, access: Arc<dyn FieldAccess>) -> Self {
        self.field_access = access;
        self
    }
    /// The instance whose focused element the start read of the current dictation reads, or
    /// `None` without an application destination (spec AC-006).
    pub fn field_target(&self, dictation_id: DictationId, generation: u64) -> Option<FieldTarget> {
        let state = self.lock();
        let session = state
            .session
            .as_ref()
            .filter(|session| session.is(dictation_id, generation))?;
        match &session.selection {
            Selection::Application(destination) => Some(FieldTarget {
                dictation_id,
                destination_generation: generation,
                identity: destination.identity.clone(),
                route_pid: destination.captured_pid,
            }),
            Selection::FrontmostAtEnd => None,
        }
    }
    // @spec AC-003: Kept only for the same running instance — .specs/features/011-dictation-field-return/spec.md#ac-003
    /// Attach the start read's `field`. Refused, and the reference released at once, when the
    /// dictation is no longer current or open to choices, its selection no longer names this
    /// instance, or the answer came after `FIELD_CAPTURE_BUDGET` (spec SC-005).
    pub fn attach_field(&self, field: RememberedField) -> bool {
        let late = self
            .clock
            .now()
            .saturating_duration_since(field.captured_at)
            > FIELD_CAPTURE_BUDGET;
        let mut state = self.lock();
        let accepted = !late
            && state.generation == field.destination_generation
            && state.session.as_ref().is_some_and(|session| {
                session.is(field.dictation_id, field.destination_generation)
                    && session.accepts_choices()
                    && field.serves(&session.selection)
            });
        if accepted {
            state.field = Some(field);
        }
        accepted
    }
    /// Insertion entry with 011: 006's freeze, and the remembered input when it still serves
    /// the frozen selection (otherwise it is released here).
    ///
    /// # Errors
    /// As [`DestinationManager::freeze`]; the field is released.
    pub fn freeze_with_field(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<(FrozenDestination, Option<RememberedField>), DispositionRefusal> {
        let frozen = self.freeze_session(dictation_id, generation);
        let field = self.lock().field.take();
        let frozen = frozen?;
        let field = field
            .filter(|field| field.dictation_id == dictation_id && field.serves(&frozen.selection));
        Ok((frozen, field))
    }
}

/// Release the remembered input unless the current session still targets its instance: called
/// after every badge choice, cross, pin and unpin (spec AC-003; same instance kept).
pub(super) fn reconcile_field(state: &mut ManagerState) {
    let keep = match (&state.field, &state.session) {
        (Some(field), Some(session)) => {
            field.dictation_id == session.dictation_id && field.serves(&session.selection)
        }
        _ => false,
    };
    if !keep && state.field.take().is_some() {
        log::debug!("field_return dropped(destination_changed)");
    }
}
