// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Command-line branches (`handy settings`, headless list/transcription) attach to the calling
// terminal instead (see `attach_terminal`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use handy_app_lib::settings_service::cli as settings_cli;

#[cfg(target_os = "windows")]
#[path = "settings_service/console_windows.rs"]
mod console_windows;

/// Process-wide environment that every later branch (worker, settings, app) inherits.
fn configure_environment() {
    #[cfg(target_os = "linux")]
    {
        // DMABUF renderer causes crashes on various GPU/display server configurations
        // See: https://github.com/tauri-apps/tauri/issues/9394
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    #[cfg(target_os = "windows")]
    {
        // Avoid overlay/capture layer crashes (#2049). Set before backend
        // initialization, preserving user overrides.
        if std::env::var_os("VK_LOADER_LAYERS_DISABLE").is_none()
            && !handy_app_lib::env_flag_enabled("HANDY_KEEP_VULKAN_IMPLICIT_LAYERS")
        {
            std::env::set_var("VK_LOADER_LAYERS_DISABLE", "~implicit~");
        }
    }
}

/// Smart-paste snapshot helper: reads the pasteboard read-only and streams it
/// to the parent, without Tauri, the CLI or single-instance handling.
fn exit_if_smart_paste_snapshot_helper() {
    #[cfg(target_os = "macos")]
    if handy_app_lib::smart_paste::snapshot_helper::is_helper_invocation() {
        std::process::exit(handy_app_lib::smart_paste::snapshot_helper::run_helper());
    }
}

fn main() {
    configure_environment();

    // Transcription worker: runs transcribe.cpp in isolation and never touches
    // Tauri, the CLI, or single-instance handling. Checked after the env setup
    // above so the worker inherits it (e.g. the Vulkan layer opt-out).
    if handy_app_lib::engine_supervisor::is_worker_invocation() {
        std::process::exit(handy_app_lib::engine_supervisor::run_worker());
    }

    // Hidden pasteboard helper: exits before the settings CLI and the app parse argv.
    exit_if_smart_paste_snapshot_helper();

    let argv: Vec<std::ffi::OsString> = std::env::args_os().collect();
    #[cfg(target_os = "windows")]
    if settings_cli::settings_requested(&argv) {
        attach_terminal();
    }
    // `handy settings` runs and exits here, before any Tauri, tray, window, model or
    // single-instance setup; every other invocation keeps its historical parse.
    let cli_args = match settings_cli::dispatch(argv) {
        settings_cli::Dispatch::App(cli_args) => cli_args,
        settings_cli::Dispatch::Exit(code) => std::process::exit(code),
    };
    #[cfg(target_os = "windows")]
    configure_console(&cli_args);
    handy_app_lib::run(cli_args)
}

/// The GUI detaches from any console; headless list and transcription runs print to the
/// terminal they were started from.
// @spec FR-001: Keep CLI handles, detach GUI console — .specs/features/008-settings-cli/spec.md#fr-001
#[cfg(target_os = "windows")]
fn configure_console(cli_args: &handy_app_lib::CliArgs) {
    if !cli_args.should_detach_console() {
        attach_terminal();
        return;
    }
    if let Err(error) = console_windows::detach_console() {
        // Startup must not continue with an unexpectedly attached GUI console.
        // This diagnostic contains only the operation and OS error, never argv.
        eprintln!("Handy GUI startup failed: {error}");
        std::process::exit(1);
    }
}

/// Give a command-line branch the terminal it was started from (stdout, stderr, stdin), keeping
/// inherited redirections. Started without a parent console (Explorer, a shortcut), there is
/// nowhere to print: the failure is ignored and the exit code still reports the result.
#[cfg(target_os = "windows")]
fn attach_terminal() {
    let _ = console_windows::attach_parent_console();
}
