//! One truthful vocabulary status for the selected model, derived from what the loaded model
//! declares and from the exact preparation of the next request: "used" only when at least one
//! whole term reaches the engine.
use super::{
    budget::ContextBudget,
    capability::ContextMechanism,
    context::ContextPreparation,
    occurrences::{eligible_terms, inspect, EligibleTerm},
    validation::TermIssue,
};
use crate::settings_service::errors::{ErrorReason, SettingsError, SettingsErrorCode};
use serde::Serialize;

/// The five statuses of the spec (AC-003).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum VocabularyStatus {
    UsedDuringRecognition,
    NoTermSent,
    NotSupported,
    NotUsedStreaming,
    UnknownUntilModelLoads,
}

/// Why a status was reached, where the status alone does not say it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum StatusReason {
    /// `no_term_sent`: the vocabulary is empty.
    EmptyVocabulary,
    /// `no_term_sent`: every saved term is invalid, a duplicate or over capacity.
    InvalidTermsOnly,
    /// `no_term_sent`: the first eligible term does not fit the budget.
    OverBudget,
    /// `unknown_until_model_loads`: the selected model is not loaded.
    ModelNotLoaded,
    /// `unknown_until_model_loads`: the model is loaded but the engine did not answer the
    /// capability query in time (for example during a dictation).
    CapabilityQueryUnanswered,
}

/// Recognition mode of the selected model's dictations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum RecognitionMode {
    Batch,
    Streaming,
}

/// Whether the report describes a prepared next request or a dispatched one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// The next request, prepared exactly as a dictation would; no run happened.
    Prepared,
}

/// Why a saved term does not reach the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    Empty,
    TooLong,
    LineBreak,
    ForbiddenCharacter,
    Duplicate,
    OverCapacity,
    /// Valid, but after the first term that did not fit the budget.
    OverBudget,
    /// Valid, but this model or mode receives no context at all.
    NoContext,
}

impl From<TermIssue> for ExclusionReason {
    fn from(issue: TermIssue) -> Self {
        match issue {
            TermIssue::Empty => Self::Empty,
            TermIssue::TooLong => Self::TooLong,
            TermIssue::LineBreak => Self::LineBreak,
            TermIssue::ForbiddenCharacter => Self::ForbiddenCharacter,
            TermIssue::Duplicate => Self::Duplicate,
            TermIssue::OverCapacity => Self::OverCapacity,
        }
    }
}

/// A saved term that is not sent, at its stored position.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub struct ExcludedTerm {
    pub index: u32,
    pub term: String,
    pub reason: ExclusionReason,
}

/// Status of the vocabulary for the selected model, with the terms sent and not sent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, specta::Type)]
pub struct VocabularyStatusReport {
    pub model_id: String,
    pub mode: RecognitionMode,
    pub status: VocabularyStatus,
    pub reason: Option<StatusReason>,
    pub mechanism: Option<ContextMechanism>,
    pub budget: Option<ContextBudget>,
    pub terms_sent: Vec<String>,
    pub terms_not_sent: Vec<ExcludedTerm>,
    pub evidence_kind: EvidenceKind,
}

/// What is known about the selected model's context support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadedContext {
    /// The selected model is not the loaded one.
    NotLoaded,
    /// The model is loaded but the engine did not answer the query in time.
    Unanswered,
    /// Loaded in an engine with no recognition-context API (the ONNX engines).
    NoContextApi,
    /// Loaded transcribe-cpp model and the exact preparation of the eligible terms.
    Prepared(ContextPreparation),
}

/// Inputs of the status of one selected model.
pub struct StatusInput<'a> {
    pub model_id: &'a str,
    /// The selected model's dictations produce their final text by streaming.
    pub streaming: bool,
    /// The saved vocabulary (custom words), as stored.
    pub stored: &'a [String],
    pub loaded: LoadedContext,
}

/// The selected model declares a context mechanism whose budget is not documented in this build
/// (`context_budget_unverified`): a technical error, never one of the five statuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextBudgetUnverified(pub ContextMechanism);

/// Durable-error rule: `internal_error` (exit 1) with `details.reason = context_budget_unverified`,
/// never `unavailable`, since no retry can clear it.
impl From<ContextBudgetUnverified> for SettingsError {
    fn from(_: ContextBudgetUnverified) -> Self {
        SettingsError::new(SettingsErrorCode::InternalError)
            .with_reason(ErrorReason::ContextBudgetUnverified)
    }
}

