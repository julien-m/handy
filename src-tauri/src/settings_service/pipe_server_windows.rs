//! Server end of the private settings pipe: overlapped instances under a protected current-logon
//! DACL that refuse remote clients, a bounded wait for the next client, and read/write deadlines,
//! so neither an idle endpoint nor a silent client holds the serving thread.
use super::{authenticate_owned_pipe, current_logon_sid, pipe_name, verify_client};
use std::cell::Cell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::time::Duration;
use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{
    GetLastError, LocalFree, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_NO_DATA,
    ERROR_OPERATION_ABORTED, ERROR_PIPE_CONNECTED, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows_sys::Win32::Storage::FileSystem::{
    ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::{
    CreateEventW, ResetEvent, WaitForSingleObject, INFINITE,
};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};

// Byte-pipe buffers keep local channel allocation finite.
const PIPE_BUFFER_BYTES: u32 = 65_536;
// Instances alive at once: the one waiting for the next client plus the connections being
// served. 16 bounds the serving threads same-session clients can hold; beyond it, creating the
// next instance fails and the server backs off until a connection ends.
const MAX_PIPE_INSTANCES: u32 = 16;

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

/// One instance of the private pipe, waiting for a client.
// @spec FR-004: Current-logon private settings pipe — .specs/features/008-settings-cli/spec.md#fr-004
pub(crate) struct PrivatePipe {
    pipe: Overlapped,
    logon_sid: String,
}
impl PrivatePipe {
    /// Create one instance. `first` publishes the name and fails when any instance of it exists
    /// (another server, live or squatting); later instances join this server's own name so it
    /// never lapses while a connection is served.
    pub(crate) fn create(store_hash: &str, first: bool) -> io::Result<Self> {
        let logon_sid = current_logon_sid()?;
        let name = wide(&pipe_name(&logon_sid, store_hash)?);
        let descriptor = LogonDescriptor::new(&logon_sid)?;
        let handle = descriptor.create_instance(&name, first)?;
        Ok(Self {
            pipe: Overlapped::new(handle)?,
            logon_sid,
        })
    }
    /// Wait at most `wait` for a client: false when none connected. A client that connected
    /// and left before the wait counts as connected; its authentication then fails.
    pub(crate) fn wait_for_client(&self, wait: Duration) -> io::Result<bool> {
        // SAFETY: `run` passes the live pipe handle and an OVERLAPPED it keeps alive until the
        // operation completed or was cancelled and drained.
        let connected = self.pipe.run(Some(wait), |handle, overlapped| unsafe {
            ConnectNamedPipe(handle, overlapped)
        });
        match connected {
            Ok(outcome) => Ok(outcome.is_some()),
            Err(error)
                if [ERROR_PIPE_CONNECTED, ERROR_NO_DATA]
                    .iter()
                    .any(|code| error.raw_os_error() == Some(*code as i32)) =>
            {
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }
    /// Authenticate the connected client's logon session before any byte is read; a refusal
    /// closes this instance.
    pub(crate) fn authenticate(self) -> io::Result<ServerPipe> {
        let logon_sid = self.logon_sid;
        let pipe =
            authenticate_owned_pipe(self.pipe, |pipe| verify_client(&pipe.file, &logon_sid))?;
        Ok(ServerPipe {
            pipe,
            timeout: Cell::new(None),
        })
    }
}

/// Self-relative descriptor granting all rights to the logon SID only, under a protected DACL
/// (no inherited grant); freed on drop.
struct LogonDescriptor(PSECURITY_DESCRIPTOR);
impl LogonDescriptor {
    fn new(logon_sid: &str) -> io::Result<Self> {
        let sddl = wide(&format!("D:P(A;;GA;;;{logon_sid})"));
        let mut descriptor = std::ptr::null_mut();
        // SAFETY: terminated SDDL buffer; the output is a LocalAlloc allocation owned by Self.
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        };
        if converted == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }
    /// Byte pipe in overlapped mode, local clients only, under this descriptor.
    fn create_instance(&self, name: &[u16], first: bool) -> io::Result<HANDLE> {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        };
        let publication = if first {
            FILE_FLAG_FIRST_PIPE_INSTANCE
        } else {
            0
        };
        // SAFETY: the terminated name and the attributes outlive the call.
        let handle = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | publication,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                MAX_PIPE_INSTANCES,
                PIPE_BUFFER_BYTES,
                PIPE_BUFFER_BYTES,
                0,
                &attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }
}
impl Drop for LogonDescriptor {
    fn drop(&mut self) {
        // SAFETY: allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW, freed once.
        unsafe { LocalFree(self.0) };
    }
}

/// A pipe handle opened for overlapped I/O, with its manual-reset completion event.
struct Overlapped {
    file: File,
    event: OwnedHandle,
}
impl Overlapped {
    /// Take ownership of a new overlapped pipe `handle`.
    fn new(handle: HANDLE) -> io::Result<Self> {
        // SAFETY: a fresh valid pipe handle, owned from here on (closed on every error below).
        let file = unsafe { File::from_raw_handle(handle) };
        // SAFETY: unnamed manual-reset event, initially non-signaled, default security.
        let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        if event.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: CreateEventW returned a new handle that this value now owns.
        let event = unsafe { OwnedHandle::from_raw_handle(event) };
        Ok(Self { file, event })
    }
    /// Start one operation with `start`, then wait for it at most `wait` (`None`: unbounded).
    /// `Ok(None)` when the wait expired: the operation was cancelled and drained, so the
    /// OVERLAPPED on this frame is never referenced after return.
    fn run(
        &self,
        wait: Option<Duration>,
        start: impl FnOnce(HANDLE, *mut OVERLAPPED) -> BOOL,
    ) -> io::Result<Option<u32>> {
        let (handle, event) = (self.file.as_raw_handle(), self.event.as_raw_handle());
        // SAFETY: an all-zero OVERLAPPED is valid; it lives on this frame until the operation
        // ends (completion, or cancellation then drain).
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = event;
        // SAFETY: the owned event stays valid for this call.
        unsafe { ResetEvent(event) };
        if start(handle, &mut overlapped) == 0 {
            // SAFETY: reads the error of the start call just made on this thread.
            let error = unsafe { GetLastError() };
            if error != ERROR_IO_PENDING {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
            // SAFETY: waits on the owned event of the pending operation.
            if unsafe { WaitForSingleObject(event, milliseconds(wait)) } != WAIT_OBJECT_0 {
                // SAFETY: cancels only this pending operation; `drain` waits for its end.
                unsafe { CancelIoEx(handle, &overlapped) };
                return drain(handle, &overlapped);
            }
        }
        completed(handle, &overlapped)
    }
}
/// Result of an operation that has completed.
fn completed(handle: HANDLE, overlapped: &OVERLAPPED) -> io::Result<Option<u32>> {
    let mut transferred = 0;
    // SAFETY: the operation has completed; this only reads its result.
    if unsafe { GetOverlappedResult(handle, overlapped, &mut transferred, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(Some(transferred))
}
/// Wait for a cancelled operation to end: `Ok(Some)` if it completed meanwhile, `Ok(None)` if it
/// was aborted.
fn drain(handle: HANDLE, overlapped: &OVERLAPPED) -> io::Result<Option<u32>> {
    let mut transferred = 0;
    // SAFETY: bWait = TRUE blocks until the cancelled operation releases the OVERLAPPED.
    if unsafe { GetOverlappedResult(handle, overlapped, &mut transferred, 1) } != 0 {
        return Ok(Some(transferred));
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(ERROR_OPERATION_ABORTED as i32) {
        return Ok(None);
    }
    Err(error)
}
/// Wait bound in milliseconds; `None` and bounds beyond `u32` wait without limit.
fn milliseconds(wait: Option<Duration>) -> u32 {
    wait.and_then(|wait| u32::try_from(wait.as_millis()).ok())
        .unwrap_or(INFINITE)
}

/// Authenticated client connection on the server end; reads and writes honor `set_timeout`.
pub(crate) struct ServerPipe {
    pipe: Overlapped,
    timeout: Cell<Option<Duration>>,
}
impl ServerPipe {
    /// Bound each following read and write: an operation still pending after `timeout` is
    /// cancelled and fails with `TimedOut`.
    pub(crate) fn set_timeout(&self, timeout: Option<Duration>) {
        self.timeout.set(timeout);
    }
    fn transfer(&self, start: impl FnOnce(HANDLE, *mut OVERLAPPED) -> BOOL) -> io::Result<usize> {
        match self.pipe.run(self.timeout.get(), start)? {
            Some(transferred) => Ok(transferred as usize),
            None => Err(io::ErrorKind::TimedOut.into()),
        }
    }
}
impl Read for ServerPipe {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let length = u32::try_from(buffer.len()).unwrap_or(u32::MAX);
        let target = buffer.as_mut_ptr();
        // SAFETY: the buffer outlives the operation, which `transfer` ends before returning.
        let read = self.transfer(|handle, overlapped| unsafe {
            ReadFile(handle, target, length, std::ptr::null_mut(), overlapped)
        });
        match read {
            // The client closed its end: end of stream, as std reports for pipes.
            Err(error) if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) => Ok(0),
            other => other,
        }
    }
}
impl Write for ServerPipe {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let length = u32::try_from(buffer.len()).unwrap_or(u32::MAX);
        let source = buffer.as_ptr();
        // SAFETY: the buffer outlives the operation, which `transfer` ends before returning.
        self.transfer(|handle, overlapped| unsafe {
            WriteFile(handle, source, length, std::ptr::null_mut(), overlapped)
        })
    }
    /// Pipe writes are not buffered by this process.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
