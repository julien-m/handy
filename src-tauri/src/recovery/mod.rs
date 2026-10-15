//! Recovery bubble (feature 010), behind 006's `destination-return` gate: when the destination
//! is unavailable or uncertain, no text field is focused (macOS, setting on) or the insertion
//! method reports failure, the transcript stays in a small nonactivating bubble in place of the
//! live indicator, with Copy, Close, an explanation and, on macOS, a native drag source.
//! Gate closed, nothing here is installed and `dev`'s `paste-error` path runs unchanged.
pub mod app;
pub mod classifier;
pub mod decision;
pub mod instance;
pub mod manager;
pub mod platform;
pub mod ports;
pub mod service;
pub mod setting;
pub mod surface;
pub mod types;
pub mod verdict;
