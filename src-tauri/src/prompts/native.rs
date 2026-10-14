//! Production provider port of the AI step: HTTP chat completions through `llm_client`, and
//! Apple Intelligence on one bounded blocking worker. The native Apple call cannot be
//! interrupted: when the budget expires its future is dropped (the result is discarded), but the
//! worker keeps its slot until the call returns, and requests meanwhile fail at once with
//! `provider_busy` instead of piling up workers.
use super::processor::{ChatRequest, FailureReason, PortFuture, ProviderPort};
use super::provider::{AvailabilityProbe, NativeAvailability};

/// The device's providers.
pub struct NativeProviders;

impl ProviderPort for NativeProviders {
    fn chat<'a>(
        &'a self,
        request: ChatRequest<'a>,
    ) -> PortFuture<'a, Result<Option<String>, String>> {
        Box::pin(crate::llm_client::send_chat_completion_with_schema(
            request.provider,
            request.api_key.to_owned(),
            request.model,
            request.user,
            request.system,
            request.schema,
            request.disable_reasoning,
        ))
    }

    fn apple<'a>(
        &'a self,
        system: String,
        user: String,
        token_limit: i32,
    ) -> PortFuture<'a, Result<String, FailureReason>> {
        Box::pin(apple_worker(system, user, token_limit))
    }

    fn apple_available(&self) -> bool {
        NativeAvailability.apple_intelligence_available()
    }
}

/// One Apple Intelligence request on the single native worker slot.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
async fn apple_worker(
    system: String,
    user: String,
    token_limit: i32,
) -> Result<String, FailureReason> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUSY: AtomicBool = AtomicBool::new(false);
    if BUSY.swap(true, Ordering::AcqRel) {
        return Err(FailureReason::ProviderBusy);
    }
    let worker = tauri::async_runtime::spawn_blocking(move || {
        let result =
            crate::apple_intelligence::process_text_with_system_prompt(&system, &user, token_limit);
        // Released only once the native call has returned, even after the step gave up on it.
        BUSY.store(false, Ordering::Release);
        result
    });
    match worker.await {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(_)) | Err(_) => {
            log::warn!("Apple Intelligence processing failed");
            Err(FailureReason::ProviderError)
        }
    }
}
/// Apple Intelligence does not exist off Apple silicon.
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
async fn apple_worker(
    _system: String,
    _user: String,
    _token_limit: i32,
) -> Result<String, FailureReason> {
    // The parameters are unused: there is no native provider to run them on this platform.
    Err(FailureReason::ProviderUnavailable)
}
