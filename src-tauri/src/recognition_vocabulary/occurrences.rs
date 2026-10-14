//! The stored list as it is: every occurrence keeps its bytes and position (legacy lists are never
//! normalized or pruned), and is flagged when it cannot reach the recognition context.
use super::validation::{term_key, validate_term, TermIssue, MAX_TERMS};
use serde::Serialize;
use std::collections::HashSet;

/// One stored occurrence, with the reason it is excluded from the recognition context, if any.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub struct Occurrence {
    /// Zero-based position in the stored list.
    pub index: u32,
    /// Stored text, unchanged.
    pub term: String,
    /// Why this occurrence is excluded from the context; `None` when it is eligible.
    pub issue: Option<TermIssue>,
}

/// A term the recognition context may use: its stored position and validated text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EligibleTerm {
    pub index: u32,
    pub text: String,
}

/// Inspect every stored occurrence: positions beyond `MAX_TERMS` are over capacity, invalid
/// terms keep their issue, and a later occurrence of an earlier key is a duplicate. The first
/// valid occurrence of each key stays eligible, in list order.
// @spec FR-001: Legacy terms kept and flagged — .specs/features/003-recognition-vocabulary/spec.md#fr-001
pub fn inspect(stored: &[String]) -> Vec<Occurrence> {
    let mut seen = HashSet::new();
    stored
        .iter()
        .enumerate()
        .map(|(index, term)| Occurrence {
            index: u32::try_from(index).unwrap_or(u32::MAX),
            term: term.clone(),
            issue: occurrence_issue(index, term, &mut seen),
        })
        .collect()
}

fn occurrence_issue(index: usize, term: &str, seen: &mut HashSet<String>) -> Option<TermIssue> {
    if index >= MAX_TERMS {
        return Some(TermIssue::OverCapacity);
    }
    match validate_term(term) {
        Err(issue) => Some(issue),
        Ok(valid) if !seen.insert(valid.key.clone()) => Some(TermIssue::Duplicate),
        Ok(_) => None,
    }
}

/// Terms the recognition context may use, in list order.
pub fn eligible_terms(stored: &[String]) -> Vec<EligibleTerm> {
    inspect(stored)
        .into_iter()
        .filter(|occurrence| occurrence.issue.is_none())
        .filter_map(|occurrence| {
            validate_term(&occurrence.term)
                .ok()
                .map(|valid| EligibleTerm {
                    index: occurrence.index,
                    text: valid.text,
                })
        })
        .collect()
}

/// Position of the first stored occurrence whose key equals the key of `raw`.
pub fn find_key(stored: &[String], raw: &str) -> Option<usize> {
    let key = term_key(raw);
    stored.iter().position(|term| term_key(term) == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn legacy_entries_are_flagged_without_being_changed() {
        let stored = list(&["Handy", "handy", "<bad>", "  spaced  ", ""]);
        let issues = inspect(&stored)
            .into_iter()
            .map(|occurrence| (occurrence.term, occurrence.issue))
            .collect::<Vec<_>>();
        assert_eq!(
            issues,
            vec![
                ("Handy".to_owned(), None),
                ("handy".to_owned(), Some(TermIssue::Duplicate)),
                ("<bad>".to_owned(), Some(TermIssue::ForbiddenCharacter)),
                ("  spaced  ".to_owned(), None),
                (String::new(), Some(TermIssue::Empty)),
            ]
        );
        let eligible = eligible_terms(&stored);
        assert_eq!(
            eligible,
            vec![
                EligibleTerm {
                    index: 0,
                    text: "Handy".into()
                },
                EligibleTerm {
                    index: 3,
                    text: "spaced".into()
                },
            ]
        );
    }

    #[test]
    fn positions_beyond_the_capacity_are_excluded_not_deleted() {
        let stored = (0..502)
            .map(|index| format!("term {index}"))
            .collect::<Vec<_>>();
        let occurrences = inspect(&stored);
        assert_eq!(occurrences.len(), 502);
        assert_eq!(occurrences[499].issue, None);
        assert_eq!(occurrences[500].issue, Some(TermIssue::OverCapacity));
        assert_eq!(eligible_terms(&stored).len(), 500);
    }

    #[test]
    fn keys_find_the_first_occurrence() {
        let stored = list(&["OpenClaw", "Claude Code", "claude  code"]);
        assert_eq!(find_key(&stored, "CLAUDE CODE"), Some(1));
        assert_eq!(find_key(&stored, "absent"), None);
    }
}
