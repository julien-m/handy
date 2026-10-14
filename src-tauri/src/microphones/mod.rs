//! Microphone priority (feature 001): the persisted ordered list of microphone choices, its
//! shared validation and transaction change (settings UI and CLI alike), the observation of the
//! present devices and the ordered, bounded resolution of the input to open.
pub mod boundary;
pub mod catalog;
pub mod effective;
pub mod native;
pub mod policy;
pub mod priority;
pub mod types;
