//! Windows passive owner inspection and current-logon private-pipe primitives.

// Bound SID text without truncation; SHA-256 store identities use 64 hex digits.
const MAX_SID_TEXT_CHARS: usize = 192;
const STORE_HASH_HEX_CHARS: usize = 64;

/// Transfer an owned pipe only through the authentication boundary; errors close it.
pub(crate) fn authenticate_owned_pipe(
    file: std::fs::File,
    verify: impl FnOnce(&std::fs::File) -> std::io::Result<()>,
) -> std::io::Result<std::fs::File> {
    verify(&file)?;
    Ok(file)
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
    use super::super::instance_probe::{classify_owner, LegacyState, ProbeScope};
    use super::{authenticate_owned_pipe, pipe_name, MAX_SID_TEXT_CHARS};
    use std::fs::File;
    use std::io;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, LocalFree, ERROR_FILE_NOT_FOUND, ERROR_PIPE_CONNECTED, HANDLE,
        INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenLogonSid, SECURITY_ATTRIBUTES, TOKEN_GROUPS, TOKEN_QUERY,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeClientProcessId,
        GetNamedPipeServerProcessId, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
        PIPE_TYPE_BYTE, PIPE_WAIT,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenMutexW, OpenProcess, OpenProcessToken,
        PROCESS_QUERY_LIMITED_INFORMATION, SYNCHRONIZATION_SYNCHRONIZE,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;

    // Bounded token inspection and byte-pipe buffers keep local channel allocation finite.
    const TOKEN_BUFFER_BYTES: u32 = 65_536;
    const PIPE_BUFFER_BYTES: u32 = 65_536;

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    /// Inspect existing plugin resources only; never create a mutex/window or send WM_COPYDATA.
    // @spec FR-004: Passive Windows legacy detection — .specs/features/008-settings-cli/spec.md#fr-004
    pub(crate) fn inspect_legacy_owner() -> LegacyState {
        let mutex = wide("com.pais.handy-sim");
        let class = wide("com.pais.handy-sic");
        let window = wide("com.pais.handy-siw");
        // SYNCHRONIZE only opens an existing mutex; no wait/acquisition occurs.
        // SAFETY: names are terminated UTF-16 buffers alive through these calls.
        unsafe {
            let handle = OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, mutex.as_ptr());
            let error = GetLastError();
            let target = FindWindowW(class.as_ptr(), window.as_ptr());
            if !handle.is_null() {
                CloseHandle(handle);
                return LegacyState::Occupied;
            }
            if !target.is_null() {
                return LegacyState::Occupied;
            }
            if error == ERROR_FILE_NOT_FOUND {
                // Unprefixed mutexes and FindWindow inspect only this session;
                // another session of the same user may still own the settings.
                classify_owner(Ok(false), ProbeScope::CurrentSession)
            } else {
                LegacyState::Unavailable
            }
        }
    }

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

    /// A first-instance byte pipe with a protected current-logon DACL and no remote clients.
    // @spec FR-004: Current-logon private settings pipe — .specs/features/008-settings-cli/spec.md#fr-004
    pub(crate) struct PrivatePipe {
        file: File,
        logon_sid: String,
    }

    impl PrivatePipe {
        /// Create only the new settings pipe. The caller owns deadlines/worker lifecycle.
        pub(crate) fn create(store_hash: &str) -> io::Result<Self> {
            let logon_sid = current_logon_sid()?;
            let name = wide(&pipe_name(&logon_sid, store_hash)?);
            // Only this logon SID receives rights; protected DACL forbids inherited grants.
            let sddl = wide(&format!("D:P(A;;GA;;;{logon_sid})"));
            let mut descriptor = std::ptr::null_mut();
            // SAFETY: terminated buffers and attributes live through CreateNamedPipe;
            // descriptor is LocalAlloc-owned and transferred handle is owned by File.
            unsafe {
                if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    SDDL_REVISION_1,
                    &mut descriptor,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return Err(io::Error::last_os_error());
                }
                let attributes = SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: descriptor,
                    bInheritHandle: 0,
                };
                let handle = CreateNamedPipeW(
                    name.as_ptr(),
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    1,
                    PIPE_BUFFER_BYTES,
                    PIPE_BUFFER_BYTES,
                    0,
                    &attributes,
                );
                let error = io::Error::last_os_error();
                LocalFree(descriptor);
                if handle == INVALID_HANDLE_VALUE {
                    return Err(error);
                }
                Ok(Self {
                    file: File::from_raw_handle(handle),
                    logon_sid,
                })
            }
        }

        /// Consume the listener and return its file only after authenticating the client.
        /// Every refusal closes the owned handle before any frame is consumed.
        pub(crate) fn accept(self) -> io::Result<File> {
            // SAFETY: File keeps the synchronous pipe handle alive through the call.
            unsafe {
                if ConnectNamedPipe(self.file.as_raw_handle(), std::ptr::null_mut()) == 0
                    && GetLastError() != ERROR_PIPE_CONNECTED
                {
                    return Err(io::Error::last_os_error());
                }
            }
            let logon_sid = self.logon_sid;
            authenticate_owned_pipe(self.file, |file| verify_client(file, &logon_sid))
        }
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
