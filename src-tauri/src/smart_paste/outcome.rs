//! Truthful smart-paste outcomes consumed by automatic Enter (009) and the
//! recovery bubble (010).
//!
//! One terminal [`PasteOutcome`] is produced per dictation. A read that
//! arrives after "not read" was declared never revises it; it produces a
//! separate [`PasteDiagnostic`] instead (AC-012).

use super::ids::{DestinationGeneration, DictationId, TransactionId};
use crate::destination::outcome::{MethodOutcome, MethodResult};

// @spec FR-008: Truthful outcome contract — .specs/features/007-smart-paste/spec.md#fr-008

/// The eight terminal outcomes of AC-012 and AC-017.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PasteOutcomeKind {
    /// The field was read back through AX and holds the expected value.
    InsertedVerified,
    /// The target read the transcript after the paste keystroke; the field
    /// content itself is not verified.
    ReadObserved,
    /// No qualifying read before the per-application decision deadline.
    NotRead,
    /// Accessibility permission is absent; nothing was posted or published.
    PermissionMissing,
    /// An AX write may have applied but the field does not show the
    /// expected value, so no fallback paste is attempted.
    Uncertain,
    /// Unicode key events were posted; the field content is not verified.
    Typed,
    /// Screen Sharing received the transcript on the clipboard.
    ClipboardHandedOver,
    /// Posting keys failed, or the transaction could not start.
    InsertionFailed,
}

impl PasteOutcomeKind {
    /// Whether automatic Enter (009) may submit after this outcome.
    pub fn is_auto_submit_eligible(self) -> bool {
        matches!(
            self,
            Self::InsertedVerified | Self::ReadObserved | Self::Typed
        )
    }

    /// Whether the recovery bubble (010) treats this outcome as a failure.
    ///
    /// "Clipboard handed over" is deliberately neither a success for Enter
    /// nor a failure for recovery (AC-012).
    pub fn requires_recovery(self) -> bool {
        matches!(
            self,
            Self::NotRead | Self::PermissionMissing | Self::Uncertain | Self::InsertionFailed
        )
    }

    /// Whether the outcome states that the field contains the text.
    ///
    /// Only an AX read-back proves field content; every other outcome makes
    /// no claim about it (AC-012).
    pub fn claims_field_content(self) -> bool {
        self == Self::InsertedVerified
    }
}

/// The insertion path used, recorded for native evidence (AC-016).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsertionPath {
    /// No insertion effect was attempted (for example permission missing).
    NotAttempted,
    /// Verified accessibility insertion (AC-017).
    DirectAccessibility,
    /// On-demand clipboard item followed by Cmd+V (AC-008, AC-009).
    ClipboardPaste,
    /// Unicode key events without the clipboard (AC-010).
    UnicodeTyping,
    /// Eager clipboard write for a remote screen (AC-010).
    ClipboardHandOver,
}

/// The single terminal result of one dictation's insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct PasteOutcome {
    /// Dictation this outcome belongs to.
    pub dictation_id: DictationId,
    /// Clipboard transaction, when one was started.
    pub transaction_id: Option<TransactionId>,
    /// Destination generation supplied by 006, carried for correlation.
    pub destination_generation: DestinationGeneration,
    /// Path that produced the outcome.
    pub path: InsertionPath,
    /// Terminal outcome kind.
    pub kind: PasteOutcomeKind,
}

/// Idempotent diagnostic for a read observed after "not read" was declared.
///
/// It lets 010 update its existing bubble ("the text may have been pasted
/// after all") without promoting the terminal outcome (AC-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct PasteDiagnostic {
    /// Dictation whose transcript was read late.
    pub dictation_id: DictationId,
    /// Transaction whose on-demand item was read late.
    pub transaction_id: TransactionId,
    /// Always true; kept explicit for the event payload.
    pub late_read_observed: bool,
}

