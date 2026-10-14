//! The settings window's replacement commits through the real settings service (an offline
//! store): the list Save and bulk Apply edits, their revision binding and their refusals.
use super::{
    check_word_replacements, draft_rules, fresh_ids, import_edit, import_edit_at, preview_view,
    save_edit, RuleDraft,
};
use crate::settings_service::{
    command_error::SettingsCommandError,
    errors::{SettingsError, SettingsErrorCode, ValidationReason},
    live::LiveSettings,
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::SettingsRepository,
};
use crate::word_replacements::{
    model::ReplacementRule,
    preview::ImportMode,
    validation::{Field, IssueKind},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn offline(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = directory.path().join("settings_store.json");
    LiveSettings::start_offline(SettingsRepository::new(store), Box::new(probe))
}
fn draft(id: Option<&str>, recognized: &str, replacement: &str) -> RuleDraft {
    RuleDraft {
        id: id.map(Into::into),
        recognized: recognized.into(),
        replacement: replacement.into(),
    }
}
fn texts(rules: &[ReplacementRule]) -> Vec<(&str, &str)> {
    rules
        .iter()
        .map(|rule| (rule.recognized.as_str(), rule.replacement.as_str()))
        .collect()
}
fn saved(live: &LiveSettings) -> Vec<ReplacementRule> {
    live.snapshot().settings.word_replacements
}

#[test]
fn save_commits_the_whole_draft_once_and_keeps_existing_ids() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = offline(&directory)?;
    let first = [draft(None, "cloud code", "Claude Code")];
    let edit = save_edit(draft_rules(&first, &fresh_ids(&first)));
    let commit = live.commit_edit(&edit, Some(Revision(0)))?;
    let id = saved(&live)[0].id.clone();
    let second = [
        draft(Some(id.as_str()), " cloud code ", "Claude Code CLI"),
        draft(None, "get hub", "GitHub"),
    ];
    let edit = save_edit(draft_rules(&second, &fresh_ids(&second)));
    live.commit_edit(&edit, Some(commit.revision))?;
    let rules = saved(&live);
    let expected = [("cloud code", "Claude Code CLI"), ("get hub", "GitHub")];
    assert_eq!(
        (rules[0].id.clone(), texts(&rules)),
        (id, expected.to_vec())
    );
    assert_eq!(live.snapshot().revision, Revision(2));
    Ok(())
}

#[test]
fn save_refuses_an_invalid_draft_or_a_stale_revision_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = offline(&directory)?;
    let duplicate = [draft(None, "Same", "1"), draft(None, "same", "2")];
    let edit = save_edit(draft_rules(&duplicate, &fresh_ids(&duplicate)));
    let refused = live.commit_edit(&edit, Some(Revision(0))).err();
    assert_eq!(refused.map(|error| error.items().len()), Some(2));
    let valid = [draft(None, "a", "b")];
    let edit = save_edit(draft_rules(&valid, &fresh_ids(&valid)));
    let stale = live.commit_edit(&edit, Some(Revision(7))).err();
    let code = stale.map(|error| error.code());
    assert_eq!(code, Some(SettingsErrorCode::RevisionConflict));
    assert!(saved(&live).is_empty());
    Ok(())
}

#[test]
fn apply_commits_the_previewed_import_at_its_revision() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = offline(&directory)?;
    let input = "recognized;replacement\ncloud code;Claude Code\nget hub\tGitHub\n";
    let edit = import_edit(&saved(&live), input.into(), ImportMode::Replace)?;
    live.commit_edit(&edit, Some(Revision(0)))?;
    let expected = vec![("cloud code", "Claude Code"), ("get hub", "GitHub")];
    assert_eq!(texts(&saved(&live)), expected);
    let error_line = import_edit(&saved(&live), "a;1\nA;2\n".into(), ImportMode::Merge).err();
    let items = error_line.map(|error| error.items().len());
    assert_eq!(items, Some(2));
    let oversized = "#".repeat(1_000_001);
    let refused = import_edit(&[], oversized, ImportMode::Merge).err();
    let code = refused.map(|error| error.code());
    assert_eq!(code, Some(SettingsErrorCode::LimitExceeded));
    Ok(())
}

