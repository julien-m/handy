//! Unit tests for the rule-set edits shared by the settings UI and the CLI domain.

use super::model::{ReplacementRule, RuleId};
use super::preview::{ImportMode, PreviewStatus};
use super::service::{
    add_rule, import_rules, new_rule_count, remove_rule, update_rule, PlannedImport,
    PreallocatedIds, RuleSetError,
};
use super::validation::{Field, IssueKind, RuleIssue, MAX_RULES};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rule(id: &str, recognized: &str, replacement: &str) -> ReplacementRule {
    ReplacementRule {
        id: id.into(),
        recognized: recognized.into(),
        replacement: replacement.into(),
    }
}

fn saved() -> Vec<ReplacementRule> {
    vec![
        rule("a", "cloud code", "Claude Code"),
        rule("b", "get hub", "GitHub"),
    ]
}

fn invalid_issues(result: Result<Vec<ReplacementRule>, RuleSetError>) -> Vec<RuleIssue> {
    match result {
        Err(RuleSetError::Invalid(issues)) => issues,
        _ => Vec::new(),
    }
}

#[test]
fn add_appends_a_trimmed_rule_with_the_given_identity() -> TestResult {
    let rules = add_rule(&saved(), "c".into(), "  type script ", " TypeScript ")
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(rules.len(), 3);
    assert_eq!(rules[2], rule("c", "type script", "TypeScript"));
    assert_eq!(&rules[..2], &saved()[..]);
    Ok(())
}

#[test]
fn add_refuses_a_folded_duplicate_key_for_both_participants() {
    let issues = invalid_issues(add_rule(&saved(), "c".into(), "CLOUD  Code", "x"));
    assert_eq!(
        issues,
        vec![
            RuleIssue {
                rule_index: Some(0),
                field: Field::Recognized,
                kind: IssueKind::DuplicateKey
            },
            RuleIssue {
                rule_index: Some(2),
                field: Field::Recognized,
                kind: IssueKind::DuplicateKey
            },
        ]
    );
}

#[test]
fn add_reports_field_issues_at_the_new_position() {
    let issues = invalid_issues(add_rule(&saved(), "c".into(), "", "a\tb"));
    assert_eq!(
        issues
            .iter()
            .map(|issue| (issue.rule_index, issue.field, issue.kind))
            .collect::<Vec<_>>(),
        vec![
            (Some(2), Field::Recognized, IssueKind::TooShort),
            (Some(2), Field::Replacement, IssueKind::ForbiddenCharacter),
        ]
    );
}

#[test]
fn add_refuses_the_rule_after_the_maximum() {
    let full: Vec<_> = (0..MAX_RULES)
        .map(|index| rule(&index.to_string(), &format!("key {index}"), "value"))
        .collect();
    let issues = invalid_issues(add_rule(&full, "new".into(), "one more", "value"));
    assert!(issues
        .iter()
        .any(|issue| issue.kind == IssueKind::TooManyRules));
}

#[test]
fn update_keeps_identity_and_position_and_may_change_the_key() -> TestResult {
    let rules = update_rule(&saved(), &"a".into(), "clod code", "Claude Code CLI")
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        rules,
        vec![
            rule("a", "clod code", "Claude Code CLI"),
            rule("b", "get hub", "GitHub")
        ]
    );
    Ok(())
}

#[test]
fn update_refuses_a_key_owned_by_another_rule_and_an_unknown_identity() {
    let issues = invalid_issues(update_rule(&saved(), &"a".into(), "Get Hub", "x"));
    assert!(issues
        .iter()
        .all(|issue| issue.kind == IssueKind::DuplicateKey));
    assert_eq!(issues.len(), 2);
    assert_eq!(
        update_rule(&saved(), &"missing".into(), "x", "y"),
        Err(RuleSetError::NotFound)
    );
}

#[test]
fn remove_keeps_the_order_of_the_others_and_refuses_an_unknown_identity() -> TestResult {
    let rules = remove_rule(&saved(), &"a".into()).map_err(|error| format!("{error:?}"))?;
    assert_eq!(rules, vec![rule("b", "get hub", "GitHub")]);
    assert_eq!(
        remove_rule(&saved(), &"missing".into()),
        Err(RuleSetError::NotFound)
    );
    Ok(())
}

