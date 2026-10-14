//! Terminal insertion outcome v1 and its mapping from insertion-method results. 006 reports
//! what it attempted, never a field verification of its own: only 007's `inserted_verified`
//! carries read-back evidence. The UI event carries metadata only; the transcript stays in the
//! native recovery slot behind an opaque reference.
use super::context::{ApplicationIdentity, DestinationSource, DictationId, RecoveryDisposition};
use serde::Serialize;

/// Destination status of one terminal dictation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum InsertionStatus {
    InsertionAttempted,
    Failed,
    ClipboardHandedOver,
    Skipped,
    Cancelled,
}
/// Why the destination flow failed (009 adds `pinned_app_not_running`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DestinationFailure {
    AppClosed,
    ActivationFailed,
    InsertionFailed,
}
/// 007's `PasteOutcome.kind` wire values, carried unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum MethodOutcome {
    InsertedVerified,
    ReadObserved,
    NotRead,
    PermissionMissing,
    Uncertain,
    Typed,
    ClipboardHandedOver,
    InsertionFailed,
}
/// History result of this dictation, from the actual `save_entry` result only (a WAV failure
/// still saves a text-only row: `saved`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum HistoryStatus {
    Saved,
    /// No write attempted (cancellation observed before the write).
    NotSaved,
    Failed,
}
/// Opaque reference to a transcript retained in the native recovery slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, specta::Type)]
#[serde(transparent)]
pub struct TranscriptRef(pub u64);

/// Result of one insertion method run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MethodResult {
    /// An existing method without detailed result: success or its error text (never shown).
    Legacy(Result<(), String>),
    /// A method with a typed terminal result (007 smart paste).
    Detailed {
        outcome: MethodOutcome,
        transaction_id: Option<String>,
    },
}

/// Status, failure reason and the disposition after the result (`None`: unchanged).
pub type MethodMapping = (
    InsertionStatus,
    Option<DestinationFailure>,
    Option<RecoveryDisposition>,
);

// @spec FR-005: Attempted versus verified outcome mapping — .specs/features/006-destination-application/spec.md#fr-005
/// Business rule (plan "Outcome" table): verified, read-observed and typed results and a legacy
/// success are attempts that keep `Insert`; a clipboard hand-over keeps `Insert` (009 sends no
/// Enter for it); every other result or a legacy error is an insertion failure to recover.
pub fn map_method_result(result: &MethodResult) -> MethodMapping {
    use InsertionStatus::{ClipboardHandedOver, Failed, InsertionAttempted};
    use MethodOutcome as Kind;
    let failed = (
        Failed,
        Some(DestinationFailure::InsertionFailed),
        Some(RecoveryDisposition::Recover),
    );
    match result {
        MethodResult::Legacy(Ok(())) => (InsertionAttempted, None, None),
        MethodResult::Legacy(Err(_)) => failed,
        MethodResult::Detailed { outcome, .. } => match outcome {
            Kind::InsertedVerified | Kind::ReadObserved | Kind::Typed => {
                (InsertionAttempted, None, None)
            }
            Kind::ClipboardHandedOver => (ClipboardHandedOver, None, None),
            Kind::NotRead | Kind::PermissionMissing | Kind::Uncertain | Kind::InsertionFailed => {
                failed
            }
        },
    }
}
impl MethodResult {
    /// The exact 007 kind, absent for a legacy method.
    pub fn method_outcome(&self) -> Option<MethodOutcome> {
        match self {
            Self::Legacy(_) => None,
            Self::Detailed { outcome, .. } => Some(*outcome),
        }
    }
    /// 007's transaction, when smart paste ran.
    pub fn transaction_id(&self) -> Option<String> {
        match self {
            Self::Legacy(_) => None,
            Self::Detailed { transaction_id, .. } => transaction_id.clone(),
        }
    }
}

// @spec FR-004: Truthful outcome with history status — .specs/features/006-destination-application/spec.md#fr-004
/// Terminal outcome of one dictation, published once (`insertion-outcome`). Metadata only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct InsertionOutcome {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    /// Deduplication key of consumers.
    pub outcome_id: u64,
    pub target: Option<ApplicationIdentity>,
    pub destination_source: DestinationSource,
    pub status: InsertionStatus,
    pub method_outcome: Option<MethodOutcome>,
    pub transaction_id: Option<String>,
    pub reason: Option<DestinationFailure>,
    pub history_status: HistoryStatus,
    pub history_entry_id: Option<i64>,
    pub transcript_ref: Option<TranscriptRef>,
    /// Milliseconds since the Unix epoch.
    pub completed_at: u64,
}

#[cfg(test)]
#[path = "outcome_tests.rs"]
mod tests;
