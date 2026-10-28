//! The AI step through the production port (`NativeProviders` → `llm_client`) against a real
//! loopback HTTP server, never a real provider: an override routes the request to the local
//! endpoint with its model, an HTTP error falls back to the raw transcript, and a server slower
//! than the budget is abandoned with its connection dropped.
use super::native::NativeProviders;
use super::processor::{process, FailureReason, ProcessingOutcome, ResolvedPrompt};
use super::provider::resolve_provider;
use crate::settings::{get_default_settings, PromptProviderOverride};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;

/// One HTTP exchange: the response is written after `delay`; the request text is sent back.
async fn serve_once(
    status: &'static str,
    body: String,
    delay: Duration,
) -> (String, oneshot::Receiver<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.ok();
    let address = listener.as_ref().and_then(|l| l.local_addr().ok());
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let Some(listener) = listener else { return };
        let Ok((mut stream, _)) = listener.accept().await else {
            return;
        };
        let mut request = vec![0_u8; 16384];
        let read = stream.read(&mut request).await.unwrap_or(0);
        let _ = sender.send(String::from_utf8_lossy(&request[..read]).into_owned());
        tokio::time::sleep(delay).await;
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes()).await;
    });
    let base = address
        .map(|a| format!("http://{a}/v1"))
        .unwrap_or_default();
    (base, receiver)
}
/// "Notes" overridden to the local custom endpoint at `base_url` with model llama3.2.
fn local_prompt(base_url: &str) -> ResolvedPrompt {
    let mut settings = get_default_settings();
    if let Some(custom) = settings.post_process_provider_mut("custom") {
        custom.base_url = base_url.to_owned();
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
fn completion(content: &str) -> String {
    serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string()
}

#[tokio::test]
async fn an_override_sends_the_request_to_the_local_endpoint_with_its_model() {
    let (base, request) = serve_once("200 OK", completion("- buy milk"), Duration::ZERO).await;
    let never = || false;
    let report = process(
        &local_prompt(&base),
        "buy milk",
        &NativeProviders,
        Duration::from_secs(10),
        &never,
    )
    .await;
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Processed("- buy milk".into())
    );
    let request = request.await.unwrap_or_default();
    assert!(
        request.starts_with("POST /v1/chat/completions"),
        "{request}"
    );
    assert!(request.contains("\"model\":\"llama3.2\""));
    assert!(request.contains("Notes: buy milk"));
    assert!(
        !request.to_lowercase().contains("authorization:"),
        "the local endpoint gets no key"
    );
}
#[tokio::test]
async fn an_http_error_is_a_provider_error_without_its_body() {
    let (base, _request) = serve_once(
        "500 Internal Server Error",
        "PRIVATE BODY".into(),
        Duration::ZERO,
    )
    .await;
    let never = || false;
    let report = process(
        &local_prompt(&base),
        "words",
        &NativeProviders,
        Duration::from_secs(10),
        &never,
    )
    .await;
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Failed(FailureReason::ProviderError)
    );
    assert!(!format!("{report:?}").contains("PRIVATE BODY"));
}
#[tokio::test]
async fn a_server_slower_than_the_budget_is_abandoned() {
    let (base, _request) = serve_once("200 OK", completion("late"), Duration::from_secs(5)).await;
    let never = || false;
    let budget = Duration::from_millis(400);
    let report = process(
        &local_prompt(&base),
        "words",
        &NativeProviders,
        budget,
        &never,
    )
    .await;
    assert_eq!(
        report.outcome,
        ProcessingOutcome::Failed(FailureReason::Timeout)
    );
    assert!(report.duration < Duration::from_secs(3));
}
