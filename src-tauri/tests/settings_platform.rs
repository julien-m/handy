//! Entry point for private-resource and passive startup boundary tests.

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/settings_service/console_windows.rs"]
mod console_windows;
#[path = "../src/settings_service/instance_probe.rs"]
mod instance_probe;
#[cfg(unix)]
#[path = "../src/settings_service/platform_unix.rs"]
mod platform_unix;
#[path = "../src/settings_service/platform_windows.rs"]
mod platform_windows;

#[path = "settings_platform/cli_console.rs"]
mod cli_console;
#[cfg(unix)]
#[path = "settings_platform/unix.rs"]
mod unix;
#[path = "settings_platform/windows.rs"]
mod windows;
