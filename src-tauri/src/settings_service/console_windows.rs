//! Fallible GUI console detachment; CLI branches retain their inherited handles.

/// Detach only this process, propagating a safe operation context on failure.
/// Windows documents success even when no console is attached, so that no-op
/// follows the same success path without guessing from an error code.
// @spec FR-001: Fallible GUI console detachment — .specs/features/008-settings-cli/spec.md#fr-001
pub(crate) fn detach_console_with(
    free_console: impl FnOnce() -> std::io::Result<()>,
) -> std::io::Result<()> {
    free_console().map_err(|error| {
        std::io::Error::new(error.kind(), format!("detach GUI process console: {error}"))
    })
}

/// Detach the GUI process without hiding or closing its parent's shared console.
#[cfg(windows)]
pub(crate) fn detach_console() -> std::io::Result<()> {
    detach_console_with(|| {
        // SAFETY: FreeConsole takes no pointers and affects only the calling
        // process. Capture its failure immediately before any other OS call.
        if unsafe { windows_sys::Win32::System::Console::FreeConsole() } == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    })
}
