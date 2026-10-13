// Settings/headless invocations keep inherited console handles in release too.
#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

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

fn main() {
    configure_environment();

    // Transcription worker: runs transcribe.cpp in isolation and never touches
    // Tauri, the CLI, or single-instance handling. Checked after the env setup
    // above so the worker inherits it (e.g. the Vulkan layer opt-out).
    if handy_app_lib::engine_supervisor::is_worker_invocation() {
        std::process::exit(handy_app_lib::engine_supervisor::run_worker());
    }

    // `handy settings` runs and exits here, before any Tauri, tray, window, model or
    // single-instance setup; every other invocation keeps its historical parse.
    let cli_args = match settings_cli::dispatch(std::env::args_os().collect()) {
        settings_cli::Dispatch::App(cli_args) => cli_args,
        settings_cli::Dispatch::Exit(code) => std::process::exit(code),
    };
    // @spec FR-001: Keep CLI handles, detach GUI console — .specs/features/008-settings-cli/spec.md#fr-001
    #[cfg(target_os = "windows")]
    if cli_args.should_detach_console() {
        if let Err(error) = console_windows::detach_console() {
            // Startup must not continue with an unexpectedly attached GUI console.
            // This diagnostic contains only the operation and OS error, never argv.
            eprintln!("Handy GUI startup failed: {error}");
            std::process::exit(1);
        }
    }
    handy_app_lib::run(cli_args)
}
