//! An always-on stream on a fallback is kept while the preferred choice recently failed to open.
use super::{keeps_stream, FailedOpens, FAILED_OPEN_GRACE};
use crate::microphones::policy::{Attempt, AttemptResult, OpenFailure};
use std::time::{Duration, Instant};

fn attempt(choice: &str, result: AttemptResult) -> Attempt {
    Attempt {
        choice: Some(choice.to_owned()),
        result,
    }
}

/// "Desk" was present but busy, "Dock" timed out, "Old" is unplugged; "Headset" opened.
fn failed_opens(at: Instant) -> FailedOpens {
    let mut failed = FailedOpens::default();
    failed.record(
        &[
            attempt(
                "Desk",
                AttemptResult::Open(OpenFailure::Failed("busy".into())),
            ),
            attempt("Dock", AttemptResult::Open(OpenFailure::TimedOut)),
            attempt("Old", AttemptResult::Missing),
        ],
        at,
    );
    failed
}

// @spec AC-004: A present but unopenable first choice does not churn the stream — .specs/features/001-microphone-priority/spec.md#ac-004
#[test]
fn a_fallback_is_kept_while_the_preferred_choice_recently_failed_to_open() {
    let opened_at = Instant::now();
    let failed = failed_opens(opened_at);
    let current = Some(Some("Headset"));
    let soon = opened_at + Duration::from_secs(5);
    assert!(keeps_stream(current, Some("Desk"), &failed, soon));
    assert!(keeps_stream(current, Some("Dock"), &failed, soon));
}

#[test]
fn the_preferred_choice_is_tried_again_after_the_grace() {
    let opened_at = Instant::now();
    let failed = failed_opens(opened_at);
    let later = opened_at + FAILED_OPEN_GRACE;
    assert!(!keeps_stream(
        Some(Some("Headset")),
        Some("Desk"),
        &failed,
        later
    ));
}

#[test]
fn a_choice_that_made_no_open_call_or_a_forgotten_failure_reopens() {
    let opened_at = Instant::now();
    let mut failed = failed_opens(opened_at);
    let current = Some(Some("Headset"));
    // "Old" was missing, so its return is a reconnection to switch to.
    assert!(!keeps_stream(current, Some("Old"), &failed, opened_at));
    failed.forget();
    assert!(!keeps_stream(current, Some("Desk"), &failed, opened_at));
}

#[test]
fn the_preferred_stream_or_its_absence_follows_the_priority() {
    let failed = FailedOpens::default();
    let now = Instant::now();
    assert!(keeps_stream(Some(Some("Desk")), Some("Desk"), &failed, now));
    assert!(keeps_stream(Some(None), None, &failed, now));
    assert!(!keeps_stream(Some(None), Some("Desk"), &failed, now));
    assert!(!keeps_stream(Some(Some("Desk")), None, &failed, now));
    assert!(!keeps_stream(None, None, &failed, now));
}
