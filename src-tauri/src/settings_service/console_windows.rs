//! Windows console policy: the GUI branch detaches from any console; command-line branches keep
//! inherited handles and attach to the calling terminal for the ones they lack.

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

/// Give a command-line branch of the GUI-subsystem release binary the calling terminal: when a
/// standard handle (stdin, stdout, stderr, in that order in `missing`) was not inherited, attach
/// to the parent's console and open it for the missing handles only, so a redirection to a file
/// or a pipe is kept exactly as inherited.
///
/// `attach` attaches the parent console; `reopen` opens it for the handle at one index.
// @spec FR-001: Command-line branches reach the calling terminal — .specs/features/008-settings-cli/spec.md#fr-001
pub(crate) fn attach_parent_console_with(
    missing: [bool; 3],
    attach: impl FnOnce() -> std::io::Result<()>,
    mut reopen: impl FnMut(usize) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if !missing.contains(&true) {
        return Ok(());
    }
    attach().map_err(|error| {
        std::io::Error::new(error.kind(), format!("attach parent console: {error}"))
    })?;
    for (index, _) in missing.iter().enumerate().filter(|(_, missing)| **missing) {
        reopen(index)?;
    }
    Ok(())
}

/// Attach the parent's console for the standard handles this process did not inherit.
///
/// Contract: `AttachConsole(ATTACH_PARENT_PROCESS)` fails with `ERROR_ACCESS_DENIED` when this
/// process already has a console (debug console-subsystem build), which counts as attached, and
/// with `ERROR_INVALID_HANDLE` when the parent has none (Explorer, a shortcut); the console is then
/// opened as `CONIN$`/`CONOUT$` and installed with `SetStdHandle`, which std reads at each write.
#[cfg(windows)]
pub(crate) fn attach_parent_console() -> std::io::Result<()> {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ACCESS_DENIED, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Console::{
        AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
        STD_OUTPUT_HANDLE,
    };
    const HANDLES: [u32; 3] = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE];
    // SAFETY: GetStdHandle only reads this process's standard handle table.
    let missing = HANDLES.map(|id| {
        let handle = unsafe { GetStdHandle(id) };
        handle.is_null() || handle == INVALID_HANDLE_VALUE
    });
    let attach = || {
        // SAFETY: no pointers; the error is read right after the failed call.
        if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } == 0
            && unsafe { GetLastError() } != ERROR_ACCESS_DENIED
        {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    };
    attach_parent_console_with(missing, attach, |index| {
        open_console_handle(HANDLES[index], index == 0)
    })
}

/// Open the attached console (`CONIN$` for input, `CONOUT$` otherwise) as standard handle `id`;
/// the handle stays open for the life of the process.
#[cfg(windows)]
fn open_console_handle(id: u32, input: bool) -> std::io::Result<()> {
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Console::SetStdHandle;
    let device = if input { "CONIN$" } else { "CONOUT$" };
    let name = device.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // SAFETY: terminated name; no security attributes or template handle.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    // SAFETY: installs a handle this process owns for its whole life.
    if handle == INVALID_HANDLE_VALUE || unsafe { SetStdHandle(id, handle) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
