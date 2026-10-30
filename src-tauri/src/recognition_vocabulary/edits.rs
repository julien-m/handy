//! Vocabulary mutations shared by the settings UI and the CLI. Each one computes the complete new
//! list from the latest stored list: the domain runs it once on the current settings to refuse
//! early, and the shared commit path runs it again on the locked latest settings, so a legacy
//! exemption can never be asserted by a caller.
use super::{
    occurrences::find_key,
    preview::{prepare_import, ImportMode, ImportPreview},
    validation::{term_key, validate_term, TermIssue, MAX_TERMS},
};
use crate::settings_service::{
    domains::{ChangeKind, DomainChange},
    errors::{ErrorItem, SettingsError, SettingsErrorCode},
};
use std::collections::{HashMap, HashSet};

/// One vocabulary mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VocabularyEdit {
    /// Append one valid term whose key is not saved yet.
    Add(String),
    /// Remove the first occurrence whose key equals the key of this text.
    Remove(String),
    /// Remove the occurrence at `index`, which must still hold exactly `term` (UI rows, legacy
    /// duplicates included).
    RemoveOccurrence { index: usize, term: String },
    /// Apply a bulk input as one atomic candidate.
    Import { input: String, mode: ImportMode },
    /// Save a whole list edited in the settings UI (unchanged legacy occurrences stay exempt).
    Replace(Vec<String>),
}

impl VocabularyEdit {
    /// The list this edit makes of `current`.
    ///
    /// # Errors
    /// `validation_failed` with `details.items` for refused terms, `limit_exceeded` beyond
    /// `MAX_TERMS`, `not_found` for a term or occurrence that is not saved.
    // @spec FR-001: One validated atomic vocabulary edit — .specs/features/003-recognition-vocabulary/spec.md#fr-001
    pub fn apply_to(&self, current: &[String]) -> Result<Vec<String>, SettingsError> {
        match self {
            Self::Add(raw) => add(current, raw),
            Self::Remove(raw) => {
                let index = find_key(current, raw).ok_or_else(not_found)?;
                Ok(without(current, index))
            }
            Self::RemoveOccurrence { index, term } => match current.get(*index) {
                Some(saved) if saved == term => Ok(without(current, *index)),
                _ => Err(not_found()),
            },
            Self::Import { input, mode } => {
                let preview = prepare_import(current, input, *mode);
                refuse_preview(&preview)?;
                Ok(preview.candidate)
            }
            Self::Replace(next) => replace(current, next),
        }
    }

    /// The typed change committed by the shared settings path (UI and CLI alike).
    pub fn into_change(self) -> DomainChange {
        DomainChange(ChangeKind::Apply(Box::new(move |settings| {
            settings.custom_words = self.apply_to(&settings.custom_words)?;
            Ok(())
        })))
    }
}

/// The refusal of a preview that cannot be applied, if any.
///
/// # Errors
/// `validation_failed` with the invalid lines as items, or `limit_exceeded`.
pub fn refuse_preview(preview: &ImportPreview) -> Result<(), SettingsError> {
    if !preview.issues.is_empty() {
        let items = preview
            .issues
            .iter()
            .map(|line| (line.index as usize, line.issue));
        return Err(refused(items));
    }
    if preview.over_capacity {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    Ok(())
}

fn add(current: &[String], raw: &str) -> Result<Vec<String>, SettingsError> {
    let valid = validate_term(raw).map_err(|issue| refused([(0, issue)]))?;
    if current.iter().any(|saved| term_key(saved) == valid.key) {
        return Err(refused([(0, TermIssue::Duplicate)]));
    }
    // Business rule: no addition while the list is full, legacy lists over `MAX_TERMS` included.
    if current.len() >= MAX_TERMS {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    let mut next = current.to_vec();
    next.push(valid.text);
    Ok(next)
}

/// A whole list from the UI. Strategy: a remove-only change (an ordered subsequence) is always
/// accepted, so a legacy list can shrink; otherwise every occurrence that matches a saved one
/// byte for byte (each saved occurrence used once) keeps its exemption, and every new term must
/// be valid, have a key no other entry has, and fit `MAX_TERMS`.
fn replace(current: &[String], next: &[String]) -> Result<Vec<String>, SettingsError> {
    if is_subsequence(next, current) {
        return Ok(next.to_vec());
    }
    let kept = kept_occurrences(current, next);
    let (result, items) = validate_new_terms(next, &kept);
    if !items.is_empty() {
        return Err(refused(items));
    }
    // A reordered legacy list over capacity keeps its exemption; adding to it does not.
    if kept.contains(&false) && result.len() > MAX_TERMS {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    Ok(result)
}

/// For each entry of `next`, whether it is an unchanged saved occurrence.
fn kept_occurrences(current: &[String], next: &[String]) -> Vec<bool> {
    let mut saved: HashMap<&str, usize> = HashMap::new();
    for term in current {
        *saved.entry(term.as_str()).or_default() += 1;
    }
    next.iter().map(|term| take(&mut saved, term)).collect()
}

/// The list with new terms validated, and the refused positions; a new term may not share its
/// key with any other entry, kept legacy occurrences included.
fn validate_new_terms(next: &[String], kept: &[bool]) -> (Vec<String>, Vec<(usize, TermIssue)>) {
    let entries = || next.iter().zip(kept);
    let mut keys = entries()
        .filter(|(_, kept)| **kept)
        .map(|(term, _)| term_key(term))
        .collect::<HashSet<_>>();
    let mut items = Vec::new();
    let mut result = Vec::with_capacity(next.len());
    for (index, (term, kept)) in entries().enumerate() {
        match (*kept, validate_term(term)) {
            (true, _) => result.push(term.clone()),
            (false, Err(issue)) => items.push((index, issue)),
            (false, Ok(valid)) if !keys.insert(valid.key.clone()) => {
                items.push((index, TermIssue::Duplicate));
            }
            (false, Ok(valid)) => result.push(valid.text),
        }
    }
    (result, items)
}

/// Use one saved occurrence of `term`, if any is left.
fn take(saved: &mut HashMap<&str, usize>, term: &str) -> bool {
    match saved.get_mut(term) {
        Some(count) if *count > 0 => {
            *count -= 1;
            true
        }
        _ => false,
    }
}

/// Whether `next` keeps an ordered subset of `current` (only removals).
fn is_subsequence(next: &[String], current: &[String]) -> bool {
    let mut remaining = current.iter();
    next.iter()
        .all(|term| remaining.any(|candidate| candidate == term))
}

fn without(current: &[String], index: usize) -> Vec<String> {
    let mut next = current.to_vec();
    next.remove(index);
    next
}

fn not_found() -> SettingsError {
    SettingsError::new(SettingsErrorCode::NotFound)
}

/// `validation_failed` with one item per refused position (never the term itself).
fn refused(items: impl IntoIterator<Item = (usize, TermIssue)>) -> SettingsError {
    let items = items.into_iter().filter_map(|(index, issue)| {
        issue
            .error_reason()
            .map(|reason| ErrorItem::new(index as u64, reason))
    });
    SettingsError::new(SettingsErrorCode::ValidationFailed).with_items(items)
}

#[cfg(test)]
#[path = "edits_tests.rs"]
mod tests;
