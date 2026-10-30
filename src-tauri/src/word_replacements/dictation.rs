//! Per-dictation rule snapshots: the compiled matcher of the committed settings revision
//! (`published`) is captured at recording start and used at finalization, so a change saved
//! mid-recording applies from the next dictation and one dictation never mixes rule versions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use log::{debug, warn};

use super::{matcher::CompiledReplacements, model::ReplacementRule};

/// Compiled rules captured when a recording starts, keyed by shortcut binding.
#[derive(Default)]
pub struct DictationRuleCapture {
    captured: Mutex<HashMap<String, Arc<CompiledReplacements>>>,
}

impl DictationRuleCapture {
    /// Record the compiled rules in force when `binding_id` starts recording,
    /// replacing any snapshot left by an abandoned recording.
    pub fn capture(&self, binding_id: &str, rules: Arc<CompiledReplacements>) {
        // A poisoned map only means another capture panicked; its data is still
        // a complete snapshot map, so keep using it rather than losing rules.
        let mut captured = self.captured.lock().unwrap_or_else(PoisonError::into_inner);
        captured.insert(binding_id.to_owned(), rules);
    }

    /// Remove and return the snapshot captured for `binding_id`, if any.
    pub fn take(&self, binding_id: &str) -> Option<Arc<CompiledReplacements>> {
        let mut captured = self.captured.lock().unwrap_or_else(PoisonError::into_inner);
        captured.remove(binding_id)
    }

    // @spec AC-007: Captured rules apply to their dictation — .specs/features/002-word-replacements/spec.md#ac-007
    /// Rules for the dictation `binding_id` is stopping: its start snapshot,
    /// removed from the capture, or `current()` when the start path recorded
    /// none (for example because recording failed to start). `current` is only
    /// called on that fallback, so the settings are not read otherwise.
    pub fn rules_for_stop<F>(&self, binding_id: &str, current: F) -> Arc<CompiledReplacements>
    where
        F: FnOnce() -> Arc<CompiledReplacements>,
    {
        self.take(binding_id).unwrap_or_else(|| {
            // Falling back to the committed rules beats silently skipping them.
            debug!("No replacement rules captured at recording start; using current rules");
            current()
        })
    }
}

/// Final text of one dictation, with one compiled matcher shared by both paths.
///
/// `finalize` finalizes a live stream; usable (non-blank) text wins. An empty
/// result (no stream, nothing produced, stream failure) falls back to `batch`
/// over the same audio. A finalize error (such as a cancel) is returned as is,
/// so a cancelled dictation never starts a batch run.
///
/// # Errors
///
/// The error of `finalize`, or of `batch` when the fallback runs.
pub fn finalize_or_transcribe<E, F, B>(
    replacements: &CompiledReplacements,
    finalize: F,
    batch: B,
) -> Result<String, E>
where
    F: FnOnce(&CompiledReplacements) -> Result<Option<String>, E>,
    B: FnOnce(&CompiledReplacements) -> Result<String, E>,
{
    match finalize(replacements) {
        Ok(Some(text)) if !text.trim().is_empty() => Ok(text),
        Ok(_) => batch(replacements),
        Err(error) => Err(error),
    }
}

// @spec AC-007: Captured rules apply to their dictation — .specs/features/002-word-replacements/spec.md#ac-007
/// Compile one rule snapshot. An invalid stored set (only possible through a
/// manual store edit: every commit is validated) disables replacement with a
/// sanitized warning instead of failing transcription.
pub fn compile_for_dictation(rules: &[ReplacementRule]) -> CompiledReplacements {
    match CompiledReplacements::compile(rules) {
        Ok(compiled) => compiled,
        Err(error) => {
            warn!("Word replacements skipped for this dictation: {error}");
            CompiledReplacements::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, recognized: &str, replacement: &str) -> ReplacementRule {
        ReplacementRule {
            id: id.into(),
            recognized: recognized.into(),
            replacement: replacement.into(),
        }
    }

    fn compiled(rules: &[ReplacementRule]) -> Arc<CompiledReplacements> {
        Arc::new(compile_for_dictation(rules))
    }

    #[test]
    fn commit_after_recording_start_applies_from_the_next_dictation() {
        let capture = DictationRuleCapture::default();
        capture.capture(
            "transcribe",
            compiled(&[rule("a", "cloud code", "Claude Code")]),
        );
        // A commit lands while recording: the settings now hold the new rule.
        let committed = compiled(&[rule("a", "cloud code", "Claude Code CLI")]);
        let in_flight = capture.rules_for_stop("transcribe", || committed.clone());
        assert_eq!(in_flight.apply("cloud code").text(), "Claude Code");
        capture.capture("transcribe", committed);
        let next = capture.rules_for_stop("transcribe", || compiled(&[]));
        assert_eq!(next.apply("cloud code").text(), "Claude Code CLI");
        assert!(capture.take("transcribe").is_none());
    }

    #[test]
    fn keep_snapshots_separate_per_binding() {
        let capture = DictationRuleCapture::default();
        capture.capture("transcribe", compiled(&[rule("a", "one", "1")]));
        capture.capture("transcribe_with_post_process", compiled(&[]));
        let other = capture.take("transcribe_with_post_process");
        assert!(other.is_some_and(|rules| rules.is_empty()));
        let own = capture.take("transcribe");
        assert!(own.is_some_and(|rules| !rules.is_empty()));
    }

    #[test]
    fn invalid_stored_rules_leave_text_unchanged() {
        let duplicated = vec![rule("a", "Same", "1"), rule("b", "same", "2")];
        let compiled = compile_for_dictation(&duplicated);
        assert!(compiled.is_empty());
        assert_eq!(compiled.apply("same text").text(), "same text");
    }
}
