//! Pure recovery decision after a terminal outcome (plan "Recovery decision table", first
//! match): which outcomes show the bubble, with which reason and detail, per platform.
use super::types::{BubbleReason, BubbleSpec, ReasonDetail, RecoveryPlatform};
use crate::destination::outcome::{DestinationFailure, InsertionStatus, MethodOutcome};

// @spec FR-001: Decide insert or recover from the outcome — .specs/features/010-recovery-bubble/spec.md#fr-001
/// The bubble an outcome calls for, or `None` (inserted, skipped, cancelled, handed over).
/// A failure with a `method_outcome` is classified from it; one without, from its reason alone.
pub fn decide_recovery(
    status: InsertionStatus,
    reason: Option<DestinationFailure>,
    method_outcome: Option<MethodOutcome>,
    platform: RecoveryPlatform,
) -> Option<BubbleSpec> {
    if status != InsertionStatus::Failed {
        return None;
    }
    let spec = match method_outcome {
        Some(kind) => from_method(kind)?,
        None => from_reason(reason),
    };
    offered_on(spec.reason, platform).then_some(spec)
}

/// Smart paste terminal outcomes (007 wire values) that recover.
fn from_method(kind: MethodOutcome) -> Option<BubbleSpec> {
    let detail = match kind {
        MethodOutcome::NotRead | MethodOutcome::Uncertain | MethodOutcome::InsertionFailed => {
            ReasonDetail::MayHaveReached
        }
        // 007 checks the permission before any keystroke or clipboard change.
        MethodOutcome::PermissionMissing => ReasonDetail::NothingSent,
        MethodOutcome::InsertedVerified
        | MethodOutcome::ReadObserved
        | MethodOutcome::Typed
        | MethodOutcome::ClipboardHandedOver => return None,
    };
    Some(BubbleSpec {
        reason: BubbleReason::InsertionFailed,
        detail,
    })
}

/// A failure without a method outcome: destination reasons decided before insertion never
/// carry the partial-insertion detail; a legacy method failure (classic, direct, script,
/// Windows reliable paste) may have sent input.
fn from_reason(reason: Option<DestinationFailure>) -> BubbleSpec {
    let before = |reason| BubbleSpec {
        reason,
        detail: ReasonDetail::NotInserted,
    };
    match reason {
        Some(DestinationFailure::AppClosed) => before(BubbleReason::AppClosed),
        Some(DestinationFailure::PinnedAppNotRunning) => before(BubbleReason::PinnedAppNotRunning),
        Some(DestinationFailure::ActivationFailed) => before(BubbleReason::ActivationFailed),
        Some(DestinationFailure::NoTextField) => before(BubbleReason::NoTextField),
        // @spec AC-004: Field return failed and nothing sent — .specs/features/011-dictation-field-return/spec.md#ac-004
        // 011: decided at the guard before any dispatch; 010's existing "Insertion failed" with
        // its explanation that nothing was sent to the application.
        Some(DestinationFailure::FieldReturnFailed) => BubbleSpec {
            reason: BubbleReason::InsertionFailed,
            detail: ReasonDetail::NothingSent,
        },
        // A failure without a reason is still a failure: keep the text rather than lose it.
        Some(DestinationFailure::InsertionFailed) | None => BubbleSpec {
            reason: BubbleReason::InsertionFailed,
            detail: ReasonDetail::MayHaveReached,
        },
    }
}

// @spec AC-006: Windows and Linux recovery causes — .specs/features/010-recovery-bubble/spec.md#ac-006
/// Business rule (spec AC-006): Linux captures no destination, so only "Insertion failed"
/// recovers there; field detection exists on macOS only.
fn offered_on(reason: BubbleReason, platform: RecoveryPlatform) -> bool {
    match platform {
        RecoveryPlatform::MacOs => true,
        RecoveryPlatform::Windows => reason != BubbleReason::NoTextField,
        RecoveryPlatform::Linux => reason == BubbleReason::InsertionFailed,
    }
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod tests;
