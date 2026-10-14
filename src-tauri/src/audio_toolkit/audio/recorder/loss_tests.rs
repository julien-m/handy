//! Input loss during a recording (feature 001, FR-004): the captured prefix is returned without
//! waiting for a pause the dead stream can no longer acknowledge, the owner is told once, and the
//! prefix stays recoverable when the consumer ends without answering Stop.
use super::{
    lock_samples, run_consumer, AudioRecorder, CaptureProcessor, CaptureTransportState, Cmd,
    VadPolicy, PAUSE_ACK_TIMEOUT,
};
use rtrb::{Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;
/// Two 30 ms frames at 16 kHz: enough for the first-sample acknowledgement.
const PREFIX: usize = 960;

/// A consumer running on its own thread, as the capture worker runs it.
struct Consumer {
    commands: mpsc::Sender<Cmd>,
    producer: Producer<f32>,
    stream_error: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    losses: mpsc::Receiver<()>,
    worker: thread::JoinHandle<()>,
}
fn consumer() -> Consumer {
    let (producer, ring) = RingBuffer::<f32>::new(16_000);
    let (commands, receiver) = mpsc::channel();
    let (loss_tx, losses) = mpsc::channel();
    let stream_error = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::new()));
    let processor = CaptureProcessor::new(16_000, None, None, None, Instant::now())
        .with_stream_ports(
            Arc::clone(&samples),
            Some(Arc::new(move || {
                let _ = loss_tx.send(());
            })),
        );
    let transport = Arc::new(CaptureTransportState::default());
    let error = Arc::clone(&stream_error);
    let worker = thread::spawn(move || run_consumer(processor, ring, receiver, transport, error));
    Consumer {
        commands,
        producer,
        stream_error,
        samples,
        losses,
        worker,
    }
}
/// Start a recording and feed it `PREFIX` samples until the consumer reports readiness.
fn record_prefix(consumer: &mut Consumer) -> TestResult {
    let (ready_tx, ready_rx) = mpsc::channel();
    let start = Cmd::Start(VadPolicy::Disabled, Instant::now(), ready_tx);
    consumer.commands.send(start)?;
    for _ in 0..PREFIX {
        consumer.producer.push(0.25)?;
    }
    ready_rx.recv_timeout(Duration::from_secs(5))?;
    Ok(())
}

// @spec AC-004: Loss never discards the captured audio — .specs/features/001-microphone-priority/spec.md#ac-004
#[test]
fn a_lost_stream_returns_its_prefix_without_waiting_for_the_pause() -> TestResult {
    let mut consumer = consumer();
    record_prefix(&mut consumer)?;
    consumer.stream_error.store(true, Ordering::Release);
    consumer.losses.recv_timeout(Duration::from_secs(5))?;
    let (reply_tx, reply_rx) = mpsc::channel();
    let stopped_at = Instant::now();
    consumer.commands.send(Cmd::Stop(reply_tx))?;
    let samples = reply_rx.recv_timeout(Duration::from_secs(5))?;
    assert!(stopped_at.elapsed() < PAUSE_ACK_TIMEOUT);
    assert!(
        samples.len() >= PREFIX / 2,
        "prefix kept: {}",
        samples.len()
    );
    consumer.worker.join().map_err(|_| "consumer panicked")?;
    assert!(consumer.losses.try_recv().is_err(), "loss reported once");
    Ok(())
}

#[test]
fn the_processed_prefix_survives_a_consumer_ending_without_stop() -> TestResult {
    let mut consumer = consumer();
    record_prefix(&mut consumer)?;
    // Readiness follows the first drained chunk, which may hold only part of the prefix: wait
    // until the whole prefix is processed so the oracle does not depend on thread scheduling.
    wait_for_processed(&consumer.samples, PREFIX)?;
    consumer.commands.send(Cmd::Shutdown)?;
    consumer.worker.join().map_err(|_| "consumer panicked")?;
    assert_eq!(lock_samples(&consumer.samples).len(), PREFIX);
    Ok(())
}

/// Wait (at most 5 s) until `samples` holds at least `count` processed samples.
fn wait_for_processed(samples: &Mutex<Vec<f32>>, count: usize) -> TestResult {
    let deadline = Instant::now() + Duration::from_secs(5);
    while lock_samples(samples).len() < count {
        if Instant::now() >= deadline {
            return Err(format!("fewer than {count} samples processed within 5 s").into());
        }
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

#[test]
fn stop_without_a_consumer_returns_the_salvaged_prefix() -> TestResult {
    let mut recorder = AudioRecorder::new()?;
    let (commands, receiver) = mpsc::channel();
    drop(receiver);
    recorder.cmd_tx = Some(commands);
    lock_samples(&recorder.salvage).extend([0.5_f32; 12]);
    assert_eq!(recorder.stop()?.len(), 12);
    assert!(lock_samples(&recorder.salvage).is_empty());
    Ok(())
}
