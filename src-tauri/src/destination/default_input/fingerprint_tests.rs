// LiveSpec traceability: @spec(AC-002) @spec(AC-006)
//! Fingerprint capture and match rules (013 FR-002, AC-002, AC-006).
use super::*;
use crate::destination::default_input::fake_tree::TestResult;

fn facts(role: &str, title: Option<&str>, description: Option<&str>) -> NodeFacts {
    NodeFacts {
        role: Some(role.to_owned()),
        title: title.map(str::to_owned),
        description: description.map(str::to_owned),
        ..NodeFacts::default()
    }
}

// @spec AC-002: Generated DOM ids are never kept — .specs/features/013-default-input-per-app/spec.md#ac-002
#[test]
fn generated_dom_ids_are_recognized() -> TestResult {
    for generated in [
        "_ComposerLayoutBody_1q7o3_2",
        "radix-:r3:",
        "3ece0fca-4b38-4067-9b31-32a337ecb039",
        "ember123",
        "field-0042",
    ] {
        assert!(looks_generated(generated), "{generated}");
    }
    for stable in ["prompt-textarea", "search_input", "composer"] {
        assert!(!looks_generated(stable), "{stable}");
    }
    assert!(looks_generated(&"a".repeat(65)));
    Ok(())
}

#[test]
fn the_name_is_the_title_else_the_description_and_classes_never_enter() -> TestResult {
    let composer = facts("AXTextArea", None, Some("Work with Codex"));
    let fingerprint =
        fingerprint_of(&composer).map_err(|error| format!("fingerprint: {error:?}"))?;
    assert_eq!(fingerprint.name.as_deref(), Some("Work with Codex"));
    let titled = facts("AXTextArea", Some("Title"), Some("Description"));
    assert_eq!(
        fingerprint_of(&titled)
            .map_err(|error| format!("fingerprint: {error:?}"))?
            .name
            .as_deref(),
        Some("Title")
    );
    Ok(())
}

#[test]
fn a_generated_dom_id_is_dropped_and_a_stable_one_kept() -> TestResult {
    let mut element = facts("AXTextArea", None, None);
    element.dom_id = Some("_ComposerLayoutBody_1q7o3_2".to_owned());
    assert_eq!(
        fingerprint_of(&element),
        Err(FingerprintRefusal::Unrecognizable)
    );
    element.dom_id = Some("prompt-textarea".to_owned());
    assert_eq!(
        fingerprint_of(&element)
            .map_err(|error| format!("kept: {error:?}"))?
            .dom_id
            .as_deref(),
        Some("prompt-textarea")
    );
    Ok(())
}

#[test]
fn password_fields_are_refused_by_role_or_subrole() -> TestResult {
    let by_role = facts(SECURE_TEXT_FIELD, Some("Password"), None);
    let mut by_subrole = facts("AXTextField", Some("Password"), None);
    by_subrole.subrole = Some(SECURE_TEXT_FIELD.to_owned());
    for element in [by_role, by_subrole] {
        assert_eq!(fingerprint_of(&element), Err(FingerprintRefusal::Password));
        let fingerprint = FieldFingerprint {
            role: "AXTextField".to_owned(),
            subrole: None,
            name: Some("Password".to_owned()),
            placeholder: None,
            dom_id: None,
        };
        assert!(!matches(&fingerprint, &element));
    }
    Ok(())
}

#[test]
fn a_long_attribute_counts_as_absent() -> TestResult {
    let mut element = facts("AXTextField", None, None);
    element.placeholder = Some("p".repeat(MAX_TEXT_CHARS + 1));
    assert_eq!(
        fingerprint_of(&element),
        Err(FingerprintRefusal::Unrecognizable)
    );
    Ok(())
}

// @spec AC-006: Every attribute equal, never a password — .specs/features/013-default-input-per-app/spec.md#ac-006
#[test]
fn every_attribute_must_match_and_absent_means_absent() -> TestResult {
    let composer = facts("AXTextArea", Some("Work with Codex"), None);
    let fingerprint =
        fingerprint_of(&composer).map_err(|error| format!("fingerprint: {error:?}"))?;
    assert!(matches(&fingerprint, &composer));
    let mut with_placeholder = composer.clone();
    with_placeholder.placeholder = Some("Ask".to_owned());
    assert!(!matches(&fingerprint, &with_placeholder));
    let other_role = facts("AXTextField", Some("Work with Codex"), None);
    assert!(!matches(&fingerprint, &other_role));
    let other_name = facts("AXTextArea", Some("Search"), None);
    assert!(!matches(&fingerprint, &other_name));
    let mut generated_id_now = composer.clone();
    generated_id_now.dom_id = Some("x-1a2b".to_owned());
    assert!(matches(&fingerprint, &generated_id_now));
    Ok(())
}
