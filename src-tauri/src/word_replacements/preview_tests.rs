//! Unit tests for merge/replace previews and commit-time ID materialization.

use super::exchange::serialize_exchange;
use super::model::{ReplacementRule, RuleId, RuleIdSource};
use super::preview::{
    materialize_candidate, preview_exchange, ImportMode, PreviewRejected, PreviewStatus,
    ReplacementPreview, MAX_DUPLICATE_LINES, MAX_LISTED_OTHER_LINES,
};
use super::validation::{IssueKind, MAX_RULES};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct SequentialIds(usize);

impl RuleIdSource for SequentialIds {
    fn next_id(&mut self) -> RuleId {
        self.0 += 1;
        RuleId::from(format!("new-{}", self.0))
    }
}

fn saved(id: &str, recognized: &str, replacement: &str) -> ReplacementRule {
    ReplacementRule {
        id: id.into(),
        recognized: recognized.into(),
        replacement: replacement.into(),
    }
}

fn statuses(preview: &ReplacementPreview) -> Vec<(Option<usize>, PreviewStatus)> {
    preview
        .lines
        .iter()
        .map(|line| (line.line_number, line.status))
        .collect()
}

fn content(rules: &[ReplacementRule]) -> Vec<(&str, &str)> {
    rules
        .iter()
        .map(|rule| (rule.recognized.as_str(), rule.replacement.as_str()))
        .collect()
}

fn ids(rules: &[ReplacementRule]) -> Vec<&str> {
    rules.iter().map(|rule| rule.id.as_str()).collect()
}

#[test]
fn preview_pasted_block_with_header_as_two_additions() -> TestResult {
    let preview = preview_exchange(
        &[],
        "texte_reconnu;remplacement\nCloud Code;Claude Code\nCloud Sonnet;Claude Sonnet",
        ImportMode::Replace,
    );
    assert_eq!(
        statuses(&preview),
        vec![
            (Some(1), PreviewStatus::Ignored),
            (Some(2), PreviewStatus::Add),
            (Some(3), PreviewStatus::Add)
        ]
    );
    let rules = materialize_candidate(&preview, &mut SequentialIds(0))?;
    assert_eq!(
        content(&rules),
        vec![
            ("Cloud Code", "Claude Code"),
            ("Cloud Sonnet", "Claude Sonnet")
        ]
    );
    Ok(())
}

#[test]
fn invalid_line_blocks_apply_and_reports_missing_separator() {
    let preview = preview_exchange(
        &[],
        "Cloud Code;Claude Code\nCloud Sonnet;Claude Sonnet\nCloud Opus",
        ImportMode::Replace,
    );
    let line = &preview.lines[2];
    assert_eq!(
        (line.line_number, line.status),
        (Some(3), PreviewStatus::Error)
    );
    assert_eq!(line.issues[0].kind, IssueKind::MissingSeparator);
    assert!(!preview.can_apply());
    assert_eq!(
        materialize_candidate(&preview, &mut SequentialIds(0)),
        Err(PreviewRejected::HasErrors)
    );
}

#[test]
fn report_both_duplicate_lines_in_one_block() {
    let preview = preview_exchange(
        &[],
        "Cloud Code;Claude Code\ncloud  code;Claude Code CLI",
        ImportMode::Merge,
    );
    assert_eq!(
        statuses(&preview),
        vec![
            (Some(1), PreviewStatus::Error),
            (Some(2), PreviewStatus::Error)
        ]
    );
    assert_eq!(preview.lines[0].duplicate_lines, vec![2]);
    assert_eq!(preview.lines[1].duplicate_lines, vec![1]);
    assert!(preview
        .lines
        .iter()
        .all(|line| line.issues[0].kind == IssueKind::DuplicateKey));
    assert!(!preview.can_apply());
}

#[test]
fn merge_updates_in_place_and_appends_in_file_order() -> TestResult {
    let base = vec![
        saved("a", "Cloud Code", "Claude Code"),
        saved("b", "Other", "Thing"),
    ];
    let input = "cloud code;Claude Code CLI\nOpen Claw;OpenClaw\nother;Thing";
    let preview = preview_exchange(&base, input, ImportMode::Merge);
    assert_eq!(
        statuses(&preview),
        vec![
            (Some(1), PreviewStatus::Update),
            (Some(2), PreviewStatus::Add),
            (Some(3), PreviewStatus::Unchanged)
        ]
    );
    let rules = materialize_candidate(&preview, &mut SequentialIds(0))?;
    let expected = [
        ("Cloud Code", "Claude Code CLI"),
        ("Other", "Thing"),
        ("Open Claw", "OpenClaw"),
    ];
    assert_eq!(content(&rules), expected);
    assert_eq!(ids(&rules), vec!["a", "b", "new-1"]);
    Ok(())
}

