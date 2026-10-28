//! Status projection: configured order, availability, warning and effective input (pure).
use super::status;
use crate::microphones::catalog::{Catalog, CatalogEntry};
use crate::microphones::effective::{EffectiveInput, EffectiveScope, EffectiveView};
use crate::microphones::types::{Availability, EffectiveReason};
use serde_json::json;

fn order() -> Vec<String> {
    vec!["USB headset".into(), "Dock".into()]
}

// @spec AC-002: Saved order and microphone in use reported apart — .specs/features/001-microphone-priority/spec.md#ac-002
#[test]
fn a_failed_observation_reports_unknown_statuses_with_a_warning() -> Result<(), serde_json::Error> {
    let none = EffectiveView {
        scope: EffectiveScope::NotUsedYet,
        input: None,
    };
    let report = status::<()>(order(), None, "2026-10-14T10:00:00Z".into(), none);
    let wire = serde_json::to_value(&report)?;
    assert_eq!(wire["warning"], "enumeration_failed");
    assert_eq!(
        wire["availability"][1],
        json!({"name": "Dock", "status": "unknown"})
    );
    let expected =
        json!({"scope": "not_used_yet", "name": null, "reason": null, "capture_generation": null});
    assert_eq!(wire["effective_input"], expected);
    assert_eq!(wire["configured_order"], json!(["USB headset", "Dock"]));
    Ok(())
}
#[test]
fn the_effective_input_is_reported_with_its_scope_and_reason() {
    let catalog = Catalog::new(vec![CatalogEntry {
        name: "Dock".to_owned(),
        device: (),
    }]);
    let input = EffectiveInput {
        name: Some("Dock".into()),
        choice: Some("Dock".into()),
        reason: EffectiveReason::Fallback,
        capture_generation: 4,
    };
    let view = EffectiveView {
        scope: EffectiveScope::Current,
        input: Some(input),
    };
    let report = status(order(), Some(&catalog), "t".into(), view);
    assert_eq!(report.availability[0].status, Availability::Unavailable);
    assert_eq!(
        report.effective_input.reason,
        Some(EffectiveReason::Fallback)
    );
    assert_eq!(report.effective_input.capture_generation, Some(4));
    assert!(report.warning.is_none());
}
