//! Fault injection on the AI step with a fake provider port (no real provider, no key): every
//! failure class keeps the raw transcript (`Failed`), retries and the legacy fallback share one
//! budget, cancellation wins, a blank transcript is skipped, Apple availability and the busy
//! worker are reported. Budgets are scaled down (hundreds of milliseconds) with wide margins.
use super::*;
use crate::prompts::provider::resolve_provider;
use crate::settings::{get_default_settings, PromptProviderOverride};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Reply {
    Text(&'static str, u64),
    Nothing(u64),
    Error(u64),
}
struct FakePort {
    structured: Reply,
    legacy: Reply,
    apple: Result<&'static str, FailureReason>,
    apple_available: bool,
    calls: AtomicUsize,
}
impl FakePort {
    fn http(structured: Reply, legacy: Reply) -> Self {
        let apple = Ok("unused");
        Self {
            structured,
            legacy,
            apple,
            apple_available: false,
            calls: AtomicUsize::new(0),
        }
    }
}
impl ProviderPort for FakePort {
    fn chat<'a>(
        &'a self,
        request: ChatRequest<'a>,
    ) -> PortFuture<'a, Result<Option<String>, String>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let reply = if request.schema.is_some() {
            self.structured
        } else {
            self.legacy
        };
        Box::pin(async move {
            let (delay, result) = match reply {
                Reply::Text(text, delay) => (delay, Ok(Some(text.to_owned()))),
                Reply::Nothing(delay) => (delay, Ok(None)),
                Reply::Error(delay) => (delay, Err("HTTP 500".to_owned())),
            };
            tokio::time::sleep(Duration::from_millis(delay)).await;
            result
        })
    }
    fn apple<'a>(
        &'a self,
        _: String,
        _: String,
        _: i32,
    ) -> PortFuture<'a, Result<String, FailureReason>> {
        let result = self.apple.map(str::to_owned);
        Box::pin(async move { result })
    }
    fn apple_available(&self) -> bool {
        self.apple_available
    }
}

/// "Notes" on the local custom endpoint, with or without structured output.
fn prompt(structured: bool) -> ResolvedPrompt {
    let mut settings = get_default_settings();
    if let Some(custom) = settings.post_process_provider_mut("custom") {
        custom.supports_structured_output = structured;
    }
    let choice = PromptProviderOverride {
        provider_id: "custom".into(),
        model: "llama3.2".into(),
    };
    ResolvedPrompt {
        id: "starter_notes".into(),
        name: "Notes".into(),
        text: "Notes: ${output}".into(),
        choice: resolve_provider(&settings, Some(&choice)),
    }
}
const BUDGET: Duration = Duration::from_millis(500);
fn run(prompt: &ResolvedPrompt, port: &FakePort, raw: &str) -> ProcessingReport {
    let never = || false;
    tauri::async_runtime::block_on(process(prompt, raw, port, BUDGET, &never))
}
fn outcome(prompt: &ResolvedPrompt, port: &FakePort) -> ProcessingOutcome {
    run(prompt, port, "raw words").outcome
}