#[test]
fn check_reports_shared_validator_issues_of_a_draft() {
    let issues = check_word_replacements(vec![
        draft(None, "cloud code", ""),
        draft(None, "Cloud  Code", "x"),
    ]);
    let found: Vec<_> = issues
        .iter()
        .map(|issue| (issue.rule_index, issue.field, issue.kind))
        .collect();
    assert!(found.contains(&(Some(0), Field::Replacement, IssueKind::TooShort)));
    assert!(found.contains(&(Some(1), Field::Recognized, IssueKind::DuplicateKey)));
    assert!(check_word_replacements(vec![draft(None, "a", "b")]).is_empty());
}

#[test]
fn apply_refuses_a_preview_of_an_older_revision_before_building_the_edit() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = offline(&directory)?;
    let published = (live.snapshot().revision, live.snapshot().settings);
    let stale = import_edit_at(published, "a;b\n".into(), ImportMode::Merge, Revision(3)).err();
    let error = stale.map(SettingsCommandError::from);
    let found = error.map(|error| (error.code, error.current_revision));
    assert_eq!(found, Some((SettingsErrorCode::RevisionConflict, Some(0))));
    let published = (live.snapshot().revision, live.snapshot().settings);
    let edit = import_edit_at(published, "a;b\n".into(), ImportMode::Merge, Revision(0))?;
    live.commit_edit(&edit, Some(Revision(0)))?;
    assert_eq!(texts(&saved(&live)), vec![("a", "b")]);
    Ok(())
}

#[test]
fn refusals_reach_the_editor_as_the_shared_typed_settings_error() -> TestResult {
    let refused = import_edit(&[], "a;1\nA;2\n".into(), ImportMode::Merge).err();
    let error = refused.map(SettingsCommandError::from).ok_or("refusal")?;
    let expected = (
        SettingsErrorCode::ValidationFailed,
        Some("word_replacements".to_owned()),
        Some(ValidationReason::InvalidItems),
    );
    assert_eq!((error.code, error.field.clone(), error.reason), expected);
    let wire = serde_json::to_value(&error)?;
    assert_eq!(wire["code"], "validation_failed");
    assert_eq!(wire["reason"], "invalid_items");
    let conflict = SettingsCommandError::from(SettingsError::revision_conflict(Revision(5)));
    let wire = serde_json::to_value(conflict)?;
    assert_eq!(
        (&wire["code"], &wire["current_revision"]),
        (&"revision_conflict".into(), &5.into())
    );
    Ok(())
}

#[test]
fn preview_is_bounded_like_apply_and_counts_every_line() {
    let oversized = "#".repeat(1_000_001);
    let refused = preview_view(Revision(3), &[], &oversized, ImportMode::Merge).err();
    let code = refused.map(|error| error.code());
    assert_eq!(code, Some(SettingsErrorCode::LimitExceeded));
    let input = format!("{}a;1\n", "\n".repeat(999_990));
    let view = preview_view(Revision(3), &[], &input, ImportMode::Merge);
    let preview = view.map(|view| (view.revision, view.preview)).ok();
    let counts = preview.map(|(revision, preview)| {
        (
            revision,
            preview.ignored_lines,
            preview.lines.len(),
            preview.can_apply(),
        )
    });
    assert_eq!(counts, Some((3, 999_990, 1001, true)));
}

#[test]
fn the_ui_preview_lists_an_error_line_after_a_thousand_ignored_lines() {
    let input = format!("{}{}Cloud Opus\n", "\n".repeat(500), "# note\n".repeat(500));
    let view = preview_view(Revision(3), &[], &input, ImportMode::Merge);
    let preview = view.map(|view| view.preview).ok();
    let error_rows = preview.as_ref().map(|preview| {
        preview
            .lines
            .iter()
            .filter(|line| !line.issues.is_empty())
            .map(|line| {
                (
                    line.line_number,
                    line.issues.first().map(|issue| issue.kind),
                )
            })
            .collect::<Vec<_>>()
    });
    let expected = vec![(Some(1001), Some(IssueKind::MissingSeparator))];
    assert_eq!(error_rows, Some(expected));
    assert_eq!(preview.map(|preview| preview.can_apply()), Some(false));
}
