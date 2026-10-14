//! The AI step of a dictation or of a prompt test (005 FR-003, FR-004, FR-005): one frozen
//! prompt and provider, one 30-second budget for the whole step (Apple availability check,
//! structured request, its retry without reasoning fields, the legacy fallback and parsing all
//! share it), cancellation, and a typed outcome. On any failure the caller keeps the raw
//! transcript; this module never inserts text or writes history.
use super::formatting;
use super::provider::{is_apple, ProviderChoice, ReadinessIssue};
use crate::settings::PostProcessProvider;
use serde::Serialize;
use specta::Type;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Budget of the whole AI step (AC-005): past it, the step is abandoned and the raw transcript
/// is used.
pub const PROCESSING_BUDGET: Duration = Duration::from_secs(30);
/// How often a running step checks for cancellation (same cadence as the dictation pipeline).
const CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(25);

/// A boxed future of a provider port.
pub type PortFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Why the AI step produced no usable text. Codes are stored in history and shown translated;
/// they never carry a provider message, a URL or a credential.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    /// The provider or the HTTP request failed.
    ProviderError,
    /// The reply was empty or only whitespace once cleaned.
    EmptyResult,
    /// The whole step exceeded `PROCESSING_BUDGET`.
    Timeout,
    /// The previous Apple Intelligence request is still running natively.
    ProviderBusy,
    MissingEndpoint,
    MissingModel,
    MissingApiKey,
    ProviderUnavailable,
    UnknownProvider,
    InvalidPrompt,
    PromptMissing,
}
impl From<ReadinessIssue> for FailureReason {
    fn from(issue: ReadinessIssue) -> Self {
        match issue {
            ReadinessIssue::MissingEndpoint => Self::MissingEndpoint,
            ReadinessIssue::MissingModel => Self::MissingModel,
            ReadinessIssue::MissingApiKey => Self::MissingApiKey,
            ReadinessIssue::ProviderUnavailable => Self::ProviderUnavailable,
            ReadinessIssue::UnknownProvider => Self::UnknownProvider,
            ReadinessIssue::InvalidPrompt => Self::InvalidPrompt,
            ReadinessIssue::PromptMissing => Self::PromptMissing,
        }
    }
}
impl FailureReason {
    /// Stable snake_case code (history column, events).
    pub fn code(self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "provider_error".to_owned())
    }
}

/// A prompt frozen for one step: its text and the provider resolved from one settings view (or
/// why none could be).
#[derive(Clone, Debug)]
pub struct ResolvedPrompt {
    pub id: String,
    pub name: String,
    pub text: String,
    pub choice: Result<ProviderChoice, ReadinessIssue>,
}

/// What the AI step produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessingOutcome {
    /// Usable processed text.
    Processed(String),
    /// Nothing to process (empty transcript): the raw transcript is used, no failure.
    Skipped,
    /// No usable text: the raw transcript is used and the failure recorded.
    Failed(FailureReason),
    /// Cancelled by the user: nothing is inserted.
    Cancelled,
}

/// Outcome of one step with the provider and model it used and how long it took.
#[derive(Clone, Debug)]
pub struct ProcessingReport {
    pub outcome: ProcessingOutcome,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub duration: Duration,
}

/// One chat completion request (OpenAI-compatible endpoints).
pub struct ChatRequest<'a> {
    pub provider: &'a PostProcessProvider,
    pub api_key: &'a str,
    pub model: &'a str,
    pub user: String,
    pub system: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub disable_reasoning: bool,
}

/// The providers the AI step talks to; production is `native::NativeProviders`, tests inject
/// fakes. Every call is dropped (and its HTTP request cancelled) when the budget expires.
pub trait ProviderPort: Send + Sync {
    /// Send one chat completion; `Ok(None)` when the reply has no content.
    fn chat<'a>(
        &'a self,
        request: ChatRequest<'a>,
    ) -> PortFuture<'a, Result<Option<String>, String>>;
    /// Run Apple Intelligence on `system` and `user` with `token_limit` (0 = default).
    fn apple<'a>(
        &'a self,
        system: String,
        user: String,
        token_limit: i32,
    ) -> PortFuture<'a, Result<String, FailureReason>>;
    /// Whether Apple Intelligence can run now (synchronous native check).
    fn apple_available(&self) -> bool;
}

/// Run `operation` until it completes or `is_cancelled` turns true (polled every 25 ms).
pub async fn complete_unless_cancelled<F, C>(operation: F, is_cancelled: C) -> Option<F::Output>
where
    F: Future,
    C: Fn() -> bool,
{
    tokio::pin!(operation);
    loop {
        if is_cancelled() {
            return None;
        }
        if let Ok(result) =
            tokio::time::timeout(CANCELLATION_POLL_INTERVAL, operation.as_mut()).await
        {
            return Some(result);
        }
    }
}

