//! Opt-in native check of the production AX adapter and direct-insertion
//! algorithm against disposable, offscreen synthetic AppKit fields.
//!
//! No activation, key event or clipboard API is used: the fixture windows are
//! ordered front offscreen without becoming active, and every fixture process
//! is killed and reaped by its owner.
#![cfg(target_os = "macos")]

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::ax_port::AxTextPort;
use handy_app_lib::smart_paste::clock::SystemClock;
use handy_app_lib::smart_paste::direct::{attempt_direct_insertion, DirectAttempt, GuardVerdict};
use handy_app_lib::smart_paste::macos_ax::MacAxTextPort;

// @spec FR-011: Native AX adapter evidence — .specs/features/007-smart-paste/spec.md#fr-011
// @spec FR-010: Native evidence — .specs/features/007-smart-paste/spec.md#fr-010

// Generates offscreen fields: "small" (selection over an astral character),
// "large" (one unit beyond the 1,000,000 guard), "delayed" (applies writes
// 151 ms later, after the read-back window) and "secure" (password field).
// The process prints READY once its main loop runs, then waits to be killed.
const FIXTURE_SOURCE: &str = r#"import AppKit
class DelayedTextView: NSTextView {
    override func setAccessibilitySelectedText(_ selectedText: String?) {
        let range = selectedRange()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.151) { [self] in
            insertText(selectedText ?? "", replacementRange: range)
        }
    }
}
let mode = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "small"
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let window = NSWindow(contentRect: NSRect(x: -5000, y: -5000, width: 320, height: 100),
    styleMask: [.titled], backing: .buffered, defer: false)
let frame = NSRect(x: 10, y: 10, width: 300, height: 70)
if mode == "secure" {
    let secure = NSSecureTextField(frame: frame)
    window.contentView?.addSubview(secure)
    window.makeFirstResponder(secure)
} else {
    let text: NSTextView = mode == "delayed" ? DelayedTextView(frame: frame) : NSTextView(frame: frame)
    text.string = mode == "large" ? String(repeating: "x", count: 1000001) : "A😀Z"
    text.setSelectedRange(NSRange(location: 1, length: 2))
    window.contentView?.addSubview(text)
    window.makeFirstResponder(text)
}
window.orderFrontRegardless()
app.finishLaunching()
DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
    print("READY")
    fflush(stdout)
}
app.run()
"#;

// Swift compilation includes compiler startup; one minute fails closed.
const COMPILE_TIMEOUT: Duration = Duration::from_secs(60);
// Fixture GUI registration needs no interaction; ten seconds fails closed.
const READY_TIMEOUT: Duration = Duration::from_secs(10);
// The delayed fixture applies 151 ms after the write; 100 ms more is ample.
const LATE_WRITE_SETTLE: Duration = Duration::from_millis(250);
const TRANSCRIPT: &str = "é🦀";
// Warm-up contact is not part of the measured attempt; one second fails closed.
const AX_WARMUP_TIMEOUT: Duration = Duration::from_secs(1);

/// Owned fixture process, killed and reaped on drop.
struct Fixture(Child);

impl Drop for Fixture {
    fn drop(&mut self) {
        // Already-exited fixtures make kill fail harmlessly; wait reaps.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_with_deadline(child: &mut Child, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status.success());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = child.kill();
    let _ = child.wait();
    Err("swiftc exceeded its deadline".into())
}

fn compile_fixture(directory: &Path) -> Result<PathBuf, String> {
    let source = directory.join("fixture.swift");
    let binary = directory.join("fixture");
    std::fs::write(&source, FIXTURE_SOURCE).map_err(|error| error.to_string())?;
    let mut compiler = Command::new("xcrun")
        .args(["swiftc", "-O"])
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .spawn()
        .map_err(|error| error.to_string())?;
    if !wait_with_deadline(&mut compiler, COMPILE_TIMEOUT)? {
        return Err("swiftc failed".into());
    }
    Ok(binary)
}

fn launch(binary: &Path, mode: &str) -> Result<(Fixture, i32), String> {
    let mut child = Command::new(binary)
        .arg(mode)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = child.stdout.take().ok_or("fixture stdout missing")?;
    let pid = i32::try_from(child.id()).map_err(|error| error.to_string())?;
    let fixture = Fixture(child);
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(stdout).read_line(&mut line);
        let _ = sender.send(line);
    });
    match receiver.recv_timeout(READY_TIMEOUT) {
        Ok(line) if line.trim() == "READY" => Ok((fixture, pid)),
        _ => Err(format!("{mode} fixture did not become ready")),
    }
}

