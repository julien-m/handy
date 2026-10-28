//! Compile-only check (Step 5) of the production Windows settings endpoint and pipe primitives,
//! with the IPC types it uses copied from ipc.rs (Duplex, Connection) and no other stub.
mod settings_service {
    pub mod ipc {
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
    }
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/instance_probe.rs"]
    pub mod instance_probe;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/platform_windows.rs"]
    pub mod platform_windows;
    #[path = "/Users/julienm/projects/handy/src-tauri/src/settings_service/endpoint_windows.rs"]
    pub mod endpoint_windows;
}
fn main() {
    use settings_service::{endpoint_windows::Endpoint, ipc::Connection};
    if let Ok(endpoint) = Endpoint::for_identity(&"0".repeat(64)) {
        if let Connection::Connected(_) = endpoint.connect() {}
        if let Ok(listener) = endpoint.listen() {
            let _ = listener.accept();
            let _ = listener.close();
        }
    }
}
