//! The eight 007 result kinds and the legacy results map to status, reason and disposition
//! exactly as the plan's outcome table says; no mapping claims a 006 field verification.
use super::*;
use DestinationFailure::InsertionFailed as Failure;
use RecoveryDisposition::Recover;

fn detailed(outcome: MethodOutcome) -> MethodResult {
    MethodResult::Detailed {
        outcome,
        transaction_id: Some("tx-7".to_owned()),
    }
}

#[test]
fn every_007_kind_maps_to_its_status_reason_and_disposition() {
    use InsertionStatus::{ClipboardHandedOver, Failed, InsertionAttempted};
    use MethodOutcome as Kind;
    let table = [
        (Kind::InsertedVerified, (InsertionAttempted, None, None)),
        (Kind::ReadObserved, (InsertionAttempted, None, None)),
        (Kind::Typed, (InsertionAttempted, None, None)),
        (Kind::ClipboardHandedOver, (ClipboardHandedOver, None, None)),
        (Kind::NotRead, (Failed, Some(Failure), Some(Recover))),
        (
            Kind::PermissionMissing,
            (Failed, Some(Failure), Some(Recover)),
        ),
        (Kind::Uncertain, (Failed, Some(Failure), Some(Recover))),
        (
            Kind::InsertionFailed,
            (Failed, Some(Failure), Some(Recover)),
        ),
    ];
    for (kind, expected) in table {
        let result = detailed(kind);
        assert_eq!(map_method_result(&result), expected, "{kind:?}");
        assert_eq!(result.method_outcome(), Some(kind));
        assert_eq!(result.transaction_id().as_deref(), Some("tx-7"));
    }
}
#[test]
fn legacy_success_is_attempted_and_legacy_error_is_an_insertion_failure() {
    let success = MethodResult::Legacy(Ok(()));
    assert_eq!(
        map_method_result(&success),
        (InsertionStatus::InsertionAttempted, None, None)
    );
    let error = MethodResult::Legacy(Err("Failed to press key".to_owned()));
    assert_eq!(
        map_method_result(&error),
        (InsertionStatus::Failed, Some(Failure), Some(Recover))
    );
    assert_eq!(
        (error.method_outcome(), error.transaction_id()),
        (None, None)
    );
}
#[test]
fn the_wire_values_are_the_007_kind_names() -> Result<(), serde_json::Error> {
    let kinds = serde_json::to_value([
        MethodOutcome::InsertedVerified,
        MethodOutcome::ClipboardHandedOver,
        MethodOutcome::PermissionMissing,
    ])?;
    assert_eq!(
        kinds,
        serde_json::json!([
            "inserted_verified",
            "clipboard_handed_over",
            "permission_missing"
        ])
    );
    let statuses = serde_json::to_value([
        InsertionStatus::InsertionAttempted,
        InsertionStatus::ClipboardHandedOver,
    ])?;
    assert_eq!(
        statuses,
        serde_json::json!(["insertion_attempted", "clipboard_handed_over"])
    );
    Ok(())
}