/// Contacts the fixture's AX tree once without writing, as a long-running
/// destination would already be; returns the warm-up duration in ms.
fn warm_up(port: &mut MacAxTextPort, pid: i32) -> f64 {
    let started = Instant::now();
    if let Ok(mut field) = port.focused_field(pid, AX_WARMUP_TIMEOUT) {
        let _ = field.is_secure(AX_WARMUP_TIMEOUT);
    }
    started.elapsed().as_secs_f64() * 1000.0
}

fn run_mode(binary: &Path, mode: &str, is_warm: bool) -> Result<DirectAttempt, String> {
    let (_fixture, pid) = launch(binary, mode)?;
    let mut port = MacAxTextPort;
    let warmup_ms = if is_warm {
        warm_up(&mut port, pid)
    } else {
        0.0
    };
    let started = Instant::now();
    let attempt = attempt_direct_insertion(
        &SystemClock,
        &mut port,
        pid,
        &TRANSCRIPT.encode_utf16().collect::<Vec<_>>(),
    );
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{{\"mode\":\"{mode}\",\"warm\":{is_warm},\"warmup_ms\":{warmup_ms:.3},\"attempt\":\"{attempt:?}\",\"elapsed_ms\":{elapsed_ms:.3}}}"
    );
    assert!(elapsed_ms < 400.0, "{mode} took {elapsed_ms} ms");
    Ok(check_late_write(mode, attempt))
}

fn check_late_write(mode: &str, attempt: DirectAttempt) -> DirectAttempt {
    let DirectAttempt::Unchanged(mut guard) = attempt else {
        return attempt;
    };
    std::thread::sleep(LATE_WRITE_SETTLE);
    let verdict = guard.check();
    println!("{{\"mode\":\"{mode}\",\"late_guard\":\"{verdict:?}\"}}");
    assert_eq!(verdict, GuardVerdict::AppliedLate);
    DirectAttempt::Unchanged(guard)
}

fn is_expected(mode: &str, attempt: &DirectAttempt) -> bool {
    let printed = format!("{attempt:?}");
    match mode {
        "small" => matches!(attempt, DirectAttempt::Verified),
        "large" => printed == "Skipped(TooLarge)",
        "delayed" => matches!(attempt, DirectAttempt::Unchanged(_)),
        _ => printed == "Skipped(SecureField)",
    }
}

#[test]
#[ignore = "Launches offscreen synthetic AppKit fixtures and needs accessibility trust"]
fn production_ax_adapter_verifies_skips_and_detects_late_writes() -> Result<(), String> {
    if !MacAxTextPort.is_trusted() {
        return Err("accessibility permission not granted: pending release evidence".into());
    }
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let binary = compile_fixture(directory.path())?;
    // A first AX contact with a freshly launched app may exceed the 100 ms
    // preparation budget: that is characterized, and must only ever skip.
    let cold = run_mode(&binary, "small", false)?;
    let cold_printed = format!("{cold:?}");
    assert!(
        matches!(cold, DirectAttempt::Verified) || cold_printed.starts_with("Skipped("),
        "cold attempt: {cold_printed}"
    );
    for mode in ["small", "large", "delayed", "secure"] {
        let attempt = run_mode(&binary, mode, true)?;
        assert!(is_expected(mode, &attempt), "{mode}: {attempt:?}");
    }
    Ok(())
}
