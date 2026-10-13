//! Settings descriptors, validation, redacted wire contracts, the atomic repository and the
//! live service through which every writer of the running app commits, and the early CLI path.

mod atomic;
pub mod cli;
pub mod command_error;
pub mod descriptor;
mod descriptor_data;
pub mod domains;
pub(crate) mod edit;
mod effects;
#[cfg(unix)]
mod endpoint_unix;
#[cfg(windows)]
mod endpoint_windows;
pub mod errors;
pub mod input;
pub mod ipc;
pub mod live;
pub mod locator;
mod native_changes;
mod normalization;
pub mod operations;
pub mod ownership;
#[cfg(unix)]
mod platform_unix;
#[cfg(windows)]
mod platform_windows;
pub mod protocol;
pub mod public;
mod raw_merge;
pub mod render;
mod request;
pub mod route;
pub(crate) mod sequence;
pub mod store;
pub(crate) mod tauri_adapter;
pub mod transaction;
pub mod ui_value;
pub mod validation;
#[cfg(windows)]
mod windows_store;
/// Apply the runtime logging state of `settings` (startup), honoring a `--debug` launch.
pub(crate) fn apply_runtime_logging(settings: &crate::settings::AppSettings) {
    effects::apply_logging(settings);
}
// The Windows platform module reaches the passive probe as `settings_service::instance_probe`.
#[cfg(windows)]
use ownership::probe as instance_probe;
