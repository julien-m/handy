//! The prompt test on a fake provider port: result with provider, model and duration; typed
//! failure; draft validation before anything runs; one test at a time; cancel.
use super::*;
use crate::prompts::processor::{ChatRequest, PortFuture};
use crate::prompts::types::PromptIssue;
use crate::settings::get_default_settings;
use std::sync::atomic::AtomicUsize;

/// Answers every chat after `delay_ms` with `reply` (`None`: an HTTP error).
struct Port {
    reply: Option<&'static str>,
    delay_ms: u64,
    calls: AtomicUsize,
}
impl ProviderPort for Port {
    fn chat<'a>(&'a self, _: ChatRequest<'a>) -> PortFuture<'a, Result<Option<String>, String>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (reply, delay) = (self.reply, self.delay_ms);
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            reply
                .map(|text| Some(text.to_owned()))
                .ok_or_else(|| "HTTP 500".to_owned())
        })
    }
    fn apple<'a>(
        &'a self,
        _: String,
        _: String,
        _: i32,
    ) -> PortFuture<'a, Result<String, FailureReason>> {
        Box::pin(async { Err(FailureReason::ProviderUnavailable) })
    }
    fn apple_available(&self) -> bool {
        false
    }
}
fn port(reply: Option<&'static str>, delay_ms: u64) -> Port {
    Port {
        reply,
        delay_ms,
        calls: AtomicUsize::new(0),
    }
}
fn request(text: &str) -> PromptTestRequest {
    let local = PromptProviderOverride {
        provider_id: "custom".into(),
        model: "llama3.2".into(),
    };
    PromptTestRequest {
        text: text.into(),
        provider_override: Some(local),
        sample: "um so shorten this".into(),
    }
}
fn run(
    request: &PromptTestRequest,
    port: &Port,
    gate: &TestGate,
) -> Result<PromptTestResult, PromptError> {
    let settings = get_default_settings();
    let ports = (
        port as &dyn ProviderPort,
        Duration::from_secs(5),
        PlatformCapabilities::current(),
    );
    tauri::async_runtime::block_on(run_test(&settings, request, ports, gate))
}

#[test]
fn a_successful_test_reports_text_provider_model_and_duration() -> Result<(), PromptError> {
    let result = run(
        &request("Shorten: ${output}"),
        &port(Some("Shorter."), 20),
        &TestGate::default(),
    )?;
    assert_eq!(
        (result.status, result.text.as_deref()),
        (PromptTestStatus::Success, Some("Shorter."))
    );
    assert_eq!(
        (result.provider_id.as_deref(), result.model.as_deref()),
        (Some("custom"), Some("llama3.2"))
    );
    assert!(result.duration_ms >= 20);
    Ok(())
}
#[test]
fn a_provider_error_is_a_typed_failure() -> Result<(), PromptError> {
    let result = run(
        &request("Shorten: ${output}"),
        &port(None, 0),
        &TestGate::default(),
    )?;
    assert_eq!(
        (result.status, result.failure),
        (PromptTestStatus::Failed, Some(FailureReason::ProviderError))
    );
    assert_eq!(result.text, None);
    Ok(())
}
#[test]
fn an_invalid_draft_is_refused_before_any_request() {
    let idle = port(Some("unused"), 0);
    let blank = run(&request("   "), &idle, &TestGate::default());
    assert_eq!(
        blank.err(),
        Some(PromptError::Invalid(PromptIssue::TextEmpty))
    );
    let mut unknown = request("x");
    unknown.provider_override = Some(PromptProviderOverride {
        provider_id: "nope".into(),
        model: "m".into(),
    });
    let refused = run(&unknown, &idle, &TestGate::default());
    assert_eq!(
        refused.err(),
        Some(PromptError::Invalid(PromptIssue::UnknownProvider))
    );
    assert_eq!(idle.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn a_second_test_while_one_runs_is_busy_and_cancel_ends_the_first() {
    let gate = std::sync::Arc::new(TestGate::default());
    let first_gate = std::sync::Arc::clone(&gate);
    let first = std::thread::spawn(move || {
        run(
            &request("x ${output}"),
            &port(Some("late"), 600),
            &first_gate,
        )
    });
    std::thread::sleep(Duration::from_millis(150));
    let second = run(&request("x ${output}"), &port(Some("unused"), 0), &gate).map(|r| r.status);
    assert_eq!(second, Ok(PromptTestStatus::Busy));
    gate.cancel();
    let first = first
        .join()
        .map_err(|_| "panicked")
        .and_then(|r| r.map_err(|_| "refused"));
    assert_eq!(first.map(|r| r.status), Ok(PromptTestStatus::Cancelled));
    let after = run(&request("x ${output}"), &port(Some("again"), 0), &gate).map(|r| r.status);
    assert_eq!(after, Ok(PromptTestStatus::Success));
}
