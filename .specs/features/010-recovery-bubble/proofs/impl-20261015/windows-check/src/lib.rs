//! Compile-only check of the production Windows recovery window adapter (010): the real
//! `recovery/platform/windows.rs` is compiled as a library for `x86_64-pc-windows-msvc` against
//! the same Tauri version as `src-tauri`. Only its two crate paths are stubbed below
//! (`portable::data_dir`, the window constants of `platform/mod.rs`, copied verbatim). It
//! proves the adapter builds and passes clippy, not that it works: the native Windows bubble
//! (first click without focus theft, Narrator) is pending release evidence.
#![deny(warnings)]

/// Stub of `crate::portable` (the real one reads the portable marker).
pub mod portable {
    /// No portable data directory in this check.
    pub fn data_dir() -> Option<std::path::PathBuf> {
        None
    }
}

/// The constants of `src-tauri/src/recovery/platform/mod.rs` and the adapter.
pub mod platform;
