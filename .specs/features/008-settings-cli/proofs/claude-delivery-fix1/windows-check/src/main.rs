//! Compile-only check (delivery correction round 1) of the production Windows settings code:
//! passive legacy probe, private pipe server (overlapped instances, bounded waits, deadlines),
//! endpoint listener, and console attachment of the `windows_subsystem = "windows"` binary. The
//! IPC types the endpoint uses (Duplex, Connection, Incoming, Accept, Publish) are copied from
//! ipc.rs as a sibling module; nothing else is stubbed. Paths are relative to this file.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]
mod ipc {
    use std::io::{self, Read, Write};
    use std::time::Duration;
    pub(crate) trait Duplex: Read + Write + Send {
        fn limit(&self, timeout: Duration) -> io::Result<()>;
    }
    pub(crate) enum Connection {
        Connected(Box<dyn Duplex>),
        NotListening,
        Refused,
    }
    pub(crate) enum Incoming {
        Peer(Box<dyn Duplex>),
        Refused,
        Idle,
    }
    pub(crate) trait Accept: Send {
        fn accept(&mut self, wait: Duration) -> io::Result<Incoming>;
        fn published(&self) -> bool;
        fn close(self: Box<Self>) -> io::Result<()>;
    }
    pub(crate) trait Publish: Send + Sync {
        fn publish(&self) -> io::Result<Box<dyn Accept>>;
    }
}
#[path = "../../../../../../../src-tauri/src/settings_service/instance_probe.rs"]
mod instance_probe;
#[path = "../../../../../../../src-tauri/src/settings_service/platform_windows.rs"]
mod platform_windows;
#[path = "../../../../../../../src-tauri/src/settings_service/endpoint_windows.rs"]
mod endpoint_windows;
#[path = "../../../../../../../src-tauri/src/settings_service/console_windows.rs"]
mod console_windows;
fn main() {
    use endpoint_windows::Endpoint;
    use instance_probe::{classify_owner, ProbeScope};
    use ipc::{Connection, Incoming, Publish};
    use std::io::{Read, Write};
    use std::time::Duration;
    let _ = console_windows::attach_parent_console();
    let _ = console_windows::detach_console();
    let _ = instance_probe::inspect_legacy_windows();
    // ownership.rs (not compiled here) uses the other scopes for its isolated path probe.
    let _ = [ProbeScope::SameUser, ProbeScope::CurrentSession].map(|scope| classify_owner(Ok(false), scope));
    let Ok(endpoint) = Endpoint::for_identity(&"0".repeat(64)) else {
        return;
    };
    if let Connection::Connected(mut client) = endpoint.connect() {
        let _ = client.limit(Duration::from_secs(5));
        let _ = client.write_all(b"x");
    }
    let Ok(mut listener) = endpoint.publish() else {
        return;
    };
    match listener.accept(Duration::from_millis(200)) {
        Ok(Incoming::Peer(mut stream)) => {
            let _ = stream.limit(Duration::from_secs(5));
            let mut byte = [0; 1];
            let _ = stream.read(&mut byte);
            let _ = stream.write_all(&byte).and_then(|()| stream.flush());
        }
        Ok(Incoming::Refused | Incoming::Idle) | Err(_) => {}
    }
    let _ = listener.published();
    let _ = listener.close();
    if let Connection::NotListening | Connection::Refused = endpoint.connect() {}
}
