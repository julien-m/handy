//! The real 011 modules, with stubs of the 006 types `field.rs` imports.
/// Stub of 006's `context` (the two types `field.rs` reads).
pub mod context {
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum ApplicationIdentity {
        BundleId(String),
    }
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DictationId(pub u64);
}
/// Stub of 006's `session::Selection` (the fields `field.rs` reads).
pub mod session {
    pub struct Destination {
        pub identity: super::context::ApplicationIdentity,
        pub captured_pid: u32,
    }
    pub enum Selection {
        FrontmostAtEnd,
        Application(Destination),
    }
}
#[path = "../../../../../../../../src-tauri/src/destination/bounded_worker.rs"]
pub mod bounded_worker;
#[path = "../../../../../../../../src-tauri/src/destination/field.rs"]
pub mod field;
#[path = "../../../../../../../../src-tauri/src/destination/uia_mapping.rs"]
pub mod uia_mapping;
#[path = "../../../../../../../../src-tauri/src/destination/windows_field.rs"]
pub mod windows_field;
#[path = "../../../../../../../../src-tauri/src/destination/windows_uia.rs"]
mod windows_uia;
