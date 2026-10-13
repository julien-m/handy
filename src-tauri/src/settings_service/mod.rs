//! Pure settings descriptors, validation and redacted wire contracts.

mod atomic;
pub mod descriptor;
mod descriptor_data;
pub mod errors;
pub mod locator;
mod normalization;
pub mod ownership;
pub mod protocol;
pub mod public;
mod raw_merge;
mod request;
pub mod store;
pub mod transaction;
pub mod validation;
#[cfg(windows)]
mod windows_store;
