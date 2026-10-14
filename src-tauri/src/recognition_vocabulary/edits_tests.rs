//! Vocabulary edits against legacy and current lists: atomic refusals with indexed items, legacy
//! exemptions limited to unchanged occurrences, and remove-only changes always accepted.
use super::*;
use crate::settings_service::errors::ErrorReason;

type TestResult = Result<(), SettingsError>;

fn list(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
fn codes(error: &SettingsError) -> Vec<(u64, ErrorReason)> {
    error
        .items()
        .iter()
        .map(|item| (item.index(), item.code()))
        .collect()
}

#[test]
fn add_appends_one_valid_unique_term() -> TestResult {
    let next = VocabularyEdit::Add("  Claude   Code ".into()).apply_to(&list(&["OpenClaw"]))?;
    assert_eq!(next, list(&["OpenClaw", "Claude Code"]));
    Ok(())
}

#[test]
fn direct_duplicate_and_invalid_adds_fail_atomically() {
    let current = list(&["Claude Code"]);
    let duplicate = VocabularyEdit::Add("CLAUDE code".into()).apply_to(&current);
    let duplicate = duplicate.err().map(|error| (error.code(), codes(&error)));
    assert_eq!(
        duplicate,
        Some((
            SettingsErrorCode::ValidationFailed,
            vec![(0, ErrorReason::TermDuplicate)]
        ))
    );
    let too_long = VocabularyEdit::Add("x".repeat(101)).apply_to(&current);
    assert_eq!(
        too_long.err().map(|error| codes(&error)),
        Some(vec![(0, ErrorReason::TermTooLong)])
    );
}

#[test]
fn the_501st_term_is_a_limit_refusal() -> TestResult {
    let full = (0..500)
        .map(|index| format!("t{index}"))
        .collect::<Vec<_>>();
    let refused = VocabularyEdit::Add("one more".into()).apply_to(&full);
    assert_eq!(
        refused.err().map(|error| error.code()),
        Some(SettingsErrorCode::LimitExceeded)
    );
    let almost = &full[..499];
    assert_eq!(
        VocabularyEdit::Add("last".into()).apply_to(almost)?.len(),
        500
    );
    Ok(())
}

#[test]
fn remove_takes_the_first_occurrence_by_key_or_is_not_found() -> TestResult {
    let current = list(&["Handy", "OpenClaw", "handy"]);
    assert_eq!(
        VocabularyEdit::Remove("HANDY".into()).apply_to(&current)?,
        list(&["OpenClaw", "handy"])
    );
    let missing = VocabularyEdit::Remove("absent".into()).apply_to(&current);
    assert_eq!(
        missing.err().map(|e| e.code()),
        Some(SettingsErrorCode::NotFound)
    );
    Ok(())
}

#[test]
fn occurrence_removal_is_bound_to_the_exact_saved_row() -> TestResult {
    let current = list(&["Handy", "handy"]);
    let edit = VocabularyEdit::RemoveOccurrence {
        index: 1,
        term: "handy".into(),
    };
    assert_eq!(edit.apply_to(&current)?, list(&["Handy"]));
    let stale = VocabularyEdit::RemoveOccurrence {
        index: 1,
        term: "Handy".into(),
    };
    assert_eq!(
        stale.apply_to(&current).err().map(|e| e.code()),
        Some(SettingsErrorCode::NotFound)
    );
    Ok(())
}

#[test]
fn import_refuses_every_line_issue_and_never_saves_a_subset() {
    let edit = VocabularyEdit::Import {
        input: "ok\n<bad>\n\nfine\n".into(),
        mode: ImportMode::Merge,
    };
    let refused = edit.apply_to(&[]).err();
    assert_eq!(
        refused.as_ref().map(codes),
        Some(vec![
            (1, ErrorReason::TermForbiddenCharacter),
            (2, ErrorReason::TermEmpty)
        ])
    );
}

#[test]
fn ui_list_keeps_unchanged_legacy_rows_and_validates_new_ones() -> TestResult {
    let legacy = list(&["<legacy>", "dup", "DUP"]);
    // Adding a valid term keeps the invalid and duplicate legacy rows as they are.
    let next = list(&["<legacy>", "dup", "DUP", "New Term"]);
    assert_eq!(
        VocabularyEdit::Replace(next.clone()).apply_to(&legacy)?,
        next
    );
    // A new duplicate of a legacy key is refused at its index.
    let refused =
        VocabularyEdit::Replace(list(&["<legacy>", "dup", "DUP", "Dup"])).apply_to(&legacy);
    assert_eq!(
        refused.err().map(|error| codes(&error)),
        Some(vec![(3, ErrorReason::TermDuplicate)])
    );
    // An edited legacy row is a new term and must be valid.
    let edited = VocabularyEdit::Replace(list(&["<legacy2>", "dup", "DUP"])).apply_to(&legacy);
    assert_eq!(
        edited.err().map(|error| codes(&error)),
        Some(vec![(0, ErrorReason::TermForbiddenCharacter)])
    );
    Ok(())
}

#[test]
fn remove_only_changes_shrink_an_over_capacity_legacy_list() -> TestResult {
    let legacy = (0..503)
        .map(|index| format!("t{index}"))
        .collect::<Vec<_>>();
    let shrunk = legacy[1..].to_vec();
    assert_eq!(
        VocabularyEdit::Replace(shrunk.clone()).apply_to(&legacy)?,
        shrunk
    );
    let mut grown = legacy[1..].to_vec();
    grown.push("new".into());
    let refused = VocabularyEdit::Replace(grown).apply_to(&legacy);
    assert_eq!(
        refused.err().map(|e| e.code()),
        Some(SettingsErrorCode::LimitExceeded)
    );
    Ok(())
}
