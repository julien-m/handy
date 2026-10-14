//! Pure decisions about what happens to the clipboard once a transaction
//! settles (AC-009, AC-013, Edge Cases) and which snapshot a new dictation
//! starts from.
//!
//! Every write decided here is applied by the coordinator only while Handy
//! still owns the pasteboard (`changeCount` unchanged since publication);
//! otherwise the newer content wins and nothing is written.

// @spec FR-006: Guarded restoration decisions — .specs/features/007-smart-paste/spec.md#fr-006

/// The user's clipboard handling setting, as seen by smart paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardPolicy {
    /// Put the previous clipboard back after a confirmed read.
    DontModify,
    /// Leave the transcript on the clipboard as plain text (AC-013).
    CopyToClipboard,
}

/// Why a pending clipboard transaction is being settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementTrigger {
    /// Reads went quiet after a qualifying read, or the cap elapsed.
    ReadsWentQuiet,
    /// "Not read" was declared at the per-application deadline.
    NotReadDeclared,
    /// The keystroke was cancelled after publication (destination lost or
    /// a late accessibility write appeared): no read can be expected.
    KeystrokeCancelled,
    /// Handy is exiting normally.
    Exit {
        /// A qualifying read was observed and its quiet period or cap elapsed.
        is_restoration_due: bool,
    },
}

/// Clipboard write chosen for a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementAction {
    /// Put back the saved items and types, or clear if it was empty.
    RestoreSnapshot,
    /// Replace the on-demand item with eager plain text of the transcript.
    MaterializeTranscript,
    /// Leave the on-demand item in place (still served on request).
    KeepLazyTranscript,
    /// Ownership was lost: write nothing.
    LeaveNewerContent,
}

impl SettlementAction {
    /// Whether this action writes to the pasteboard and therefore requires
    /// the ownership guard.
    pub fn writes_pasteboard(self) -> bool {
        matches!(self, Self::RestoreSnapshot | Self::MaterializeTranscript)
    }
}

/// Chooses the clipboard action for a settlement, before the ownership guard.
///
/// Business rules: restoration only ever follows a confirmed read whose
/// quiet period elapsed; "not read" never restores (the previous content
/// could otherwise be pasted by a slow target); copy mode keeps the
/// transcript as plain text instead of restoring; exit and a cancelled
/// keystroke materialize the transcript whenever restoration is not due.
pub fn settlement_action(trigger: SettlementTrigger, policy: ClipboardPolicy) -> SettlementAction {
    match (trigger, policy) {
        (SettlementTrigger::ReadsWentQuiet, ClipboardPolicy::DontModify) => {
            SettlementAction::RestoreSnapshot
        }
        (SettlementTrigger::NotReadDeclared, ClipboardPolicy::DontModify) => {
            SettlementAction::KeepLazyTranscript
        }
        (
            SettlementTrigger::Exit {
                is_restoration_due: true,
            },
            ClipboardPolicy::DontModify,
        ) => SettlementAction::RestoreSnapshot,
        _ => SettlementAction::MaterializeTranscript,
    }
}

/// Applies the ownership guard: a write decided for a pasteboard that
/// changed since publication becomes [`SettlementAction::LeaveNewerContent`].
pub fn guard_settlement(action: SettlementAction, is_still_owned: bool) -> SettlementAction {
    if action.writes_pasteboard() && !is_still_owned {
        SettlementAction::LeaveNewerContent
    } else {
        action
    }
}

/// What a pending transaction still holds when a new dictation starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingClipboardView {
    /// `changeCount` still equals the pending publication count.
    pub is_still_owned: bool,
    /// The pending transaction kept a snapshot it may still restore
    /// (not discarded by "not read", and not copy mode).
    pub is_snapshot_restorable: bool,
}

/// Where the snapshot of a new clipboard transaction comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotSource {
    /// Capture the current pasteboard through the bounded helper.
    CaptureCurrent,
    /// Reuse the pending transaction's snapshot: no intermediate restoration.
    CarryPendingSnapshot,
    /// The pasteboard holds Handy's own earlier transcript: snapshot its
    /// known bytes instead of reading (and self-triggering) the provider.
    PendingTranscript,
}

/// Chooses the snapshot source of a new clipboard transaction.
///
/// Business rule (Edge Cases, AC-009): a second dictation never restores in
/// between. If Handy still owns the pasteboard, the first snapshot stays the
/// content to restore, unless "not read" already discarded it, in which case
/// the earlier transcript is the current content and the pre-first snapshot
/// never returns. If someone else wrote meanwhile, the current content is the
/// new snapshot.
pub fn snapshot_source(pending: Option<PendingClipboardView>) -> SnapshotSource {
    match pending {
        Some(view) if view.is_still_owned && view.is_snapshot_restorable => {
            SnapshotSource::CarryPendingSnapshot
        }
        Some(view) if view.is_still_owned => SnapshotSource::PendingTranscript,
        _ => SnapshotSource::CaptureCurrent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClipboardPolicy::*;
    use SettlementAction::*;
    use SettlementTrigger::*;

    #[test]
    fn quiet_reads_restore_unless_copy_mode() {
        assert_eq!(
            settlement_action(ReadsWentQuiet, DontModify),
            RestoreSnapshot
        );
        assert_eq!(
            settlement_action(ReadsWentQuiet, CopyToClipboard),
            MaterializeTranscript
        );
    }

    #[test]
    fn not_read_never_restores() {
        assert_eq!(
            settlement_action(NotReadDeclared, DontModify),
            KeepLazyTranscript
        );
        assert_eq!(
            settlement_action(NotReadDeclared, CopyToClipboard),
            MaterializeTranscript
        );
    }

    #[test]
    fn exit_restores_only_when_due_and_not_copy_mode() {
        let due = Exit {
            is_restoration_due: true,
        };
        let not_due = Exit {
            is_restoration_due: false,
        };
        assert_eq!(settlement_action(due, DontModify), RestoreSnapshot);
        assert_eq!(
            settlement_action(not_due, DontModify),
            MaterializeTranscript
        );
        assert_eq!(
            settlement_action(due, CopyToClipboard),
            MaterializeTranscript
        );
    }

    #[test]
    fn cancelled_keystroke_materializes_and_never_restores() {
        for policy in [DontModify, CopyToClipboard] {
            assert_eq!(
                settlement_action(KeystrokeCancelled, policy),
                MaterializeTranscript
            );
        }
    }

    #[test]
    fn lost_ownership_turns_every_write_into_no_write() {
        for action in [RestoreSnapshot, MaterializeTranscript] {
            assert_eq!(guard_settlement(action, false), LeaveNewerContent);
            assert_eq!(guard_settlement(action, true), action);
        }
        assert_eq!(
            guard_settlement(KeepLazyTranscript, false),
            KeepLazyTranscript
        );
    }

    #[test]
    fn second_dictation_snapshot_source_follows_ownership_and_retention() {
        let view = |is_still_owned, is_snapshot_restorable| {
            Some(PendingClipboardView {
                is_still_owned,
                is_snapshot_restorable,
            })
        };
        assert_eq!(snapshot_source(None), SnapshotSource::CaptureCurrent);
        assert_eq!(
            snapshot_source(view(true, true)),
            SnapshotSource::CarryPendingSnapshot
        );
        assert_eq!(
            snapshot_source(view(true, false)),
            SnapshotSource::PendingTranscript
        );
        assert_eq!(
            snapshot_source(view(false, true)),
            SnapshotSource::CaptureCurrent
        );
    }
}
