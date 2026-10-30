//! The two keys pressed during a dictation (014), applied to its destination session while it
//! still accepts choices (recording, processing; never once insertion has begun). « Send to the
//! app in front » re-routes a pinned dictation to the application in front at the end, the badge
//! cross's route, and a second press undoes it; the session pin is never touched. « Insert
//! without Enter » marks the dictation so 009's Enter claim refuses it (`UserSkipped`), and a
//! second press clears the mark; the stored Automatic Enter policy is never touched.
use super::context::DictationId;
use super::manager::{DestinationManager, ManagerState};
use super::session::{DestinationSession, Selection, SelectionRefusal};
use super::snapshot::DestinationSnapshot;

/// The route « Send to the app in front » replaced, and the selection version it set.
#[derive(Clone, Debug, PartialEq)]
pub struct Retarget {
    pub previous: Selection,
    pub version: u64,
}

// @spec AC-007: No-pin notice on the indicator — .specs/features/014-in-dictation-keys/spec.md#ac-007
/// What a press of « Send to the app in front » did to an open dictation.
#[derive(Clone, Debug, PartialEq)]
pub enum RetargetOutcome {
    /// The route changed (re-routed or restored); the badge must be told.
    Changed(DestinationSnapshot),
    /// Nothing is pinned, so there is nothing to step away from: the indicator says so
    /// (decision `no_pin_notice`).
    NoPin,
    /// An app is pinned but the route is already « Frontmost app at the end », set by the badge
    /// cross or a pin change rather than by this key: silent, as before.
    Unchanged,
}

/// Per-dictation state of the two keys, reset at each dictation start.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyToggles {
    /// Set by « Send to the app in front »; it still applies while the selection version is
    /// the one it set (no badge choice, pin or unpin moved the route since).
    pub retarget: Option<Retarget>,
    /// « Insert without Enter » is on for this dictation.
    pub enter_skipped: bool,
}

impl DestinationManager {
    // @spec FR-003: Re-route this dictation, pin intact — .specs/features/014-in-dictation-keys/spec.md#fr-003
    /// « Send to the app in front » for the dictation `dictation_id`/`generation`. With a pin,
    /// a route other than the frontmost app at the end becomes that route (new selection
    /// version), remembering the previous one; a second press while it still applies (same
    /// selection version, still pinned) restores the previous route. Without a pin nothing
    /// changes and the result is `NoPin`; on a frontmost-at-the-end route this key did not set,
    /// nothing changes either (`Unchanged`).
    ///
    /// # Errors
    /// `Stale` for another dictation, `Frozen` once insertion has begun.
    pub fn toggle_active_app(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<RetargetOutcome, SelectionRefusal> {
        let mut state = self.lock();
        let pinned = state.pin.is_some();
        let session = open_session(&mut state, dictation_id, generation)?;
        let version = session.selection_version;
        let Some(next) = next_route(session, pinned) else {
            // Business rule (014 AC-001, AC-007): only a missing pin is reported to the user.
            return Ok(if pinned {
                RetargetOutcome::Unchanged
            } else {
                RetargetOutcome::NoPin
            });
        };
        session.choose(dictation_id, version, next)?;
        let snapshot = DestinationSnapshot::from(&*session);
        // 011 AC-003: leaving the application releases the input remembered at the start.
        super::field_lifetime::reconcile_field(&mut state);
        Ok(RetargetOutcome::Changed(snapshot))
    }

    // @spec FR-004: Skip Automatic Enter for this dictation — .specs/features/014-in-dictation-keys/spec.md#fr-004
    /// « Insert without Enter » for the dictation `dictation_id`/`generation`: switch its mark.
    /// The selection version is unchanged, so an open badge menu stays valid.
    ///
    /// # Errors
    /// `Stale` for another dictation, `Frozen` once insertion has begun.
    pub fn toggle_skip_enter(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<DestinationSnapshot, SelectionRefusal> {
        let mut state = self.lock();
        let session = open_session(&mut state, dictation_id, generation)?;
        session.keys.enter_skipped = !session.keys.enter_skipped;
        Ok(DestinationSnapshot::from(&*session))
    }

    /// True while the dictation `dictation_id`/`generation` accepts the keys (before insertion).
    pub fn accepts_keys(&self, dictation_id: DictationId, generation: u64) -> bool {
        let state = self.lock();
        state.generation == generation
            && state.session.as_ref().is_some_and(|session| {
                session.is(dictation_id, generation) && session.accepts_choices()
            })
    }
}

/// The session of `dictation_id`/`generation` while it accepts choices.
fn open_session(
    state: &mut ManagerState,
    dictation_id: DictationId,
    generation: u64,
) -> Result<&mut DestinationSession, SelectionRefusal> {
    let session = state
        .session
        .as_mut()
        .filter(|session| session.is(dictation_id, generation))
        .ok_or(SelectionRefusal::Stale)?;
    if !session.accepts_choices() {
        return Err(SelectionRefusal::Frozen);
    }
    Ok(session)
}

/// Business rule (014 AC-001): the route a press gives `session`, if it changes, remembering
/// what a second press restores. The re-routing still applies while the selection version is
/// the one it set and an app is pinned.
fn next_route(session: &mut DestinationSession, pinned: bool) -> Option<Selection> {
    let version = session.selection_version;
    let still_applies = session
        .keys
        .retarget
        .take()
        .filter(|retarget| pinned && retarget.version == version);
    match (&session.selection, still_applies) {
        (Selection::FrontmostAtEnd, Some(retarget)) => Some(retarget.previous),
        (Selection::FrontmostAtEnd, None) => None,
        (current, _) if pinned => {
            session.keys.retarget = Some(Retarget {
                previous: current.clone(),
                version: version + 1,
            });
            Some(Selection::FrontmostAtEnd)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "dictation_toggles_tests.rs"]
mod tests;
