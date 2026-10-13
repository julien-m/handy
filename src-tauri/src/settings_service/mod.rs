//! Settings descriptors, validation, redacted wire contracts, the atomic repository and the
//! live service through which every writer of the running app commits.

mod atomic;
pub mod command_error;
pub mod descriptor;
mod descriptor_data;
mod edit;
mod effects;
pub mod errors;
pub mod live;
pub mod locator;
mod normalization;
pub mod ownership;
pub mod protocol;
pub mod public;
mod raw_merge;
mod request;
pub mod store;
pub(crate) mod tauri_adapter;
pub mod transaction;
pub mod validation;
#[cfg(windows)]
mod windows_store;
