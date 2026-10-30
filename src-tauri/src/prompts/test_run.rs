//! Prompt test of the settings editor (005 AC-007, FR-005): the unsaved draft (text and
//! override) runs once on an editable sample through the same AI step as a dictation, within
//! the same 30-second budget, and reports the result, provider, model and duration, or the
//! typed failure. It has no insertion or history dependency at all, so it can never insert text
//! or write history; one test runs at a time and a newer cancel invalidates a running one.
use super::library::{override_issue, text_issue};
use super::processor::{
    process, FailureReason, ProcessingOutcome, ProcessingReport, ProviderPort, ResolvedPrompt,
};
use super::provider::resolve_provider;
use super::types::PromptError;
use crate::settings::{AppSettings, PromptProviderOverride, APPLE_INTELLIGENCE_PROVIDER_ID};
use crate::settings_service::platform_capability::{PlatformCapabilities, PlatformCapability};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// The editor's draft and the sample to process.
#[derive(Clone, Debug, Deserialize, Type)]
pub struct PromptTestRequest {
    /// Prompt text as edited (not necessarily saved).
    pub text: String,
    /// Provider override as edited; `None` uses the default provider.
    pub provider_override: Option<PromptProviderOverride>,
    /// Sample text (prefilled by the UI with the last transcript, editable).
    pub sample: String,
}

/// How the test ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PromptTestStatus {
    Success,
    Failed,
    /// Another test is still running; this one did not start.
    Busy,
    Cancelled,
    /// The sample is empty: nothing to process.
    Skipped,
}

/// Result shown by the test panel; never a provider message or credential.
#[derive(Clone, Debug, Serialize, Type)]
pub struct PromptTestResult {
    pub status: PromptTestStatus,
    pub text: Option<String>,
    pub failure: Option<FailureReason>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub duration_ms: u64,
}

/// One test at a time; `cancel` invalidates the running one.
#[derive(Default)]
pub struct TestGate {
    busy: AtomicBool,
    generation: AtomicU64,
}
impl TestGate {
    /// An idle gate (usable in a `static`).
    pub const fn new() -> Self {
        Self {
            busy: AtomicBool::new(false),
            generation: AtomicU64::new(0),
        }
    }
    /// Cancel the running test, if any (its late result is reported `cancelled`).
    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }
}
/// Releases the gate when the test ends, however it ends (dropped future included).
struct Running<'a>(&'a TestGate);
impl Drop for Running<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::SeqCst);
    }
}

/// Refuse a draft the library would refuse: empty or too long text, incomplete or unknown
/// override, Apple Intelligence without the capability.
fn check_draft(
    settings: &AppSettings,
    request: &PromptTestRequest,
    platform: PlatformCapabilities,
) -> Result<(), PromptError> {
    if let Some(issue) = text_issue(&request.text) {
        return Err(PromptError::Invalid(issue));
    }
    let Some(choice) = &request.provider_override else {
        return Ok(());
    };
    let is_apple = choice.provider_id.trim() == APPLE_INTELLIGENCE_PROVIDER_ID;
    if is_apple && !platform.supports(PlatformCapability::AppleIntelligence) {
        return Err(PromptError::UnsupportedPlatform);
    }
    override_issue(settings, choice).map_or(Ok(()), |issue| Err(PromptError::Invalid(issue)))
}

// @spec FR-005: Test operation without insertion or history — .specs/features/005-custom-ai-prompts/spec.md#fr-005
/// Run the draft of `request` on its sample within `budget`.
///
/// # Errors
/// The draft's validation refusal (nothing runs).
pub async fn run_test(
    settings: &AppSettings,
    request: &PromptTestRequest,
    (port, budget, platform): (&dyn ProviderPort, Duration, PlatformCapabilities),
    gate: &TestGate,
) -> Result<PromptTestResult, PromptError> {
    check_draft(settings, request, platform)?;
    if gate.busy.swap(true, Ordering::SeqCst) {
        return Ok(result(PromptTestStatus::Busy, None, None));
    }
    let _running = Running(gate);
    let generation = gate.generation.load(Ordering::SeqCst);
    let prompt = ResolvedPrompt {
        id: String::new(),
        name: String::new(),
        text: request.text.clone(),
        choice: resolve_provider(settings, request.provider_override.as_ref()),
    };
    let cancelled = || gate.generation.load(Ordering::SeqCst) != generation;
    let report = process(&prompt, &request.sample, port, budget, &cancelled).await;
    Ok(test_result(report))
}

/// The panel's view of one AI step.
fn test_result(report: ProcessingReport) -> PromptTestResult {
    let (status, text, failure) = match report.outcome {
        ProcessingOutcome::Processed(text) => (PromptTestStatus::Success, Some(text), None),
        ProcessingOutcome::Failed(reason) => (PromptTestStatus::Failed, None, Some(reason)),
        ProcessingOutcome::Skipped => (PromptTestStatus::Skipped, None, None),
        ProcessingOutcome::Cancelled => (PromptTestStatus::Cancelled, None, None),
    };
    PromptTestResult {
        provider_id: report.provider_id,
        model: report.model,
        duration_ms: u64::try_from(report.duration.as_millis()).unwrap_or(u64::MAX),
        ..result(status, text, failure)
    }
}

fn result(
    status: PromptTestStatus,
    text: Option<String>,
    failure: Option<FailureReason>,
) -> PromptTestResult {
    PromptTestResult {
        status,
        text,
        failure,
        provider_id: None,
        model: None,
        duration_ms: 0,
    }
}

#[cfg(test)]
#[path = "test_run_tests.rs"]
mod tests;