// @spec FR-003: Whole AI step bounded to 30 seconds — .specs/features/005-custom-ai-prompts/spec.md#fr-003
// @spec FR-004: Failure and cancellation outcomes — .specs/features/005-custom-ai-prompts/spec.md#fr-004
/// Process `raw` with `prompt` through `port` within `budget` (production: `PROCESSING_BUDGET`),
/// unless `is_cancelled` turns true first. The budget starts before any external work.
pub async fn process(
    prompt: &ResolvedPrompt,
    raw: &str,
    port: &dyn ProviderPort,
    budget: Duration,
    is_cancelled: &(dyn Fn() -> bool + Sync),
) -> ProcessingReport {
    let started = Instant::now();
    let outcome = if raw.trim().is_empty() {
        ProcessingOutcome::Skipped
    } else {
        let work = complete_unless_cancelled(attempt(prompt, raw, port), is_cancelled);
        match tokio::time::timeout(budget, work).await {
            Err(_) => ProcessingOutcome::Failed(FailureReason::Timeout),
            Ok(None) => ProcessingOutcome::Cancelled,
            Ok(Some(Ok(text))) => ProcessingOutcome::Processed(text),
            Ok(Some(Err(reason))) => ProcessingOutcome::Failed(reason),
        }
    };
    let choice = prompt.choice.as_ref().ok();
    ProcessingReport {
        outcome,
        provider_id: choice.map(|choice| choice.provider.id.clone()),
        model: choice.map(|choice| choice.model.clone()),
        duration: started.elapsed(),
    }
}

/// One attempt chain: readiness, then Apple Intelligence or HTTP (structured first when the
/// provider supports it, legacy otherwise or after a structured error). A reply empty once
/// cleaned is a failure.
async fn attempt(
    prompt: &ResolvedPrompt,
    raw: &str,
    port: &dyn ProviderPort,
) -> Result<String, FailureReason> {
    let choice = prompt
        .choice
        .as_ref()
        .map_err(|issue| FailureReason::from(*issue))?;
    let text = if is_apple(choice) {
        apple(prompt, raw, choice, port).await?
    } else {
        http(prompt, raw, choice, port).await?
    };
    if text.trim().is_empty() {
        return Err(FailureReason::EmptyResult);
    }
    Ok(text)
}

/// Apple Intelligence: availability first, then the bounded native worker.
async fn apple(
    prompt: &ResolvedPrompt,
    raw: &str,
    choice: &ProviderChoice,
    port: &dyn ProviderPort,
) -> Result<String, FailureReason> {
    if !port.apple_available() {
        return Err(FailureReason::ProviderUnavailable);
    }
    // The Apple model setting holds an optional token limit; anything else means the default.
    let token_limit = choice.model.trim().parse::<i32>().unwrap_or(0);
    let system = formatting::build_system_prompt(&prompt.text);
    let reply = port.apple(system, raw.to_owned(), token_limit).await?;
    Ok(formatting::strip_invisible_chars(&reply))
}

/// HTTP providers: structured request when supported (its errors fall back to the legacy
/// request, as before 005), then the legacy request.
async fn http(
    prompt: &ResolvedPrompt,
    raw: &str,
    choice: &ProviderChoice,
    port: &dyn ProviderPort,
) -> Result<String, FailureReason> {
    if choice.provider.supports_structured_output {
        let request = chat_request(choice, raw.to_owned(), Some(&prompt.text));
        match port.chat(request).await {
            Ok(Some(content)) => return Ok(formatting::structured_text(&content)),
            Ok(None) => return Err(FailureReason::EmptyResult),
            Err(_) => log::warn!("Structured request failed; falling back to the legacy request"),
        }
    }
    let legacy = formatting::legacy_prompt(&prompt.text, raw);
    match port.chat(chat_request(choice, legacy, None)).await {
        Ok(Some(content)) => Ok(formatting::clean_reply(&content)),
        Ok(None) => Err(FailureReason::EmptyResult),
        Err(_) => Err(FailureReason::ProviderError),
    }
}

/// Structured request (`system` = the prompt template, transcript as the user message) or
/// legacy request (`user` = the substituted prompt).
fn chat_request<'a>(
    choice: &'a ProviderChoice,
    user: String,
    template: Option<&str>,
) -> ChatRequest<'a> {
    // Reasoning adds seconds of latency and rarely helps post-processing; these two providers
    // are asked to skip it (llm_client retries without the fields if they are rejected).
    let disable_reasoning = matches!(choice.provider.id.as_str(), "custom" | "openrouter");
    ChatRequest {
        provider: &choice.provider,
        api_key: choice.api_key.expose(),
        model: &choice.model,
        user,
        system: template.map(formatting::build_system_prompt),
        schema: template.map(|_| formatting::transcription_schema()),
        disable_reasoning,
    }
}

#[cfg(test)]
#[path = "processor_tests.rs"]
mod tests;