/// The status of `input`. Priority: unknown (not loaded or unanswered), streaming, no mechanism,
/// undocumented budget (error), then no term sent or used, from the exact fit.
///
/// # Errors
/// `ContextBudgetUnverified` for a batch model whose declared mechanism has no documented budget.
// @spec FR-002: Five truthful status outcomes — .specs/features/003-recognition-vocabulary/spec.md#fr-002
pub fn status_report(
    input: StatusInput<'_>,
) -> Result<VocabularyStatusReport, ContextBudgetUnverified> {
    let mut report = base_report(&input);
    let eligible = eligible_terms(input.stored);
    apply_loaded(&mut report, &input, &eligible)?;
    report.terms_not_sent.sort_by_key(|excluded| excluded.index);
    Ok(report)
}

/// The unknown status of `input`, with the stored terms that can never be sent.
fn base_report(input: &StatusInput<'_>) -> VocabularyStatusReport {
    VocabularyStatusReport {
        model_id: input.model_id.to_owned(),
        mode: if input.streaming {
            RecognitionMode::Streaming
        } else {
            RecognitionMode::Batch
        },
        status: VocabularyStatus::UnknownUntilModelLoads,
        reason: None,
        mechanism: None,
        budget: None,
        terms_sent: Vec::new(),
        terms_not_sent: legacy_exclusions(input.stored),
        evidence_kind: EvidenceKind::Prepared,
    }
}

fn apply_loaded(
    report: &mut VocabularyStatusReport,
    input: &StatusInput<'_>,
    eligible: &[EligibleTerm],
) -> Result<(), ContextBudgetUnverified> {
    match &input.loaded {
        LoadedContext::NotLoaded => report.reason = Some(StatusReason::ModelNotLoaded),
        LoadedContext::Unanswered => report.reason = Some(StatusReason::CapabilityQueryUnanswered),
        _ if input.streaming => no_context(report, eligible, VocabularyStatus::NotUsedStreaming),
        LoadedContext::NoContextApi | LoadedContext::Prepared(ContextPreparation::NoMechanism) => {
            no_context(report, eligible, VocabularyStatus::NotSupported);
        }
        LoadedContext::Prepared(ContextPreparation::Unverified(mechanism)) => {
            return Err(ContextBudgetUnverified(*mechanism));
        }
        LoadedContext::Prepared(ContextPreparation::Fitted(fitted)) => {
            report.mechanism = Some(fitted.mechanism);
            report.budget = Some(fitted.budget);
            fitted_terms(report, input.stored, eligible, fitted.sent);
        }
    }
    Ok(())
}

/// Stored occurrences that can never be sent, with their issue.
fn legacy_exclusions(stored: &[String]) -> Vec<ExcludedTerm> {
    inspect(stored)
        .into_iter()
        .filter_map(|occurrence| {
            occurrence.issue.map(|issue| ExcludedTerm {
                index: occurrence.index,
                term: occurrence.term,
                reason: issue.into(),
            })
        })
        .collect()
}

fn exclude(report: &mut VocabularyStatusReport, terms: &[EligibleTerm], reason: ExclusionReason) {
    report
        .terms_not_sent
        .extend(terms.iter().map(|term| ExcludedTerm {
            index: term.index,
            term: term.text.clone(),
            reason,
        }));
}

fn no_context(
    report: &mut VocabularyStatusReport,
    eligible: &[EligibleTerm],
    status: VocabularyStatus,
) {
    report.status = status;
    exclude(report, eligible, ExclusionReason::NoContext);
}

/// Business rule: "used" needs at least one whole term in the request; an empty or invalid-only
/// list, or a first term over budget, is "no term sent" with that reason.
fn fitted_terms(
    report: &mut VocabularyStatusReport,
    stored: &[String],
    eligible: &[EligibleTerm],
    sent: usize,
) {
    let sent = sent.min(eligible.len());
    if sent == 0 {
        report.status = VocabularyStatus::NoTermSent;
        report.reason = Some(if stored.is_empty() {
            StatusReason::EmptyVocabulary
        } else if eligible.is_empty() {
            StatusReason::InvalidTermsOnly
        } else {
            StatusReason::OverBudget
        });
    } else {
        report.status = VocabularyStatus::UsedDuringRecognition;
        report.terms_sent = eligible[..sent]
            .iter()
            .map(|term| term.text.clone())
            .collect();
    }
    exclude(report, &eligible[sent..], ExclusionReason::OverBudget);
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
