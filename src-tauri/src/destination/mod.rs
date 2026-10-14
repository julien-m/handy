//! Destination application (feature 006): remember the application frontmost when a dictation
//! actually starts, let the user change or remove it from the badge for that dictation only,
//! return to it before insertion with a bounded wait and a pre-dispatch guard, and publish a
//! truthful outcome while keeping a failed transcript for the recovery features. Destination,
//! insertion method and clipboard handling stay separate: this module only decides which
//! application is activated.
pub mod context;
#[cfg(test)]
pub(crate) mod fake_workspace;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod manager;
mod menu;
pub mod outcome;
pub mod recovery;
pub mod session;
pub mod snapshot;
#[cfg(target_os = "windows")]
pub mod windows;
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
