//! Bounded probes with real worker threads: a slow probe is abandoned at its budget and the
//! running-probe cap refuses new probes until stalled ones end. One test: the cap is global.
use super::{call_within, MAX_RUNNING_PROBES, RUNNING_PROBES};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Wait (bounded) until every probe thread ended.
fn drained() -> bool {
    let started = Instant::now();
    while RUNNING_PROBES.load(Ordering::Acquire) > 0 && started.elapsed() < Duration::from_secs(5) {
        std::thread::yield_now();
    }
    RUNNING_PROBES.load(Ordering::Acquire) == 0
}

#[test]
fn slow_probes_are_abandoned_and_capped_until_they_end() {
    assert_eq!(call_within(Duration::from_secs(5), || 7), Some(7));
    let mut releases = Vec::new();
    for _ in 0..MAX_RUNNING_PROBES {
        let (release, wait) = mpsc::channel::<()>();
        releases.push(release);
        let answer = call_within(Duration::from_millis(5), move || wait.recv().is_ok());
        assert_eq!(answer, None, "a probe outlasting its budget is abandoned");
    }
    assert_eq!(
        call_within(Duration::from_secs(5), || 7),
        None,
        "cap reached"
    );
    drop(releases);
    assert!(drained());
    assert_eq!(call_within(Duration::from_secs(5), || 7), Some(7));
}