// @spec FR-008: Outcome mapped to the 006 method result — .specs/features/007-smart-paste/spec.md#fr-008
impl PasteOutcomeKind {
    /// The 006 `method_outcome` wire value carrying this kind unchanged
    /// (006 destination contract, "insertion method").
    pub fn method_outcome(self) -> MethodOutcome {
        match self {
            Self::InsertedVerified => MethodOutcome::InsertedVerified,
            Self::ReadObserved => MethodOutcome::ReadObserved,
            Self::NotRead => MethodOutcome::NotRead,
            Self::PermissionMissing => MethodOutcome::PermissionMissing,
            Self::Uncertain => MethodOutcome::Uncertain,
            Self::Typed => MethodOutcome::Typed,
            Self::ClipboardHandedOver => MethodOutcome::ClipboardHandedOver,
            Self::InsertionFailed => MethodOutcome::InsertionFailed,
        }
    }
}

impl PasteOutcome {
    /// The typed result 006 maps to its insertion status, failure reason
    /// and recovery disposition; the transaction id is carried as text.
    pub fn method_result(&self) -> MethodResult {
        MethodResult::Detailed {
            outcome: self.kind.method_outcome(),
            transaction_id: self.transaction_id.map(|id| id.0.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PasteOutcomeKind::*;
    use super::*;

    const ALL: [PasteOutcomeKind; 8] = [
        InsertedVerified,
        ReadObserved,
        NotRead,
        PermissionMissing,
        Uncertain,
        Typed,
        ClipboardHandedOver,
        InsertionFailed,
    ];

    #[test]
    fn only_verified_read_and_typed_are_enter_eligible() {
        let eligible: Vec<_> = ALL
            .into_iter()
            .filter(|kind| kind.is_auto_submit_eligible())
            .collect();
        assert_eq!(eligible, vec![InsertedVerified, ReadObserved, Typed]);
    }

    #[test]
    fn recovery_covers_exactly_the_four_failures() {
        let failures: Vec<_> = ALL
            .into_iter()
            .filter(|kind| kind.requires_recovery())
            .collect();
        assert_eq!(
            failures,
            vec![NotRead, PermissionMissing, Uncertain, InsertionFailed]
        );
    }

    #[test]
    fn hand_over_is_neither_success_nor_failure() {
        assert!(!ClipboardHandedOver.is_auto_submit_eligible());
        assert!(!ClipboardHandedOver.requires_recovery());
    }

    #[test]
    fn only_verified_insertion_claims_field_content() {
        let claims: Vec<_> = ALL
            .into_iter()
            .filter(|kind| kind.claims_field_content())
            .collect();
        assert_eq!(claims, vec![InsertedVerified]);
    }

    #[test]
    fn every_kind_keeps_its_wire_value_as_the_006_method_outcome() -> Result<(), serde_json::Error>
    {
        for kind in ALL {
            assert_eq!(
                serde_json::to_value(kind.method_outcome())?,
                serde_json::to_value(kind)?
            );
        }
        Ok(())
    }

    #[test]
    fn destination_maps_enter_eligible_kinds_to_attempts_and_failures_to_recovery() {
        use crate::destination::context::RecoveryDisposition::Recover;
        use crate::destination::outcome::{map_method_result, InsertionStatus as Status};
        for kind in ALL {
            let outcome = PasteOutcome {
                dictation_id: DictationId(1),
                transaction_id: Some(TransactionId(9)),
                destination_generation: DestinationGeneration(2),
                path: InsertionPath::ClipboardPaste,
                kind,
            };
            let result = outcome.method_result();
            assert_eq!(result.transaction_id().as_deref(), Some("9"));
            let (status, reason, disposition) = map_method_result(&result);
            let expected = match kind {
                ClipboardHandedOver => Status::ClipboardHandedOver,
                _ if kind.is_auto_submit_eligible() => Status::InsertionAttempted,
                _ => Status::Failed,
            };
            assert_eq!(status, expected, "{kind:?}");
            assert_eq!(reason.is_some(), kind.requires_recovery(), "{kind:?}");
            assert_eq!(disposition == Some(Recover), kind.requires_recovery());
        }
    }

    #[test]
    fn wire_values_are_snake_case() -> Result<(), serde_json::Error> {
        let wire: Vec<String> = ALL
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<_, _>>()?;
        assert_eq!(
            wire,
            [
                "\"inserted_verified\"",
                "\"read_observed\"",
                "\"not_read\"",
                "\"permission_missing\"",
                "\"uncertain\"",
                "\"typed\"",
                "\"clipboard_handed_over\"",
                "\"insertion_failed\"",
            ]
        );
        Ok(())
    }
}
