//! Opt-in AppKit spike, isolated from the user's general pasteboard.
#![cfg(target_os = "macos")]

use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::time::{Duration, Instant};

// Includes Swift compilation; the wrapper has tighter per-process deadlines.
const WRAPPER_TIMEOUT: Duration = Duration::from_secs(120);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

fn stop_fixture(child: &mut std::process::Child) -> std::io::Result<()> {
    // SAFETY: spawn created this owned child's process group; the controlled
    // wrapper and native readers inherit or explicitly join that group.
    let result = unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
    if result != 0 {
        eprintln!(
            "Native fixture group teardown failed: {}",
            std::io::Error::last_os_error()
        );
        if child.try_wait()?.is_none() {
            child.kill()?;
        }
    }
    child.wait()?;
    Ok(())
}

type Drain = std::thread::JoinHandle<std::io::Result<Vec<u8>>>;

fn drain(mut pipe: impl std::io::Read + Send + 'static) -> Drain {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes).map(|_| bytes)
    })
}

fn collect(reader: Drain) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = reader.join().map_err(|_| "pipe reader panicked")??;
    Ok(bytes)
}

// @spec FR-005: AppKit provider feasibility — .specs/features/007-smart-paste/spec.md#fr-005
// @spec FR-010: Native evidence — .specs/features/007-smart-paste/spec.md#fr-010
#[test]
#[ignore = "Runs the native Swift/AppKit fixture with an isolated named pasteboard"]
fn native_item_provider_characterizes_cache_and_services_reads_during_worker_hold(
) -> Result<(), Box<dyn std::error::Error>> {
    let project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Cargo manifest is inside the worktree")?;
    let mut child = std::process::Command::new("bun")
        .arg("scripts/smart-paste-native-proof.ts")
        .current_dir(project)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Isolate the wrapper and its descendants for bounded teardown.
        .process_group(0)
        .spawn()?;
    // Drain both pipes while waiting: Bun spins instead of exiting when a
    // pipe it writes to is never read (observed with 512 bytes pending).
    let stdout = drain(child.stdout.take().ok_or("Bun stdout missing")?);
    let stderr = drain(child.stderr.take().ok_or("Bun stderr missing")?);
    let deadline = Instant::now() + WRAPPER_TIMEOUT;
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            stop_fixture(&mut child)?;
            return Err("Bun native fixture exceeded its 120-second safety deadline".into());
        }
        std::thread::sleep(PROCESS_POLL_INTERVAL);
    }
    let status = child.wait()?;
    let (stdout, stderr) = (collect(stdout)?, collect(stderr)?);
    assert!(
        status.success(),
        "Native spike failed: {}",
        String::from_utf8_lossy(&stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&stdout)?;
    assert_eq!(report["general_clipboard_touched"], false);
    assert_eq!(report["callbacks_during_worker_hold"], true);
    assert_eq!(report["lazy_fulfillment_preserves_change_count"], true);
    assert_eq!(report["markers_eager"], true);
    assert_eq!(report["snapshot_roundtrip_equal"], true);
    assert_eq!(report["prekey_callback_qualifies"], false);
    assert_eq!(report["first_postkey_callback_count"], 1);
    assert_eq!(report["all_reads_received_full_fixture"], true);
    // Characterization of this OS's counterexamples, not insertion acceptance.
    assert_eq!(report["cache_limitation_observed"], true);
    assert_eq!(report["prekey_cached_after_keydown"], true);
    assert_eq!(report["repeated_postkey_read_cached"], true);
    assert_eq!(report["prekey_then_postkey_callback_count"], 1);
    assert_eq!(report["repeated_postkey_callback_count"], 1);
    assert_eq!(report["prekey_qualifying_callback_count"], 0);
    assert_eq!(report["compiler_exit_status"], 0);
    assert_eq!(report["native_exit_status"], 0);
    println!("{}", String::from_utf8_lossy(&stdout));
    Ok(())
}
