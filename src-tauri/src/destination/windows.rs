//! Windows workspace adapter over Win32: foreground window → process → identity (application
//! user model ID when the process is packaged, otherwise its canonical executable path), the
//! visible unowned non-tool top-level windows as "applications with windows", and one
//! `SetForegroundWindow` request, which the system may refuse (foreground lock). Handy is
//! excluded by its own process id. No window title is read. Native behavior on Windows is
//! pending release evidence; this file is compile-checked only.
use super::context::ApplicationIdentity;
use super::workspace::{ActivationRequest, Foreground, RunningApp, WorkspacePort};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, HWND, LPARAM};
use windows_sys::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow, GWL_EXSTYLE, GW_OWNER, SW_RESTORE,
    WS_EX_TOOLWINDOW,
};

// Longest path or AUMID read: 32 767 UTF-16 units is the Win32 extended-path maximum.
const MAX_TEXT: usize = 32_767;

/// The Windows workspace of this process.
pub struct WindowsWorkspace {
    own_pid: u32,
}
impl WindowsWorkspace {
    /// Adapter excluding the current process (Handy and its panels).
    pub fn new() -> Self {
        Self {
            own_pid: std::process::id(),
        }
    }
}
impl Default for WindowsWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

/// Process id owning `window`, or `None` for a null window.
fn window_pid(window: HWND) -> Option<u32> {
    if window.is_null() {
        return None;
    }
    let mut pid = 0_u32;
    // SAFETY: `window` is a window handle from the system; `pid` outlives the call.
    unsafe { GetWindowThreadProcessId(window, &mut pid) };
    (pid != 0).then_some(pid)
}
/// A top-level window the user switches to: visible, unowned and not a tool window.
fn is_app_window(window: HWND) -> bool {
    // SAFETY: plain queries on a window handle enumerated by the system.
    unsafe {
        IsWindowVisible(window) != 0
            && GetWindow(window, GW_OWNER).is_null()
            && (GetWindowLongPtrW(window, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW) == 0
    }
}
/// Top-level application windows in Z order (front first).
fn app_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(window: HWND, state: LPARAM) -> i32 {
        // SAFETY: `state` is the address of the vector below, alive for the whole enumeration.
        let windows = unsafe { &mut *(state as *mut Vec<HWND>) };
        if is_app_window(window) {
            windows.push(window);
        }
        1
    }
    let mut windows: Vec<HWND> = Vec::new();
    // SAFETY: the callback only touches `windows`, which outlives the synchronous enumeration.
    unsafe { EnumWindows(Some(collect), &mut windows as *mut Vec<HWND> as LPARAM) };
    windows
}

/// Read a UTF-16 text through `read(buffer, length)`, which returns true on success.
fn read_text(read: impl FnOnce(*mut u16, &mut u32) -> bool) -> Option<String> {
    let mut buffer = vec![0_u16; MAX_TEXT];
    let mut length = MAX_TEXT as u32;
    if !read(buffer.as_mut_ptr(), &mut length) {
        return None;
    }
    let text = &buffer[..(length as usize).min(MAX_TEXT)];
    let text = String::from_utf16_lossy(text);
    Some(text.trim_end_matches('\0').to_owned())
}
/// Stable identity of process `pid`: its AUMID when packaged, otherwise its executable path.
fn identity_of(pid: u32) -> Option<ApplicationIdentity> {
    // SAFETY: a query-only handle, closed below on every path.
    let process: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return None;
    }
    let aumid = read_text(|buffer, length| {
        // SAFETY: `buffer` holds `length` UTF-16 units; the call writes at most that many.
        unsafe { GetApplicationUserModelId(process, length, buffer) == ERROR_SUCCESS }
    });
    let path = read_text(|buffer, length| {
        // SAFETY: same contract as above.
        unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buffer, length) != 0 }
    });
    // SAFETY: `process` was opened above and is closed exactly once.
    unsafe { CloseHandle(process) };
    match (aumid.filter(|text| !text.is_empty()), path) {
        (Some(aumid), _) => Some(ApplicationIdentity::Aumid(aumid)),
        (None, Some(path)) => Some(ApplicationIdentity::ExecutablePath(PathBuf::from(path))),
        (None, None) => None,
    }
}
/// Typed snapshot of process `pid`; the display name is the executable stem or AUMID tail.
fn snapshot(pid: u32) -> Option<RunningApp> {
    let identity = identity_of(pid)?;
    let display_name = match &identity {
        ApplicationIdentity::ExecutablePath(path) => path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default(),
        ApplicationIdentity::Aumid(text) | ApplicationIdentity::BundleId(text) => {
            text.rsplit('!').next().unwrap_or(text).to_owned()
        }
    };
    Some(RunningApp {
        pid,
        identity,
        display_name,
        icon_ref: None,
    })
}

impl WorkspacePort for WindowsWorkspace {
    // @spec FR-001: Read the foreground application identity — .specs/features/006-destination-application/spec.md#fr-001
    fn frontmost(&self) -> Foreground {
        // SAFETY: no arguments; returns a possibly null window handle.
        let window = unsafe { GetForegroundWindow() };
        match window_pid(window).map(|pid| (pid, snapshot(pid))) {
            Some((pid, _)) if pid == self.own_pid => Foreground::Handy,
            Some((_, Some(app))) => Foreground::Application(app),
            _ => Foreground::Unreadable,
        }
    }
    fn running_windowed_apps(&self) -> Vec<RunningApp> {
        let mut apps: Vec<RunningApp> = Vec::new();
        for pid in app_windows().into_iter().filter_map(window_pid) {
            // Skip Handy and a process already listed (several windows of one process).
            if pid == self.own_pid || apps.iter().any(|known| known.pid == pid) {
                continue;
            }
            if let Some(app) = snapshot(pid) {
                if !apps.iter().any(|known| known.identity == app.identity) {
                    apps.push(app);
                }
            }
        }
        apps
    }
    fn instances(&self, identity: &ApplicationIdentity) -> Vec<RunningApp> {
        let mut apps: Vec<RunningApp> = Vec::new();
        for pid in app_windows().into_iter().filter_map(window_pid) {
            if pid == self.own_pid || apps.iter().any(|known| known.pid == pid) {
                continue;
            }
            apps.extend(snapshot(pid).filter(|app| app.identity == *identity));
        }
        apps
    }
    // @spec FR-002: One activation request per insertion — .specs/features/006-destination-application/spec.md#fr-002
    /// One `SetForegroundWindow` request on the front window of the target process (restored
    /// when minimized). The system may refuse it; acceptance is not proof of foreground.
    fn activate_once(&self, target: &RunningApp) -> ActivationRequest {
        let window = app_windows()
            .into_iter()
            .find(|window| window_pid(*window) == Some(target.pid));
        let Some(window) = window else {
            return ActivationRequest::Refused;
        };
        // SAFETY: `window` was just enumerated; these calls only change its show state/focus.
        let accepted = unsafe {
            if IsIconic(window) != 0 {
                ShowWindow(window, SW_RESTORE);
            }
            SetForegroundWindow(window) != 0
        };
        if accepted {
            ActivationRequest::Sent
        } else {
            ActivationRequest::Refused
        }
    }
}
