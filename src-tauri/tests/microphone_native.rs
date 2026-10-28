//! Native microphone harness of feature 001 (Step 1): real CoreAudio, WASAPI or ALSA/PulseAudio
//! devices, never fakes. Every test is ignored by default because it needs audio hardware and,
//! except the enumeration probe, microphone permission; run them on purpose with
//! `cargo test --manifest-path src-tauri/Cargo.toml --test microphone_native -- --ignored --nocapture --test-threads=1`
//! (or through `bun scripts/microphone-native-proof.ts`). Each test prints one JSON observation
//! line on stdout with timings and counters only: no device name and no audio leaves the process.
use handy_app_lib::audio_toolkit::{list_input_devices, AudioRecorder, VadPolicy};
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// AC-003: the whole resolution, enumeration included, ends within 6 s.
const RESOLUTION_DEADLINE: Duration = Duration::from_secs(6);
/// A late stream must be dropped quickly once its slow open returns; 10 s covers a cold
/// CoreAudio device on a loaded machine.
const LATE_RELEASE_WAIT: Duration = Duration::from_secs(10);
/// Window left to the operator to unplug the device during the loss test.
const UNPLUG_WINDOW: Duration = Duration::from_secs(60);

fn observe(test: &str, fields: serde_json::Value) {
    println!(
        "{}",
        json!({ "test": test, "os": std::env::consts::OS, "observed": fields })
    );
}

#[test]
#[ignore = "native: enumerates the audio devices of this machine"]
fn enumeration_is_bounded_and_groups_same_named_devices() -> TestResult {
    let started = Instant::now();
    let devices = list_input_devices().map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let mut groups: HashMap<String, usize> = HashMap::new();
    for device in &devices {
        *groups.entry(device.name.clone()).or_default() += 1;
    }
    let ambiguous = groups.values().filter(|count| **count > 1).count();
    observe(
        "enumeration",
        json!({ "devices": devices.len(), "ambiguous_groups": ambiguous, "ms": elapsed.as_millis() }),
    );
    assert!(elapsed < RESOLUTION_DEADLINE);
    Ok(())
}

#[test]
#[ignore = "native: opens the default microphone (needs microphone permission)"]
fn a_default_open_is_bounded_and_releases_the_device() -> TestResult {
    let mut recorder = AudioRecorder::new()?;
    let opened_at = Instant::now();
    recorder.open(None)?;
    let first_open = opened_at.elapsed();
    let enumerated_while_open = list_input_devices().map(|d| d.len()).unwrap_or(0);
    recorder.close()?;
    let reopened_at = Instant::now();
    recorder.open(None)?;
    let second_open = reopened_at.elapsed();
    recorder.close()?;
    observe(
        "default_open",
        json!({ "first_ms": first_open.as_millis(), "reopen_ms": second_open.as_millis(),
                "devices_while_open": enumerated_while_open }),
    );
    assert!(first_open <= Duration::from_secs(2) + Duration::from_millis(100));
    Ok(())
}

#[test]
#[ignore = "native: abandons a default open (needs microphone permission)"]
fn a_timed_out_open_drops_its_late_stream_and_frees_the_device() -> TestResult {
    let mut recorder = AudioRecorder::new()?;
    let timed_out = recorder.open_within(None, Duration::ZERO).is_err();
    let waited = Instant::now();
    while recorder.stalled_opens() > 0 && waited.elapsed() < LATE_RELEASE_WAIT {
        std::thread::sleep(Duration::from_millis(10));
    }
    let late_release = waited.elapsed();
    let stalled = recorder.stalled_opens();
    recorder.open(None)?;
    recorder.close()?;
    observe(
        "late_stream",
        json!({ "timed_out": timed_out, "stalled_after_wait": stalled,
                "late_release_ms": late_release.as_millis() }),
    );
    assert_eq!(stalled, 0);
    Ok(())
}

#[test]
#[ignore = "native, interactive: unplug the opened microphone when asked"]
fn an_unplugged_microphone_ends_capture_with_its_prefix() -> TestResult {
    let (lost_tx, lost_rx) = mpsc::channel();
    let mut recorder = AudioRecorder::new()?.with_loss_callback(move || {
        let _ = lost_tx.send(Instant::now());
    });
    recorder.open(None)?;
    let ready = recorder.start(VadPolicy::Disabled)?;
    ready.recv_timeout(Duration::from_secs(5))?;
    eprintln!("Speak, then unplug the default microphone within {UNPLUG_WINDOW:?}.");
    let lost_at = lost_rx.recv_timeout(UNPLUG_WINDOW)?;
    let samples = recorder.stop()?;
    let stop_ms = lost_at.elapsed().as_millis();
    let _ = recorder.close();
    observe(
        "loss_prefix",
        json!({ "prefix_samples": samples.len(), "stop_after_loss_ms": stop_ms }),
    );
    assert!(!samples.is_empty());
    Ok(())
}
