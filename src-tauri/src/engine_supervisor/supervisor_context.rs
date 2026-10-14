//! Vocabulary context preparation on the loaded model (feature 003): one short call to the
//! current worker, which fits the terms with the model's tokenizer. A status query is bounded so
//! a settings read never waits behind a dictation, and it never starts a worker.
use super::{
    engine_stopped, not_loaded, unexpected, Command, EngineError, EngineSupervisor, Owner,
};
use crate::engine_supervisor::protocol::{Request, Response};
use crate::recognition_vocabulary::context::ContextPreparation;
use std::sync::mpsc;
use std::time::Duration;

impl EngineSupervisor {
    /// Fit `terms` (eligible vocabulary, list order) to the loaded model's context mechanism.
    /// With `wait`, the answer is bounded and no worker is started (status queries); without it,
    /// the call queues like a transcription and may restart the model's worker (dictation).
    ///
    /// # Errors
    /// `Failed` when no model is loaded, a stream is active, the worker failed or, with `wait`,
    /// the engine did not answer in time.
    pub fn prepare_context(
        &self,
        terms: Vec<String>,
        wait: Option<Duration>,
    ) -> Result<ContextPreparation, EngineError> {
        let (reply, answer) = mpsc::channel();
        let command = Command::PrepareContext {
            terms,
            start_worker: wait.is_none(),
            reply,
        };
        self.commands.send(command).map_err(|_| engine_stopped())?;
        match wait {
            None => answer.recv().unwrap_or_else(|_| Err(engine_stopped())),
            Some(wait) => answer.recv_timeout(wait).unwrap_or_else(|_| {
                Err(EngineError::Failed(
                    "the transcription engine did not answer in time".to_string(),
                ))
            }),
        }
    }
}

impl Owner {
    /// Owner side of [`EngineSupervisor::prepare_context`]. An active stream is never disturbed:
    /// the worker serves only that stream until it ends.
    pub(super) fn prepare_context(
        &mut self,
        terms: Vec<String>,
        start_worker: bool,
    ) -> Result<ContextPreparation, EngineError> {
        if self.stream.is_some() {
            return Err(EngineError::Failed("a live stream is active".to_string()));
        }
        if start_worker {
            self.ensure_worker()?;
        }
        let deadline = self.deadlines().call_floor;
        let worker = self.worker.as_mut().ok_or_else(not_loaded)?;
        match worker.call(&Request::PrepareContext { terms }, None, deadline) {
            Ok(Response::ContextPrepared(prepared)) => Ok(prepared),
            Ok(other) => Err(unexpected(&other).into()),
            Err(failure) => {
                if failure.worker_lost() {
                    self.worker_lost("context preparation", &failure);
                }
                Err(failure.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_loaded_model_no_context_is_prepared_and_no_worker_starts() {
        let engine = EngineSupervisor::new(false);
        let status =
            engine.prepare_context(vec!["Claude Code".into()], Some(Duration::from_secs(5)));
        assert!(matches!(status, Err(EngineError::Failed(_))));
        let dictation = engine.prepare_context(vec!["Claude Code".into()], None);
        assert!(matches!(dictation, Err(EngineError::Failed(_))));
        assert!(engine.loaded().is_none());
    }
}