#[test]
fn remove_repairs_a_damaged_set_one_rule_at_a_time() -> TestResult {
    let damaged = vec![rule("a", "same", "1"), rule("b", "SAME", "2")];
    let rules = remove_rule(&damaged, &"b".into()).map_err(|error| format!("{error:?}"))?;
    assert_eq!(rules, vec![rule("a", "same", "1")]);
    Ok(())
}

#[test]
fn import_merge_runs_twice_to_the_same_set_with_preallocated_ids() -> TestResult {
    let input = "recognized;replacement\nget hub;GitHub!\nnew one;New\n";
    let ids: Vec<RuleId> = vec!["fresh".into()];
    let first = import_rules(
        &saved(),
        input,
        ImportMode::Merge,
        &mut PreallocatedIds::new(&ids),
    )
    .map_err(|error| format!("{error:?}"))?;
    let second = import_rules(
        &saved(),
        input,
        ImportMode::Merge,
        &mut PreallocatedIds::new(&ids),
    )
    .map_err(|error| format!("{error:?}"))?;
    assert_eq!(first, second);
    assert_eq!(new_rule_count(&first.preview), 1);
    assert_eq!(
        first.rules,
        vec![
            rule("a", "cloud code", "Claude Code"),
            rule("b", "get hub", "GitHub!"),
            rule("fresh", "new one", "New"),
        ]
    );
    Ok(())
}

#[test]
fn import_replace_with_errors_returns_the_locating_preview() -> TestResult {
    let input = "a;1\nbad line\nA;2\n";
    let ids: Vec<RuleId> = Vec::new();
    let result = import_rules(
        &saved(),
        input,
        ImportMode::Replace,
        &mut PreallocatedIds::new(&ids),
    );
    let Err(RuleSetError::InvalidInput(preview)) = result else {
        return Err("an input with error lines must be refused with its preview".into());
    };
    let errors: Vec<_> = preview
        .lines
        .iter()
        .filter(|line| line.status == PreviewStatus::Error)
        .map(|line| line.line_number)
        .collect();
    assert_eq!(errors, vec![Some(1), Some(2), Some(3)]);
    Ok(())
}

#[test]
fn an_exhausted_identity_pool_is_refused_instead_of_inventing_an_identity() -> TestResult {
    let ids: Vec<RuleId> = Vec::new();
    let result = import_rules(
        &saved(),
        "new one;New\n",
        ImportMode::Merge,
        &mut PreallocatedIds::new(&ids),
    );
    let Err(RuleSetError::Invalid(issues)) = result else {
        return Err("a new rule without a preallocated identity must be refused".into());
    };
    assert!(issues.iter().any(|issue| issue.kind == IssueKind::EmptyId));
    Ok(())
}

#[test]
fn stable_names_match_the_serialized_forms() -> TestResult {
    let fields = [
        Field::Id,
        Field::Recognized,
        Field::Replacement,
        Field::Rules,
        Field::Line,
    ];
    for field in fields {
        assert_eq!(serde_json::to_value(field)?, field.as_str());
    }
    let kinds = [
        IssueKind::EmptyId,
        IssueKind::DuplicateId,
        IssueKind::TooShort,
        IssueKind::TooLong,
        IssueKind::ForbiddenCharacter,
        IssueKind::LeadingComment,
        IssueKind::DuplicateKey,
        IssueKind::TooManyRules,
        IssueKind::UnicodeDataUnavailable,
        IssueKind::MissingSeparator,
        IssueKind::InvalidSavedRules,
    ];
    for kind in kinds {
        assert_eq!(serde_json::to_value(kind)?, kind.as_str());
    }
    Ok(())
}

#[test]
fn a_planned_import_reuses_its_set_on_unchanged_rules_and_recomputes_otherwise() -> TestResult {
    let ids = [RuleId::from("n1")];
    let input = "type script;TypeScript\n".to_owned();
    let mut source = PreallocatedIds::new(&ids);
    let (planned, preview) = PlannedImport::new(&saved(), input, ImportMode::Merge, &mut source)
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(new_rule_count(&preview), 1);
    let unchanged = planned
        .apply(&saved())
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(unchanged[2], rule("n1", "type script", "TypeScript"));
    let moved = [rule("z", "zed", "Zed")];
    let recomputed = planned
        .apply(&moved)
        .map_err(|error| format!("{error:?}"))?;
    let expected = vec![
        rule("z", "zed", "Zed"),
        rule("n1", "type script", "TypeScript"),
    ];
    assert_eq!(recomputed, expected);
    Ok(())
}
