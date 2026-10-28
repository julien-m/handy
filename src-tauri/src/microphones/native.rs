//! Native adapters of the resolution probes: enumeration of the input devices and the lid
//! state, each run on its own worker and awaited at most its budget, never on the webview/main
//! loop. A probe still running when its budget ends is abandoned (a blocked native call cannot
//! be cancelled) and its late answer discarded; at most `MAX_RUNNING_PROBES` run at once, so a
//! stuck backend cannot accumulate threads: past that, a probe fails at once (status unknown).
use super::catalog::{Catalog, CatalogEntry};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

/// Probes allowed to run at once. A recording start runs at most two enumerations and one lid
/// probe; four leaves room for a status refresh of the settings window without unbounded growth.
pub const MAX_RUNNING_PROBES: usize = 4;
static RUNNING_PROBES: AtomicUsize = AtomicUsize::new(0);

/// Run `probe` on a worker and wait at most `budget` for its answer.
pub fn call_within<T: Send + 'static>(
    budget: Duration,
    probe: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    if RUNNING_PROBES.fetch_add(1, Ordering::AcqRel) >= MAX_RUNNING_PROBES {
        RUNNING_PROBES.fetch_sub(1, Ordering::AcqRel);
        return None;
    }
    let (sender, answer) = mpsc::sync_channel(1);
    let spawned = std::thread::Builder::new()
        .name("microphone-probe".into())
        .spawn(move || {
            let result = probe();
            RUNNING_PROBES.fetch_sub(1, Ordering::AcqRel);
            // The caller may have stopped waiting; a late answer is dropped on purpose.
            let _ = sender.send(result);
        });
    if spawned.is_err() {
        RUNNING_PROBES.fetch_sub(1, Ordering::AcqRel);
        return None;
    }
    answer.recv_timeout(budget).ok()
}

/// One fresh enumeration of the input devices within `budget`, or `None` when it failed or
/// timed out. A device whose name cannot be read makes the catalog partial.
pub fn enumerate_within(budget: Duration) -> Option<Catalog<cpal::Device>> {
    call_within(budget, enumerate).flatten()
}
fn enumerate() -> Option<Catalog<cpal::Device>> {
    let host = crate::audio_toolkit::get_cpal_host();
    let devices = host.input_devices().ok()?;
    let mut entries = Vec::new();
    let mut complete = true;
    for device in devices {
        match device.name() {
            Ok(name) => entries.push(CatalogEntry { name, device }),
            Err(_) => complete = false,
        }
    }
    Some(match complete {
        true => Catalog::new(entries),
        false => Catalog::partial(entries),
    })
}

/// Whether the lid is confirmed closed within `budget`; `None` when unknown.
pub fn lid_closed_within(budget: Duration) -> Option<bool> {
    call_within(budget, || crate::helpers::clamshell::is_clamshell().ok()).flatten()
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
