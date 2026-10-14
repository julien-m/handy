//! Deterministic replacement rules: validation, exchange, previews and matching.

pub mod dictation;
pub mod exchange;
pub mod exchange_file;
pub mod matcher;
pub mod materialize;
pub mod model;
pub mod preview;
pub mod published;
pub mod service;
pub mod transfer;
pub mod unicode;
pub mod validation;

#[cfg(test)]
mod exchange_tests;
#[cfg(test)]
mod matcher_script_tests;
#[cfg(test)]
mod matcher_tests;
#[cfg(test)]
mod preview_tests;
#[cfg(test)]
mod service_tests;

#[cfg(test)]
mod tests {
    use super::model::{ReplacementRule, RuleId};
    use super::validation::{validate_fields, validate_rules, Field, IssueKind};

    #[test]
    fn default_legacy_settings_to_empty_replacements() -> serde_json::Result<()> {
        let settings: crate::settings::AppSettings = serde_json::from_str("{}")?;
        assert!(settings.word_replacements.is_empty());
        assert!(crate::settings::AppSettings::default()
            .word_replacements
            .is_empty());
        Ok(())
    }

    #[test]
    fn reject_duplicate_folded_recognized_keys() {
        let rules = vec![
            ReplacementRule {
                id: RuleId::from("one"),
                recognized: "Straße".into(),
                replacement: "Street".into(),
            },
            ReplacementRule {
                id: RuleId::from("two"),
                recognized: "STRASSE".into(),
                replacement: "Road".into(),
            },
        ];
        assert!(validate_rules(&rules).is_err());
    }

    fn rule(index: usize, recognized: &str, replacement: &str) -> ReplacementRule {
        ReplacementRule {
            id: index.to_string().into(),
            recognized: recognized.into(),
            replacement: replacement.into(),
        }
    }

    #[test]
    fn accept_scalar_limits_and_reject_empty_or_one_over_maximum() {
        for count in [0, 1, 100, 101] {
            assert_eq!(
                validate_fields(&"𠀀".repeat(count), "x").is_ok(),
                (1..=100).contains(&count),
                "recognized scalar count {count}"
            );
        }
        for count in [0, 1, 500, 501] {
            assert_eq!(
                validate_fields("x", &"😀".repeat(count)).is_ok(),
                (1..=500).contains(&count),
                "replacement scalar count {count}"
            );
        }
        assert!(validate_fields(&"ß".repeat(100), "a").is_ok());
    }

    #[test]
    fn trim_only_edges_and_preserve_valid_inner_replacement_bytes() {
        assert_eq!(
            validate_fields("\u{2003}Cloud  Code\u{A0}", "  Claude  Code; CLI  "),
            Ok(("Cloud  Code".into(), "Claude  Code; CLI".into()))
        );
        assert!(validate_fields("\u{200B}", "#;replacement").is_ok());
        assert!(validate_fields(" \u{A0} ", "replacement").is_err());
    }

    #[test]
    fn reject_forbidden_characters_and_leading_comments() {
        for value in ["a\rb", "a\nb", "a\tb", "a;b", " #comment"] {
            assert!(validate_fields(value, "good").is_err(), "{value:?}");
        }
        for value in ["a\rb", "a\nb", "a\tb"] {
            assert!(validate_fields("good", value).is_err(), "{value:?}");
        }
        assert!(validate_fields("a#b", "#a;b").is_ok());
    }

    #[test]
    fn reject_unicode_hard_line_breaks_in_both_fields() {
        for line_break in [
            '\r', '\n', '\u{000B}', '\u{000C}', '\u{0085}', '\u{2028}', '\u{2029}',
        ] {
            let broken = format!("a{line_break}b");
            for (recognized, replacement) in [(&broken[..], "valid"), ("valid", &broken[..])] {
                let issues = validate_fields(recognized, replacement).unwrap_err();
                assert!(
                    issues
                        .iter()
                        .any(|issue| issue.kind == IssueKind::ForbiddenCharacter),
                    "{line_break:?}"
                );
            }
        }
    }

    #[test]
    fn report_all_duplicate_participants_without_mutating_input() {
        let rules = vec![
            rule(1, " Straße ", "one"),
            rule(2, "STRASSE", "two"),
            rule(3, "strasse", "three"),
            rule(4, " cloud\u{A0}  code ", "four"),
            rule(5, "CLOUD CODE", "five"),
        ];
        let original = rules.clone();
        let issues = validate_rules(&rules).unwrap_err();
        assert_eq!(
            issues
                .iter()
                .filter(|issue| issue.kind == IssueKind::DuplicateKey)
                .map(|issue| issue.rule_index)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), Some(2), Some(3), Some(4)]
        );
        assert_eq!(rules, original);
    }

    #[test]
    fn preserve_accent_and_canonical_form_distinctions() {
        assert!(validate_rules(&[
            rule(1, "cafe", "one"),
            rule(2, "café", "two"),
            rule(3, "cafe\u{301}", "three")
        ])
        .is_ok());
    }

    #[test]
    fn accept_empty_and_maximum_sets_reject_one_more() {
        assert_eq!(validate_rules(&[]), Ok(Vec::new()));
        let rules: Vec<_> = (0..1001)
            .map(|index| rule(index, &format!("key {index}"), "value"))
            .collect();
        assert_eq!(
            validate_rules(&rules[..1000]).map(|rules| rules.len()),
            Ok(1000)
        );
        assert!(validate_rules(&rules)
            .unwrap_err()
            .iter()
            .any(|issue| issue.kind == IssueKind::TooManyRules && issue.rule_index.is_none()));
    }

    #[test]
    fn preserve_stable_identity_and_order_after_trimming() {
        let rules = vec![rule(1, " second ", " two "), rule(2, " first ", " one ")];
        assert_eq!(
            validate_rules(&rules),
            Ok(vec![rule(1, "second", "two"), rule(2, "first", "one")])
        );
    }

    #[test]
    fn validate_opaque_id_rejects_empty_without_changing_input() {
        let mut invalid = rule(1, "recognized", "replacement");
        invalid.id = "".into();
        let original = vec![invalid];
        assert_eq!(
            validate_rules(&original),
            Err(vec![super::validation::RuleIssue {
                rule_index: Some(0),
                field: Field::Id,
                kind: IssueKind::EmptyId
            }])
        );
        assert_eq!(original[0].id.as_str(), "");
    }

    #[test]
    fn validate_opaque_id_rejects_all_duplicate_participants(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = vec![
            rule(1, "first", "one"),
            rule(2, "second", "two"),
            rule(3, "third", "three"),
        ];
        for entry in &mut rules {
            entry.id = "same opaque id".into();
        }
        let original = rules.clone();
        let issues = validate_rules(&rules)
            .err()
            .ok_or("duplicate IDs unexpectedly accepted")?;
        assert!(issues
            .iter()
            .all(|issue| issue.field == Field::Id && issue.kind == IssueKind::DuplicateId));
        assert_eq!(
            issues
                .iter()
                .map(|issue| issue.rule_index)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), Some(2)]
        );
        assert_eq!(rules, original);
        Ok(())
    }

    #[test]
    fn preserve_opaque_id_bytes_without_format_length_or_trim_limits() {
        let mut rules = vec![
            rule(1, "first", "one"),
            rule(2, "second", "two"),
            rule(3, "third", "three"),
        ];
        rules[0].id = " ".into();
        rules[1].id = "  ".into();
        rules[2].id = "opaque".repeat(200).into();
        assert_eq!(validate_rules(&rules), Ok(rules.clone()));
    }
}
