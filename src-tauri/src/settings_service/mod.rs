//! Settings descriptors, validation, redacted wire contracts, the atomic repository and the
//! live service through which every writer of the running app commits, and the early CLI path.

mod atomic;
pub mod cli;
pub mod command_error;
pub mod descriptor;
mod descriptor_data;
mod edit;
mod effects;
pub mod errors;
pub mod live;
pub mod locator;
mod normalization;
pub mod operations;
pub mod ownership;
pub mod protocol;
pub mod public;
mod raw_merge;
pub mod render;
mod request;
pub mod route;
pub mod store;
pub(crate) mod tauri_adapter;
pub mod transaction;
pub mod validation;
#[cfg(windows)]
mod windows_store;
