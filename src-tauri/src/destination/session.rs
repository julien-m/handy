//! One dictation's destination session and its pure transitions: explicit badge choices and the
//! one-click removal guarded by `selection_version`, the insertion-entry freeze, the monotonic
//! `RecoveryDisposition` and the single Enter claim of 009. The manager owns the lock; nothing
//! here performs I/O or is persisted.
use super::context::{ApplicationIdentity, DestinationSource, DictationId, RecoveryDisposition};
use crate::settings::OverlayStyle;
use serde::Serialize;
use std::time::Instant;

/// Why destination return is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SupportReason {
    /// This build has no destination capability (Linux).
    PlatformUnsupported,
    /// The running instance cannot read the frontmost application.
    FrontmostUnreadable,
}
/// Runtime support of destination return in the running instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "state", content = "reason", rename_all = "snake_case")]
pub enum DestinationSupport {
    Supported,
    Unsupported(SupportReason),
}

/// The application chosen for one dictation (captured at start or chosen from the badge).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Destination {
    pub identity: ApplicationIdentity,
    pub display_name: String,
    pub icon_ref: Option<String>,
    pub captured_pid: u32,
    pub source: DestinationSource,
}
/// Where this dictation's text goes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    /// The application frontmost at insertion, as before this feature.
    FrontmostAtEnd,
    Application(Destination),
}
impl Selection {
    /// Source of this selection.
    pub fn source(&self) -> DestinationSource {
        match self {
            Self::FrontmostAtEnd => DestinationSource::FrontmostAtEnd,
            Self::Application(destination) => destination.source,
        }
    }
}

/// Lifecycle of one dictation's session.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Recording,
    Processing,
    Inserting,
    Finished,
    Cancelled,
}

/// A refused badge choice; the session is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SelectionRefusal {
    /// Another dictation, or an older `selection_version`.
    Stale,
    /// Insertion has begun (or the session ended): choices are closed.
    Frozen,
    /// The chosen entry is unknown, closed or no longer running.
    Unavailable,
}
/// A refused disposition change; the disposition is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispositionRefusal {
    /// Another dictation or an older generation.
    Stale,
    /// The change would go backwards (`Recover`/`Cancelled` are final, `Insert` never returns
    /// to `Pending`); carries the current disposition.
    Final(RecoveryDisposition),
}
/// Enter claim granted to 009, once per dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnterClaim {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
}
/// Why 009 may not send Enter for this dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnterRefusal {
    Stale,
    Disposition(RecoveryDisposition),
    AlreadyClaimed,
}

/// Destination state of one accepted dictation. `return_enabled` and `support` are frozen at
/// start so a settings change applies to the next dictation.
#[derive(Clone, Debug, PartialEq)]
pub struct DestinationSession {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub selection_version: u64,
    pub phase: SessionPhase,
    pub return_enabled: bool,
    pub support: DestinationSupport,
    pub selection: Selection,
    pub captured_at: Instant,
    pub disposition: RecoveryDisposition,
    pub enter_claimed: bool,
}

