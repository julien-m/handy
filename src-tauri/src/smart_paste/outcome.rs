//! Truthful smart-paste outcomes consumed by automatic Enter (009) and the
//! recovery bubble (010).
//!
//! One terminal [`PasteOutcome`] is produced per dictation. A read that
//! arrives after "not read" was declared never revises it; it produces a
//! separate [`PasteDiagnostic`] instead (AC-012).

use super::ids::{DestinationGeneration, DictationId, TransactionId};

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