#[test]
fn a_structured_reply_is_processed_text() {
    let port = FakePort::http(
        Reply::Text(r#"{"transcription":"Clean."}"#, 0),
        Reply::Error(0),
    );
    let report = run(&prompt(true), &port, "raw words");
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Processed("Clean.".into())
    );
    assert_eq!(
        (report.provider_id.as_deref(), report.model.as_deref()),
        (Some("custom"), Some("llama3.2"))
    );
}
#[test]
fn a_structured_error_falls_back_to_the_legacy_request() {
    let port = FakePort::http(Reply::Error(0), Reply::Text("<think>x</think>Legacy.", 0));
    assert_eq!(
        outcome(&prompt(true), &port),
        ProcessingOutcome::Processed("Legacy.".into())
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 2);
}
// @spec AC-005: Every failure keeps the raw transcript — .specs/features/005-custom-ai-prompts/spec.md#ac-005
#[test]
fn every_failure_class_is_a_failure_with_its_reason() {
    let error = FakePort::http(Reply::Error(0), Reply::Error(0));
    assert_eq!(
        outcome(&prompt(false), &error),
        ProcessingOutcome::Failed(FailureReason::ProviderError)
    );
    for empty in [
        "",
        "   \n",
        "<think>only thinking</think>",
        "\u{200B}\u{FEFF}",
    ] {
        let port = FakePort::http(Reply::Error(0), Reply::Text(empty, 0));
        assert_eq!(
            outcome(&prompt(false), &port),
            ProcessingOutcome::Failed(FailureReason::EmptyResult)
        );
    }
    let nothing = FakePort::http(Reply::Nothing(0), Reply::Text("unused", 0));
    assert_eq!(
        outcome(&prompt(true), &nothing),
        ProcessingOutcome::Failed(FailureReason::EmptyResult)
    );
}
#[test]
fn a_prompt_that_is_not_ready_fails_without_any_request() {
    let mut not_ready = prompt(false);
    not_ready.choice = Err(ReadinessIssue::MissingApiKey);
    let port = FakePort::http(Reply::Text("unused", 0), Reply::Text("unused", 0));
    assert_eq!(
        outcome(&not_ready, &port),
        ProcessingOutcome::Failed(FailureReason::MissingApiKey)
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn a_reply_inside_the_budget_is_kept_and_one_past_it_is_abandoned() {
    let fast = FakePort::http(Reply::Error(0), Reply::Text("In time.", 100));
    assert_eq!(
        outcome(&prompt(false), &fast),
        ProcessingOutcome::Processed("In time.".into())
    );
    let slow = FakePort::http(Reply::Error(0), Reply::Text("Too late.", 3000));
    let report = run(&prompt(false), &slow, "raw words");
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Failed(FailureReason::Timeout)
    );
    assert!(
        report.duration < Duration::from_millis(2000),
        "abandoned at the budget"
    );
}
// @spec AC-005: One 30-second budget for the whole step — .specs/features/005-custom-ai-prompts/spec.md#ac-005
#[test]
fn the_structured_attempt_and_the_legacy_fallback_share_one_budget() {
    // Each request alone fits in the 500 ms budget; together they exceed it.
    let port = FakePort::http(Reply::Error(300), Reply::Text("Late fallback.", 300));
    let report = run(&prompt(true), &port, "raw words");
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Failed(FailureReason::Timeout)
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 2);
}
// @spec AC-005: Cancel inserts nothing — .specs/features/005-custom-ai-prompts/spec.md#ac-005
#[test]
fn cancellation_wins_over_a_pending_reply() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        flag.store(true, Ordering::SeqCst);
    });
    let port = FakePort::http(Reply::Error(0), Reply::Text("Never.", 400));
    let is_cancelled = move || cancelled.load(Ordering::SeqCst);
    let budget = Duration::from_secs(5);
    let report = tauri::async_runtime::block_on(process(
        &prompt(false),
        "raw",
        &port,
        budget,
        &is_cancelled,
    ));
    assert_eq!(report.outcome, ProcessingOutcome::Cancelled);
}
#[test]
fn a_blank_transcript_is_skipped_without_any_request() {
    let port = FakePort::http(Reply::Text("unused", 0), Reply::Text("unused", 0));
    assert_eq!(
        run(&prompt(false), &port, "  \n").outcome,
        ProcessingOutcome::Skipped
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}
/// "Notes" on Apple Intelligence (provider added as on Apple silicon builds).
fn apple_prompt() -> ResolvedPrompt {
    let mut settings = get_default_settings();
    settings
        .post_process_providers
        .push(crate::settings::PostProcessProvider {
            id: crate::settings::APPLE_INTELLIGENCE_PROVIDER_ID.into(),
            label: "Apple Intelligence".into(),
            base_url: "apple-intelligence://local".into(),
            allow_base_url_edit: false,
            models_endpoint: None,
            supports_structured_output: true,
        });
    let model = "Apple Intelligence".to_owned();
    let choice = PromptProviderOverride {
        provider_id: "apple_intelligence".into(),
        model,
    };
    let choice = resolve_provider(&settings, Some(&choice));
    ResolvedPrompt {
        choice,
        ..prompt(false)
    }
}
#[test]
fn apple_intelligence_reports_unavailable_busy_or_its_text() {
    let apple = apple_prompt();
    let mut port = FakePort::http(Reply::Error(0), Reply::Error(0));
    let unavailable = ProcessingOutcome::Failed(FailureReason::ProviderUnavailable);
    assert_eq!(outcome(&apple, &port), unavailable);
    port.apple_available = true;
    port.apple = Err(FailureReason::ProviderBusy);
    let busy = ProcessingOutcome::Failed(FailureReason::ProviderBusy);
    assert_eq!(outcome(&apple, &port), busy);
    port.apple = Ok("On device.");
    assert_eq!(
        outcome(&apple, &port),
        ProcessingOutcome::Processed("On device.".into())
    );
    assert_eq!(
        port.calls.load(Ordering::SeqCst),
        0,
        "no HTTP request for Apple"
    );
}
#[test]
fn failure_codes_are_stable_snake_case() {
    assert_eq!(FailureReason::Timeout.code(), "timeout");
    assert_eq!(
        FailureReason::from(ReadinessIssue::MissingApiKey).code(),
        "missing_api_key"
    );
}
#[test]
fn a_completed_operation_returns_its_output() {
    let result = tauri::async_runtime::block_on(complete_unless_cancelled(
        std::future::ready("done"),
        || false,
    ));
    assert_eq!(result, Some("done"));
}
#[test]
fn a_pending_operation_stops_after_cancellation() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    let cancel = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        flag.store(true, Ordering::Release);
    });
    let pending = std::future::pending::<()>();
    let result = tauri::async_runtime::block_on(complete_unless_cancelled(pending, || {
        cancelled.load(Ordering::Acquire)
    }));
    let _ = cancel.join();
    assert_eq!(result, None);
}
