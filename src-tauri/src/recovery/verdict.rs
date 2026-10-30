//! Pre-insertion verdict of 010 (spec AC-002, plan "Recovery decision table"): classify only
//! when the verdict can matter — macOS, setting on (frozen at dictation start), a named
//! destination outside 007's per-application strategies — and answer `Recover(NoTextField)`
//! only for a positive "not editable".
use super::classifier::FocusClassification;
use super::types::RecoveryPlatform;
use crate::destination::context::{PreInsertionVerdict, RecoveryReason, ResolvedTarget};
use crate::smart_paste::strategy::{
    IPHONE_MIRRORING_BUNDLE_ID, SCREEN_SHARING_BUNDLE_ID, WINDOWS_APP_BUNDLE_ID,
};
use crate::smart_paste::timing::JUMP_DESKTOP_BUNDLE_ID;

/// Destinations whose remote field cannot be read locally (spec AC-002): an explicit list of
/// 007's bundle identifiers, not `InsertionStrategy::for_bundle` (which maps Jump Desktop to
/// direct insertion first).
pub const REMOTE_VIEW_BUNDLES: [&str; 4] = [
    SCREEN_SHARING_BUNDLE_ID,
    IPHONE_MIRRORING_BUNDLE_ID,
    WINDOWS_APP_BUNDLE_ID,
    JUMP_DESKTOP_BUNDLE_ID,
];

// @spec AC-002: Classify only when the verdict can matter — .specs/features/010-recovery-bubble/spec.md#ac-002
/// The process to classify, or `None` to insert at once without any accessibility call.
pub fn classification_target(
    platform: RecoveryPlatform,
    setting_on: bool,
    target: Option<&ResolvedTarget>,
) -> Option<u32> {
    if platform != RecoveryPlatform::MacOs || !setting_on {
        return None;
    }
    let target = target?;
    let remote = target
        .identity
        .bundle_id()
        .is_some_and(|bundle| REMOTE_VIEW_BUNDLES.contains(&bundle));
    (!remote).then_some(target.pid)
}

/// Only a positive "not editable" recovers; editable, undetermined and a missing permission
/// insert as usual.
pub fn verdict_of(classification: FocusClassification) -> PreInsertionVerdict {
    match classification {
        FocusClassification::NotEditable => {
            PreInsertionVerdict::Recover(RecoveryReason::NoTextField)
        }
        FocusClassification::Editable
        | FocusClassification::Undetermined(_)
        | FocusClassification::PermissionMissing => PreInsertionVerdict::Insert,
    }
}

#[cfg(test)]
#[path = "verdict_tests.rs"]
mod tests;