impl DestinationSession {
    /// True when `dictation_id`/`generation` name this session.
    pub fn is(&self, dictation_id: DictationId, generation: u64) -> bool {
        self.dictation_id == dictation_id && self.destination_generation == generation
    }
    /// True while badge choices are accepted (before insertion begins).
    pub fn accepts_choices(&self) -> bool {
        matches!(
            self.phase,
            SessionPhase::Recording | SessionPhase::Processing
        )
    }
    // @spec FR-001: Stable destination unless changed from the badge — .specs/features/006-destination-application/spec.md#fr-001
    /// Apply a badge choice (or the cross, with `FrontmostAtEnd`) for the exact dictation and
    /// version. Choosing the current destination again is a no-op that keeps the version.
    ///
    /// # Errors
    /// `Stale` for another dictation or version, `Frozen` once insertion has begun.
    pub fn choose(
        &mut self,
        dictation_id: DictationId,
        expected_version: u64,
        selection: Selection,
    ) -> Result<(), SelectionRefusal> {
        if self.dictation_id != dictation_id || self.selection_version != expected_version {
            return Err(SelectionRefusal::Stale);
        }
        if !self.accepts_choices() {
            return Err(SelectionRefusal::Frozen);
        }
        if same_target(&self.selection, &selection) {
            return Ok(());
        }
        self.selection = selection;
        self.selection_version += 1;
        Ok(())
    }
    /// Insertion entry: freeze the selection and close choices.
    ///
    /// # Errors
    /// `Stale` for another dictation or generation, or a session that already left processing.
    pub fn freeze(
        &mut self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<Selection, DispositionRefusal> {
        if !self.is(dictation_id, generation) || !self.accepts_choices() {
            return Err(DispositionRefusal::Stale);
        }
        self.phase = SessionPhase::Inserting;
        Ok(self.selection.clone())
    }
    /// Move the disposition forward (see `RecoveryDisposition::may_become`).
    ///
    /// # Errors
    /// `Stale` for another dictation or generation; `Final` for a backward change.
    pub fn set_disposition(
        &mut self,
        dictation_id: DictationId,
        generation: u64,
        next: RecoveryDisposition,
    ) -> Result<RecoveryDisposition, DispositionRefusal> {
        if !self.is(dictation_id, generation) {
            return Err(DispositionRefusal::Stale);
        }
        if !self.disposition.may_become(next) {
            return Err(DispositionRefusal::Final(self.disposition));
        }
        self.disposition = next;
        Ok(next)
    }
    // @spec FR-005: Enter claim reads the disposition atomically — .specs/features/006-destination-application/spec.md#fr-005
    /// Grant 009 its Enter claim once, only while the disposition is `Insert`. A claim never
    /// changes the disposition and a later `Recover` does not revoke it.
    ///
    /// # Errors
    /// `Stale`, `Disposition(current)` or `AlreadyClaimed`.
    pub fn claim_enter(
        &mut self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Result<EnterClaim, EnterRefusal> {
        if !self.is(dictation_id, generation) {
            return Err(EnterRefusal::Stale);
        }
        if self.disposition != RecoveryDisposition::Insert {
            return Err(EnterRefusal::Disposition(self.disposition));
        }
        if self.enter_claimed {
            return Err(EnterRefusal::AlreadyClaimed);
        }
        self.enter_claimed = true;
        Ok(EnterClaim {
            dictation_id,
            destination_generation: generation,
        })
    }
}

/// Fixed English descriptions of the refusals (logs and test diagnostics; never user text).
macro_rules! refusal_error {
    ($($kind:ty => $text:literal),+ $(,)?) => {$(
        impl std::fmt::Display for $kind {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}: {:?}", $text, self)
            }
        }
        impl std::error::Error for $kind {}
    )+};
}
refusal_error! {
    SelectionRefusal => "destination choice refused",
    DispositionRefusal => "destination disposition refused",
    EnterRefusal => "Enter claim refused",
}

/// Same target: both "frontmost at the end", or the same application identity.
fn same_target(current: &Selection, next: &Selection) -> bool {
    match (current, next) {
        (Selection::FrontmostAtEnd, Selection::FrontmostAtEnd) => true,
        (Selection::Application(current), Selection::Application(next)) => {
            current.identity == next.identity
        }
        _ => false,
    }
}

// @spec FR-003: Badge visibility predicate — .specs/features/006-destination-application/spec.md#fr-003
/// The badge is shown when destination return is on and supported and the overlay is shown.
pub fn badge_visible(
    return_enabled: bool,
    support: DestinationSupport,
    overlay_style: OverlayStyle,
) -> bool {
    return_enabled
        && support == DestinationSupport::Supported
        && overlay_style != OverlayStyle::None
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