#[test]
fn replace_uses_input_order_reuses_ids_and_lists_deletions() -> TestResult {
    let base = vec![
        saved("a", "Cloud Code", "Claude Code"),
        saved("b", "Gone", "Away"),
        saved("c", "Kept", "Same"),
    ];
    let input = "Kept;Same\nCLOUD CODE;Claude Code\nNew;Rule";
    let preview = preview_exchange(&base, input, ImportMode::Replace);
    let expected_statuses = vec![
        (Some(1), PreviewStatus::Unchanged),
        (Some(2), PreviewStatus::Update),
        (Some(3), PreviewStatus::Add),
        (None, PreviewStatus::Delete),
    ];
    assert_eq!(statuses(&preview), expected_statuses);
    assert_eq!(preview.lines[3].rule_id, Some(RuleId::from("b")));
    let rules = materialize_candidate(&preview, &mut SequentialIds(0))?;
    assert_eq!(ids(&rules), vec!["c", "a", "new-1"]);
    assert_eq!(rules[1].recognized, "CLOUD CODE");
    Ok(())
}

#[test]
fn exported_set_previews_as_noop_replace() {
    let base = vec![saved("a", "One", "1"), saved("b", "Two", "2;3")];
    let preview = preview_exchange(&base, &serialize_exchange(&base), ImportMode::Replace);
    assert!(preview.is_noop && preview.can_apply());
    assert_eq!(
        materialize_candidate(&preview, &mut SequentialIds(0)),
        Ok(base)
    );
}

#[test]
fn reordered_replace_is_not_a_noop() {
    let base = vec![saved("a", "One", "1"), saved("b", "Two", "2")];
    let preview = preview_exchange(&base, "Two;2\nOne;1", ImportMode::Replace);
    assert!(!preview.is_noop);
    assert!(preview
        .lines
        .iter()
        .all(|line| line.status == PreviewStatus::Unchanged));
}

#[test]
fn refuse_candidates_above_the_rule_limit() {
    let base: Vec<_> = (0..MAX_RULES)
        .map(|index| saved(&index.to_string(), &format!("key {index}"), "v"))
        .collect();
    let preview = preview_exchange(&base, "one more;v", ImportMode::Merge);
    assert_eq!(preview.set_issues, vec![IssueKind::TooManyRules]);
    assert!(!preview.can_apply());
}

#[test]
fn refuse_apply_against_an_invalid_saved_base() {
    let base = vec![saved("a", "Same", "1"), saved("b", "same", "2")];
    let preview = preview_exchange(&base, "Other;3", ImportMode::Merge);
    assert_eq!(preview.set_issues, vec![IssueKind::InvalidSavedRules]);
    assert!(!preview.can_apply());
}

#[test]
fn reject_colliding_allocated_ids_at_materialization() {
    struct ConstantIds;
    impl RuleIdSource for ConstantIds {
        fn next_id(&mut self) -> RuleId {
            RuleId::from("same")
        }
    }
    let preview = preview_exchange(&[], "A;1\nB;2", ImportMode::Replace);
    assert!(matches!(
        materialize_candidate(&preview, &mut ConstantIds),
        Err(PreviewRejected::Invalid(_))
    ));
}

#[test]
fn hash_exact_input_and_keep_saved_rules_untouched() {
    let base = vec![saved("a", "One", "1")];
    let before = base.clone();
    let first = preview_exchange(&base, "One;2", ImportMode::Merge);
    let second = preview_exchange(&base, "One;2\n", ImportMode::Merge);
    assert_ne!(first.input_sha256, second.input_sha256);
    assert_eq!(first.input_sha256.len(), 64);
    assert_eq!(base, before);
}

#[test]
fn a_million_blank_or_error_lines_list_a_bounded_number_of_rows() {
    let blank = "\n".repeat(999_999);
    let preview = preview_exchange(&[], &blank, ImportMode::Merge);
    let counts = (
        preview.lines.len(),
        preview.ignored_lines,
        preview.omitted_lines,
    );
    assert_eq!(counts, (MAX_LISTED_OTHER_LINES, 999_999, 999_999 - 1000));
    assert!(preview.can_apply() && preview.candidate.is_empty());
    let errors = "x\n".repeat(499_999);
    let preview = preview_exchange(&[], &errors, ImportMode::Replace);
    let counts = (
        preview.lines.len(),
        preview.error_lines,
        preview.omitted_lines,
    );
    assert_eq!(counts, (MAX_LISTED_OTHER_LINES, 499_999, 499_999 - 1000));
    assert!(!preview.can_apply());
}

#[test]
fn rule_lines_past_the_rule_limit_are_counted_and_refused() {
    let input: String = (0..MAX_RULES + 500)
        .map(|index| format!("r{index};x\n"))
        .collect();
    let preview = preview_exchange(&[], &input, ImportMode::Replace);
    assert_eq!(
        (preview.lines.len(), preview.omitted_lines),
        (MAX_RULES, 500)
    );
    assert_eq!(preview.set_issues, vec![IssueKind::TooManyRules]);
    assert!(!preview.can_apply());
}

#[test]
fn a_key_repeated_on_every_line_names_a_bounded_number_of_other_lines() {
    let preview = preview_exchange(&[], &"same;x\n".repeat(MAX_RULES), ImportMode::Merge);
    assert_eq!(preview.error_lines, MAX_RULES);
    assert!(preview
        .lines
        .iter()
        .all(|line| line.duplicate_lines.len() == MAX_DUPLICATE_LINES));
    assert_eq!(preview.lines[0].duplicate_lines[0], 2);
}
