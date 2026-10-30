//! SC-001: the decision table over every status, reason, method outcome and platform, against
//! an independent expectation written row by row from the spec (zero mismatch).
use super::*;
use crate::destination::outcome::{DestinationFailure as Failure, MethodOutcome as Kind};

const STATUSES: [InsertionStatus; 5] = [
    InsertionStatus::InsertionAttempted,
    InsertionStatus::Failed,
    InsertionStatus::ClipboardHandedOver,
    InsertionStatus::Skipped,
    InsertionStatus::Cancelled,
];
const REASONS: [Option<Failure>; 9] = [
    None,
    Some(Failure::AppClosed),
    Some(Failure::ActivationFailed),
    Some(Failure::InsertionFailed),
    Some(Failure::PinnedAppNotRunning),
    Some(Failure::NoTextField),
    Some(Failure::FieldReturnFailed),
    Some(Failure::DefaultFieldNotFound),
    Some(Failure::DefaultTerminalGone),
];
const KINDS: [Option<Kind>; 10] = [
    None,
    Some(Kind::InsertedVerified),
    Some(Kind::ReadObserved),
    Some(Kind::NotRead),
    Some(Kind::PermissionMissing),
    Some(Kind::Uncertain),
    Some(Kind::Typed),
    Some(Kind::ClipboardHandedOver),
    Some(Kind::InsertionFailed),
    Some(Kind::FieldUnchanged),
];
const PLATFORMS: [RecoveryPlatform; 3] = [
    RecoveryPlatform::MacOs,
    RecoveryPlatform::Windows,
    RecoveryPlatform::Linux,
];

/// The spec, written independently of the implementation.
fn expected(
    status: InsertionStatus,
    reason: Option<Failure>,
    kind: Option<Kind>,
    platform: RecoveryPlatform,
) -> Option<(BubbleReason, ReasonDetail)> {
    if status != InsertionStatus::Failed {
        return None;
    }
    let row = row_of(kind, reason)?;
    let offered = match platform {
        RecoveryPlatform::MacOs => true,
        RecoveryPlatform::Windows => row.0 != BubbleReason::NoTextField,
        RecoveryPlatform::Linux => row.0 == BubbleReason::InsertionFailed,
    };
    offered.then_some(row)
}

/// The reason and detail of a failure, before the platform filter.
fn row_of(kind: Option<Kind>, reason: Option<Failure>) -> Option<(BubbleReason, ReasonDetail)> {
    use BubbleReason as R;
    use ReasonDetail as D;
    Some(match (kind, reason) {
        (
            Some(Kind::NotRead | Kind::Uncertain | Kind::InsertionFailed | Kind::FieldUnchanged),
            _,
        ) => (R::InsertionFailed, D::MayHaveReached),
        (Some(Kind::PermissionMissing), _) | (None, Some(Failure::FieldReturnFailed)) => {
            (R::InsertionFailed, D::NothingSent)
        }
        (Some(_), _) => return None,
        (None, Some(Failure::AppClosed)) => (R::AppClosed, D::NotInserted),
        (None, Some(Failure::PinnedAppNotRunning)) => (R::PinnedAppNotRunning, D::NotInserted),
        (None, Some(Failure::ActivationFailed)) => (R::ActivationFailed, D::NotInserted),
        (None, Some(Failure::NoTextField)) => (R::NoTextField, D::NotInserted),
        // 013: a remembered target that was not found is decided before any dispatch.
        (None, Some(Failure::DefaultFieldNotFound)) => (R::DefaultFieldNotFound, D::NotInserted),
        (None, Some(Failure::DefaultTerminalGone)) => (R::DefaultTerminalGone, D::NotInserted),
        (None, Some(Failure::InsertionFailed) | None) => (R::InsertionFailed, D::MayHaveReached),
    })
}

// @spec SC-001: Decision table with zero mismatch — .specs/features/010-recovery-bubble/spec.md#sc-001
#[test]
fn every_status_reason_outcome_and_platform_matches_the_spec_table() {
    let mut cases = 0;
    for status in STATUSES {
        for reason in REASONS {
            for kind in KINDS {
                for platform in PLATFORMS {
                    let decided = decide_recovery(status, reason, kind, platform)
                        .map(|spec| (spec.reason, spec.detail));
                    let wanted = expected(status, reason, kind, platform);
                    assert_eq!(
                        decided, wanted,
                        "{status:?} {reason:?} {kind:?} {platform:?}"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 5 * 9 * 10 * 3);
}

// @spec AC-001: Reasons and the partial-insertion detail — .specs/features/010-recovery-bubble/spec.md#ac-001
#[test]
fn a_reason_decided_before_insertion_never_shows_the_partial_insertion_detail() {
    let mac = RecoveryPlatform::MacOs;
    let no_field = decide_recovery(
        InsertionStatus::Failed,
        Some(Failure::NoTextField),
        None,
        mac,
    );
    let legacy = decide_recovery(
        InsertionStatus::Failed,
        Some(Failure::InsertionFailed),
        None,
        mac,
    );
    assert_eq!(
        no_field.map(|spec| (spec.reason, spec.detail)),
        Some((BubbleReason::NoTextField, ReasonDetail::NotInserted))
    );
    assert_eq!(
        legacy.map(|spec| (spec.reason, spec.detail)),
        Some((BubbleReason::InsertionFailed, ReasonDetail::MayHaveReached))
    );
}

#[test]
fn permission_missing_says_nothing_was_sent_and_the_windows_reliable_failure_may_have_reached() {
    let permission = decide_recovery(
        InsertionStatus::Failed,
        Some(Failure::InsertionFailed),
        Some(Kind::PermissionMissing),
        RecoveryPlatform::MacOs,
    );
    assert_eq!(
        permission.map(|spec| spec.detail),
        Some(ReasonDetail::NothingSent)
    );
    let reliable = decide_recovery(
        InsertionStatus::Failed,
        Some(Failure::InsertionFailed),
        None,
        RecoveryPlatform::Windows,
    );
    assert_eq!(
        reliable.map(|spec| spec.detail),
        Some(ReasonDetail::MayHaveReached)
    );
}
