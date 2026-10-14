//! Publication of an open attempt (feature 001, FR-003, delivery review fix 1): an attempt
//! abandoned after its timeout keeps its own stream-error flag and config, so its late backend
//! error cannot end the stream that did open, and its late success cannot write the cache.
use super::opening::OpenedStream;
use super::AudioRecorder;
use rtrb::RingBuffer;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

type TestResult = Result<(), Box<dyn Error>>;

/// A stream that opened on a device named `name`, with a config to cache.
fn opened(name: &str) -> OpenedStream<()> {
    let (_producer, samples) = RingBuffer::<f32>::new(16);
    let config = cpal::SupportedStreamConfig::new(
        1,
        cpal::SampleRate(16_000),
        cpal::SupportedBufferSize::Unknown,
        cpal::SampleFormat::F32,
    );
    OpenedStream {
        stream: (),
        sample_rate: 16_000,
        samples,
        cache_entry: Some((name.to_owned(), config)),
    }
}

/// A recorder whose loss callback reports on the returned channel.
fn recorder() -> Result<(AudioRecorder, mpsc::Receiver<()>), Box<dyn Error>> {
    let (loss_tx, losses) = mpsc::channel();
    let recorder = AudioRecorder::new()?.with_loss_callback(move || {
        let _ = loss_tx.send(());
    });
    Ok((recorder, losses))
}

fn cached_device(recorder: &AudioRecorder) -> Option<String> {
    let cache = recorder
        .config_cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    cache.as_ref().map(|(name, _)| name.clone())
}

/// Wait (at most 5 s) until every abandoned open of `recorder` has ended.
fn wait_for_stalled_opens_to_end(recorder: &AudioRecorder) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(5);
    while recorder.stalled_opens() > 0 {
        if Instant::now() >= deadline {
            return Err("the abandoned open did not end within 5 s".into());
        }
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

/// Attempt A: its native open outlives the caller's timeout, so it is abandoned. Returns the
/// channel carrying A's stream-error flag and the sender that lets A's open complete.
fn stall_an_attempt(recorder: &mut AudioRecorder) -> Result<StalledAttempt, Box<dyn Error>> {
    let (flag_tx, flag_rx) = mpsc::channel::<Arc<AtomicBool>>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let stalled = recorder.open_attempt(Duration::from_millis(10), move |_, stream_error| {
        let _ = flag_tx.send(stream_error);
        let _ = release_rx.recv();
        Ok(opened("stalled"))
    });
    let error = stalled.err().ok_or("the stalled attempt must time out")?;
    let kind = error.downcast_ref::<std::io::Error>().map(|e| e.kind());
    assert_eq!(kind, Some(std::io::ErrorKind::TimedOut));
    Ok((flag_rx, release_tx))
}
type StalledAttempt = (mpsc::Receiver<Arc<AtomicBool>>, mpsc::Sender<()>);

// @spec AC-003: An abandoned attempt never affects the stream that opened — .specs/features/001-microphone-priority/spec.md#ac-003
#[test]
fn an_abandoned_attempt_failing_late_leaves_the_opened_stream_healthy() -> TestResult {
    let (mut recorder, losses) = recorder()?;
    let (flag_rx, release_tx) = stall_an_attempt(&mut recorder)?;

    // Attempt B (the fallback) opens and is published.
    recorder.open_attempt(Duration::from_secs(5), |_, _| Ok(opened("fallback")))?;

    // A's stream then reports a backend error and its open completes late.
    let stalled_error = flag_rx.recv_timeout(Duration::from_secs(5))?;
    stalled_error.store(true, Ordering::Release);
    release_tx.send(())?;
    wait_for_stalled_opens_to_end(&recorder)?;

    assert!(!Arc::ptr_eq(&stalled_error, &recorder.stream_error));
    assert!(!recorder.needs_reopen(), "B's stream stays healthy");
    // B's consumer checks its own flag every 10 ms; ten polls report no loss.
    let loss = losses.recv_timeout(Duration::from_millis(100));
    assert!(loss.is_err(), "no loss reported for B");
    assert_eq!(cached_device(&recorder).as_deref(), Some("fallback"));
    recorder.close()?;
    Ok(())
}

#[test]
fn the_published_flag_is_the_one_the_opened_stream_reports_on() -> TestResult {
    let (mut recorder, losses) = recorder()?;
    recorder.open_attempt(Duration::from_secs(5), |_, _| Ok(opened("microphone")))?;
    assert!(!recorder.needs_reopen());
    // What the published stream's error callback would do.
    recorder.stream_error.store(true, Ordering::Release);
    losses.recv_timeout(Duration::from_secs(5))?;
    assert!(recorder.needs_reopen());
    recorder.close()?;
    Ok(())
}

#[test]
fn a_failed_attempt_clears_the_cached_config() -> TestResult {
    let (mut recorder, _losses) = recorder()?;
    recorder.open_attempt(Duration::from_secs(5), |_, _| Ok(opened("microphone")))?;
    recorder.close()?;
    assert_eq!(cached_device(&recorder).as_deref(), Some("microphone"));
    let failed = recorder.open_attempt::<(), _>(Duration::from_secs(5), |_, _| {
        Err("Failed to build input stream: device gone".to_owned())
    });
    assert!(failed.is_err());
    assert_eq!(cached_device(&recorder), None);
    assert!(!recorder.needs_reopen());
    Ok(())
}
