//! Destination application (feature 006): remember the application frontmost when a dictation
//! actually starts, let the user change or remove it from the badge for that dictation only,
//! return to it before insertion with a bounded wait and a pre-dispatch guard, and publish a
//! truthful outcome while keeping a failed transcript for the recovery features. Destination,
//! insertion method and clipboard handling stay separate: this module only decides which
//! application is activated. 011 adds the input focused at the start (`field*` modules).
pub mod activation;
pub mod app;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod bounded_worker;
pub mod context;
pub mod delivery;
#[cfg(test)]
pub(crate) mod fake_workspace;
pub mod field;
pub mod field_attempt;
pub mod field_capture;
pub mod field_check;
mod field_lifetime;
pub mod guard;
pub mod history;
pub mod insertion;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub mod macos_field;
#[cfg(target_os = "macos")]
pub mod macos_focus;
pub mod manager;
mod menu;
pub mod outcome;
pub mod pin;
pub mod presentation;
pub mod recovery;
pub mod rollout;
pub mod session;
pub mod snapshot;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod uia_mapping;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub mod windows_field;
#[cfg(target_os = "windows")]
mod windows_uia;
pub mod workspace;

/// The workspace adapter of this platform (an unsupported one without destination support).
pub fn native_workspace() -> std::sync::Arc<dyn workspace::WorkspacePort> {
    #[cfg(target_os = "macos")]
    return std::sync::Arc::new(macos::MacWorkspace::new());
    #[cfg(target_os = "windows")]
    return std::sync::Arc::new(windows::WindowsWorkspace::new());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return std::sync::Arc::new(workspace::UnsupportedWorkspace);
}

// @spec FR-005: macOS and Windows only without a setting — .specs/features/011-dictation-field-return/spec.md#fr-005
/// The field port of this platform for 011 (unsupported on Linux: no destination there).
pub fn native_field_access() -> std::sync::Arc<dyn field::FieldAccess> {
    #[cfg(target_os = "macos")]
    return std::sync::Arc::new(macos_field::MacFieldAccess);
    #[cfg(target_os = "windows")]
    return std::sync::Arc::new(windows_field::WindowsFieldAccess::new());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return std::sync::Arc::new(field::UnsupportedFieldAccess);
}
