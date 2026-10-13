//! Deterministic interleavings of two ordered writes on one key: the second writer is released
//! exactly where an unordered implementation would let it run between the first writer's
//! preparation (or commit) and its effect. The native state must end equal to the disk.
use super::{mirrored, owned, stored, SpyPort, TestResult};
use crate::settings_service::{
    live::LiveCommit,
    protocol::Revision,
    sequence::{write_prepared, write_value, LiveEffectsPort},
};
use serde_json::{json, Value};
use std::sync::Mutex;

/// Deterministic interleaving: writer A pauses (at most `PAUSE`) inside its write, at the point
/// where an unordered implementation lets writer B run completely. In the write order, B waits
/// for A instead, so the pause simply times out.
const PAUSE: std::time::Duration = std::time::Duration::from_millis(300);
struct Interleaving {
    paused: std::sync::mpsc::SyncSender<()>,
    resumed: Mutex<std::sync::mpsc::Receiver<()>>,
}
/// Run `first` (which calls `pause` once) and, as soon as it pauses, `second` on another thread.
fn interleave(
    first: impl FnOnce(&Interleaving) + Send,
    second: impl FnOnce() + Send,
) -> Result<(), Box<dyn std::error::Error>> {
    let (paused, on_pause) = std::sync::mpsc::sync_channel(1);
    let (done, resumed) = std::sync::mpsc::sync_channel(1);
    let gate = Interleaving {
        paused,
        resumed: Mutex::new(resumed),
    };
    std::thread::scope(|scope| {
        let a = scope.spawn(|| first(&gate));
        let b = scope.spawn(move || {
            if on_pause.recv().is_ok() {
                second();
            }
            let _ = done.send(());
        });
        a.join().map_err(|_| "first writer panicked")?;
        b.join().map_err(|_| "second writer panicked")?;
        Ok(())
    })
}
impl Interleaving {
    fn pause(&self) {
        let _ = self.paused.send(());
        if let Ok(resumed) = self.resumed.lock() {
            let _ = resumed.recv_timeout(PAUSE);
        }
    }
}
#[test]
fn concurrent_prepared_writes_of_one_key_leave_native_state_equal_to_disk() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    port.set_native("selected_channel", Value::Null);
    interleave(
        |gate| {
            let mut change = mirrored(&port, "selected_channel", json!(1));
            let prepare = change.prepare;
            change.prepare = Box::new(move |before, requested| {
                prepare(before, requested)?;
                gate.pause();
                Ok(())
            });
            let _ = write_prepared(&live, &port, "selected_channel", json!(1), change);
        },
        || {
            let change = mirrored(&port, "selected_channel", json!(2));
            let _ = write_prepared(&live, &port, "selected_channel", json!(2), change);
        },
    )?;
    let disk = stored(&directory, "selected_channel")?;
    assert_eq!(port.native("selected_channel"), Some(disk));
    Ok(())
}
/// Pauses inside the live effect of the first commit that changes `key` to `value`.
struct PausingPort<'a> {
    spy: &'a SpyPort,
    gate: &'a Interleaving,
    value: Value,
}
impl LiveEffectsPort for PausingPort<'_> {
    fn apply(&self, commit: &LiveCommit, skip: &[&str]) -> Result<(), Vec<&'static str>> {
        let after = serde_json::to_value(&commit.after).unwrap_or_default();
        if after["always_on_microphone"] == self.value {
            self.gate.pause();
        }
        self.spy.apply(commit, skip)
    }
    fn notify(&self, revision: Revision, keys: &[&'static str]) {
        self.spy.notify(revision, keys);
    }
}
#[test]
fn concurrent_effect_writes_of_one_key_leave_native_state_equal_to_disk() -> TestResult {
    let directory = tempfile::tempdir()?;
    let live = owned(&directory)?;
    let port = SpyPort::default();
    interleave(
        |gate| {
            let pausing = PausingPort {
                spy: &port,
                gate,
                value: json!(true),
            };
            let _ = write_value(&live, &pausing, "always_on_microphone", json!(true));
        },
        || {
            let _ = write_value(&live, &port, "always_on_microphone", json!(false));
        },
    )?;
    let disk = stored(&directory, "always_on_microphone")?;
    assert_eq!(port.native("always_on_microphone"), Some(disk));
    Ok(())
}
