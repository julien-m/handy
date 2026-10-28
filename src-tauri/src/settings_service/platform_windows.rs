//! Windows current-logon private-pipe primitives (the passive owner probe is in `instance_probe`).

// Bound SID text without truncation; SHA-256 store identities use 64 hex digits.
const MAX_SID_TEXT_CHARS: usize = 192;
const STORE_HASH_HEX_CHARS: usize = 64;

/// Transfer an owned pipe only through the authentication boundary; errors close it.
pub(crate) fn authenticate_owned_pipe<Pipe>(
    pipe: Pipe,
    verify: impl FnOnce(&Pipe) -> std::io::Result<()>,
) -> std::io::Result<Pipe> {
    verify(&pipe)?;
    Ok(pipe)
}

/// Construct a private pipe identity from a validated SID and SHA-256 store digest.
pub(crate) fn pipe_name(logon_sid: &str, store_hash: &str) -> std::io::Result<String> {
    let sid_valid = logon_sid.starts_with("S-1-")
        && logon_sid.len() < MAX_SID_TEXT_CHARS
        && logon_sid[4..]
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    if !sid_valid
        || store_hash.len() != STORE_HASH_HEX_CHARS
        || !store_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid private identity",
        ));
    }
    Ok(format!(r"\\.\pipe\handy-settings-{logon_sid}-{store_hash}"))
}

#[cfg(windows)]
mod native {
    use super::MAX_SID_TEXT_CHARS;
    use std::fs::File;
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, HANDLE};
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenLogonSid, TOKEN_GROUPS, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Pipes::{
        GetNamedPipeClientProcessId, GetNamedPipeServerProcessId,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    // Bounded token inspection keeps local identity allocation finite.
    const TOKEN_BUFFER_BYTES: u32 = 65_536;

    /// Obtain the current logon-session SID, which is stricter than a user SID alone.
    pub(crate) fn current_logon_sid() -> io::Result<String> {
        let mut token: HANDLE = std::ptr::null_mut();
        // SAFETY: valid process pseudo-handle and initialized writable token output.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = token_logon_sid(token);
        // SAFETY: OpenProcessToken succeeded and this function owns the handle.
        unsafe {
            CloseHandle(token);
        }
        result
    }

    fn token_logon_sid(token: HANDLE) -> io::Result<String> {
        // Pointer alignment is required for TOKEN_GROUPS; token data is bounded to 64 KiB.
        let mut buffer = vec![0usize; TOKEN_BUFFER_BYTES as usize / std::mem::size_of::<usize>()];
        let mut length = 0;
        // SAFETY: aligned writable buffer has its declared size; token remains live.
        // Windows supplies valid SID pointers inside TOKEN_GROUPS and LocalAlloc text.
        unsafe {
            if GetTokenInformation(
                token,
                TokenLogonSid,
                buffer.as_mut_ptr().cast(),
                TOKEN_BUFFER_BYTES,
                &mut length,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            if length < std::mem::size_of::<TOKEN_GROUPS>() as u32 || length > TOKEN_BUFFER_BYTES {
                return Err(unavailable());
            }
            let groups = &*buffer.as_ptr().cast::<TOKEN_GROUPS>();
            if groups.GroupCount != 1 {
                return Err(unavailable());
            }
            let mut text: *mut u16 = std::ptr::null_mut();
            if ConvertSidToStringSidW(groups.Groups[0].Sid, &mut text) == 0 {
                return Err(io::Error::last_os_error());
            }
            let result = decode_sid_text(text);
            LocalFree(text.cast());
            result
        }
    }

    /// Decode a terminated native SID without truncation; the caller frees its allocation.
    /// Safety: text is the live string returned by a successful ConvertSidToStringSidW.
    unsafe fn decode_sid_text(text: *const u16) -> io::Result<String> {
        let mut size = 0;
        // SAFETY: the SID conversion API supplies a terminated UTF-16 allocation.
        // Stop at the accepted identity bound so overlong input is refused, not truncated.
        unsafe {
            while size < MAX_SID_TEXT_CHARS && *text.add(size) != 0 {
                size += 1;
            }
            if size == MAX_SID_TEXT_CHARS {
                return Err(unavailable());
            }
            String::from_utf16(std::slice::from_raw_parts(text, size)).map_err(|_| unavailable())
        }
    }

    fn unavailable() -> io::Error {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "private resource unavailable",
        )
    }

    /// Authenticate the connected client process token without changing thread identity.
    pub(crate) fn verify_client(pipe: &File, expected_logon_sid: &str) -> io::Result<()> {
        let mut process_id = 0;
        // SAFETY: a live connected pipe and initialized writable process-ID output.
        if unsafe { GetNamedPipeClientProcessId(pipe.as_raw_handle(), &mut process_id) } == 0 {
            return Err(io::Error::last_os_error());
        }
        verify_process(process_id, expected_logon_sid)
    }

    /// Authenticate the server before a connecting client submits any settings bytes.
    pub(crate) fn verify_server(pipe: &File, expected_logon_sid: &str) -> io::Result<()> {
        let mut process_id = 0;
        // SAFETY: the connected pipe is owned by File; output remains live during the call.
        if unsafe { GetNamedPipeServerProcessId(pipe.as_raw_handle(), &mut process_id) } == 0 {
            return Err(io::Error::last_os_error());
        }
        verify_process(process_id, expected_logon_sid)
    }

    fn verify_process(process_id: u32, expected_logon_sid: &str) -> io::Result<()> {
        // Hold the peer process handle while reading its token, rather than impersonating
        // before ReadFile (which authenticates the last pipe message, not the connection).
        // SAFETY: process/token handles are only queried, initialized and closed once.
        unsafe {
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if process.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut token: HANDLE = std::ptr::null_mut();
            let opened = OpenProcessToken(process, TOKEN_QUERY, &mut token);
            let error = io::Error::last_os_error();
            CloseHandle(process);
            if opened == 0 {
                return Err(error);
            }
            let result = token_logon_sid(token);
            CloseHandle(token);
            if result? != expected_logon_sid {
                return Err(unavailable());
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
pub(crate) use native::*;

#[cfg(windows)]
#[path = "pipe_server_windows.rs"]
mod pipe_server;
#[cfg(windows)]
pub(crate) use pipe_server::{PrivatePipe, ServerPipe};
