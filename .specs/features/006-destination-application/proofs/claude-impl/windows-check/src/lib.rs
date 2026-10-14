//! Compile-only check of the production Windows destination adapter (006): the real
//! `context.rs`, `workspace.rs` and `windows.rs` are compiled as a library for
//! `x86_64-pc-windows-msvc` with the same `windows-sys` features as `src-tauri/Cargo.toml`.
//! Nothing is stubbed (the adapter's `super::` paths resolve to the sibling modules below). It
//! proves the adapter builds and passes clippy, not that it works: native Windows behavior is
//! pending release evidence.
#![deny(warnings)]
#[path = "../../../../../../../src-tauri/src/destination/context.rs"]
pub mod context;
#[path = "../../../../../../../src-tauri/src/destination/windows.rs"]
pub mod windows;
#[path = "../../../../../../../src-tauri/src/destination/workspace.rs"]
pub mod workspace;
