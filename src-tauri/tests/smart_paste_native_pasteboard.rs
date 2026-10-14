//! Opt-in native checks of the production pasteboard adapter and snapshot
//! helper on an isolated, uniquely named pasteboard. The general clipboard,
//! focus and keyboard are never touched; no key event is posted.
//!
//! On-demand pasteboard items are served by the process main run loop, which
//! libtest keeps busy, so this target has its own `main` (harness = false):
//! the scenario runs on a worker thread while the main thread pumps the run
//! loop. Run with `cargo test --test smart_paste_native_pasteboard -- --ignored`.

#[cfg(target_os = "macos")]
#[path = "smart_paste_native_pasteboard/hardening.rs"]
mod hardening;
#[cfg(target_os = "macos")]
#[path = "smart_paste_native_pasteboard/recorder.rs"]
mod recorder;
#[cfg(target_os = "macos")]
#[path = "smart_paste_native_pasteboard/scenario.rs"]
mod scenario;
#[cfg(target_os = "macos")]
#[path = "smart_paste_native_pasteboard/support.rs"]
mod support;

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    support::main_with_run_loop(scenario::run_all)
}

#[cfg(not(target_os = "macos"))]
fn main() {}
