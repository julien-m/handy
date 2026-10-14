//! The settings window's replacement commits through the real settings service (an offline
//! store): the list Save and bulk Apply edits, their revision binding and their refusals.
use super::{
    check_word_replacements, draft_rules, fresh_ids, import_edit, save_edit, ReplacementCommit,
    ReplacementRulesView, RuleDraft,
};
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode},
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
fn commit_outcomes_are_tagged_values_for_the_editor() -> TestResult {
    let saved = ReplacementCommit::from(Ok(ReplacementRulesView {
        revision: 4,
        rules: Vec::new(),
    }));
    let saved = serde_json::to_value(saved)?;
    assert_eq!(
        saved,
        serde_json::json!({"status": "saved", "revision": 4, "rules": []})
    );
    let conflict = SettingsError::revision_conflict(Revision(5));
    let refused = serde_json::to_value(ReplacementCommit::from(Err(conflict.into())))?;
    assert_eq!(refused["status"], "refused");
    assert_eq!(refused["error"]["code"], "revision_conflict");
    assert_eq!(refused["error"]["current_revision"], 5);
    Ok(())
}
