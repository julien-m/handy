//! Per-dictation rule snapshots: captured at recording start, compiled at finalization.
//!
//! Until the 008 settings service publishes a compiled snapshot per committed
//! revision, the rules are read from the settings already loaded at recording
//! start, so a change saved mid-recording applies from the next dictation.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use log::{debug, warn};

use super::{matcher::CompiledReplacements, model::ReplacementRule};

/// Rules captured when a recording starts, keyed by shortcut binding.
#[derive(Default)]
pub struct DictationRuleCapture {
    captured: Mutex<HashMap<String, Vec<ReplacementRule>>>,
}

impl DictationRuleCapture {
    /// Record the rules in force when `binding_id` starts recording,
    /// replacing any snapshot left by an abandoned recording.
    pub fn capture(&self, binding_id: &str, rules: Vec<ReplacementRule>) {
        // A poisoned map only means another capture panicked; its data is still
        // a complete snapshot map, so keep using it rather than losing rules.
        let mut captured = self.captured.lock().unwrap_or_else(PoisonError::into_inner);
        captured.insert(binding_id.to_owned(), rules);
    }

    /// Remove and return the snapshot captured for `binding_id`, if any.
    pub fn take(&self, binding_id: &str) -> Option<Vec<ReplacementRule>> {
        let mut captured = self.captured.lock().unwrap_or_else(PoisonError::into_inner);
        captured.remove(binding_id)
    }

    // @spec AC-007: Captured rules apply to their dictation — .specs/features/002-word-replacements/spec.md#ac-007
    /// Rules for the dictation `binding_id` is stopping: its start snapshot,
    /// removed from the capture, or `current()` when the start path recorded
    /// none (for example because recording failed to start). `current` is only
    /// called on that fallback, so the settings are not read otherwise.
    pub fn rules_for_stop<F>(&self, binding_id: &str, current: F) -> Vec<ReplacementRule>
    where
        F: FnOnce() -> Vec<ReplacementRule>,
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
/// Compile one dictation's rules. An invalid stored set (only possible through
/// manual store edits until 008 normalizes it at startup) disables replacement
/// for this dictation with a sanitized warning instead of failing transcription.
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

    #[test]
    fn commit_after_recording_start_applies_from_the_next_dictation() {
        let capture = DictationRuleCapture::default();
        capture.capture("transcribe", vec![rule("a", "cloud code", "Claude Code")]);
        // A commit lands while recording: the settings now hold the new rule.
        let committed = vec![rule("a", "cloud code", "Claude Code CLI")];
        let in_flight = capture.take("transcribe").unwrap_or_default();
        assert_eq!(
            compile_for_dictation(&in_flight).apply("cloud code").text(),
            "Claude Code"
        );
        capture.capture("transcribe", committed);
        let next = capture.take("transcribe").unwrap_or_default();
        assert_eq!(
            compile_for_dictation(&next).apply("cloud code").text(),
            "Claude Code CLI"
        );
        assert_eq!(capture.take("transcribe"), None);
    }

    #[test]
    fn keep_snapshots_separate_per_binding() {
        let capture = DictationRuleCapture::default();
        capture.capture("transcribe", vec![rule("a", "one", "1")]);
        capture.capture("transcribe_with_post_process", vec![]);
        assert_eq!(capture.take("transcribe_with_post_process"), Some(vec![]));
        assert_eq!(capture.take("transcribe").map(|rules| rules.len()), Some(1));
    }

    #[test]
    fn invalid_stored_rules_leave_text_unchanged() {
        let duplicated = vec![rule("a", "Same", "1"), rule("b", "same", "2")];
        let compiled = compile_for_dictation(&duplicated);
        assert!(compiled.is_empty());
        assert_eq!(compiled.apply("same text").text(), "same text");
    }
}
