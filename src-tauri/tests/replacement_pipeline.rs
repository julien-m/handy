//! Dictation wiring harness with injected ports (AC-007, plan step 4).
//!
//! It drives the same public functions as the shortcut action: capture at
//! recording start, rule selection at stop (capture, else current settings),
//! one compilation per dictation and the stream-finalize/batch fallback. The
//! speech engine and the settings store are replaced by closures, so no Tauri
//! application is built.

use std::cell::Cell;
use std::sync::Arc;

use handy_app_lib::word_replacements::{
    dictation::{compile_for_dictation, finalize_or_transcribe, DictationRuleCapture},
    matcher::CompiledReplacements,
    model::ReplacementRule,
};

const BINDING: &str = "transcribe";
const RECOGNIZED: &str = "open cloud code now";

type TestResult = Result<(), String>;

fn rule_list(replacement: &str) -> Vec<ReplacementRule> {
    vec![ReplacementRule {
        id: "rule-1".into(),
        recognized: "cloud code".into(),
        replacement: replacement.into(),
    }]
}

/// The compiled snapshot a recording start captures.
fn rules(replacement: &str) -> Arc<CompiledReplacements> {
    Arc::new(compile_for_dictation(&rule_list(replacement)))
}

/// Simulate the stop path: select the compiled rules, finalize or batch.
fn stop_dictation(
    capture: &DictationRuleCapture,
    current: Arc<CompiledReplacements>,
    stream: Result<Option<&str>, String>,
) -> Result<String, String> {
    let replacements = capture.rules_for_stop(BINDING, || current);
    finalize_or_transcribe(
        &replacements,
        |matcher| stream.map(|text| text.map(|text| matcher.apply(text).text())),
        |matcher| Ok(matcher.apply(RECOGNIZED).text()),
    )
}

#[test]
fn rules_captured_at_start_win_over_a_commit_during_recording() -> TestResult {
    let capture = DictationRuleCapture::default();
    capture.capture(BINDING, rules("Claude Code"));
    let output = stop_dictation(&capture, rules("Claude Code CLI"), Ok(None))?;
    assert_eq!(output, "open Claude Code now");
    assert!(
        capture.take(BINDING).is_none(),
        "the stop path consumes the capture"
    );
    Ok(())
}

#[test]
fn missing_capture_falls_back_to_current_settings() -> TestResult {
    let capture = DictationRuleCapture::default();
    let output = stop_dictation(&capture, rules("Claude Code CLI"), Ok(None))?;
    assert_eq!(output, "open Claude Code CLI now");
    Ok(())
}

#[test]
fn current_settings_are_not_read_when_a_capture_exists() {
    let capture = DictationRuleCapture::default();
    capture.capture(BINDING, rules("Claude Code"));
    let reads = Cell::new(0);
    let selected = capture.rules_for_stop(BINDING, || {
        reads.set(reads.get() + 1);
        rules("unused")
    });
    assert_eq!(selected.apply("cloud code").text(), "Claude Code");
    assert_eq!(reads.get(), 0);
}

#[test]
fn next_dictation_uses_the_rules_committed_before_it_started() -> TestResult {
    let capture = DictationRuleCapture::default();
    capture.capture(BINDING, rules("Claude Code"));
    stop_dictation(&capture, rules("Claude Code CLI"), Ok(None))?;
    capture.capture(BINDING, rules("Claude Code CLI"));
    let output = stop_dictation(&capture, rules("ignored"), Ok(None))?;
    assert_eq!(output, "open Claude Code CLI now");
    Ok(())
}

#[test]
fn stream_and_batch_fallback_share_one_compiled_matcher() -> TestResult {
    let replacements = compile_for_dictation(&rule_list("Claude Code"));
    let finalize_seen = Cell::new(std::ptr::null::<CompiledReplacements>());
    let output = finalize_or_transcribe(
        &replacements,
        |matcher| {
            finalize_seen.set(matcher);
            Ok::<_, String>(Some("  ".to_owned()))
        },
        |matcher| {
            assert!(std::ptr::eq(matcher, finalize_seen.get()));
            assert!(std::ptr::eq(matcher, &replacements));
            Ok(matcher.apply(RECOGNIZED).text())
        },
    )?;
    assert_eq!(output, "open Claude Code now");
    Ok(())
}

#[test]
fn usable_stream_text_skips_the_batch_run() -> TestResult {
    let capture = DictationRuleCapture::default();
    capture.capture(BINDING, rules("Claude Code"));
    let output = stop_dictation(&capture, rules("unused"), Ok(Some("cloud code streamed")))?;
    assert_eq!(output, "Claude Code streamed");
    let batch_runs = Cell::new(0);
    let replacements = CompiledReplacements::empty();
    let output = finalize_or_transcribe(
        &replacements,
        |_| Ok::<_, String>(Some("kept".to_owned())),
        |_| {
            batch_runs.set(batch_runs.get() + 1);
            Ok(String::new())
        },
    )?;
    assert_eq!((output.as_str(), batch_runs.get()), ("kept", 0));
    Ok(())
}

#[test]
fn cancelled_finalize_never_starts_a_batch_run() {
    let capture = DictationRuleCapture::default();
    capture.capture(BINDING, rules("Claude Code"));
    let result = stop_dictation(&capture, rules("unused"), Err("cancelled".to_owned()));
    assert_eq!(result, Err("cancelled".to_owned()));
}
