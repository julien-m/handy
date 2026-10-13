//! Smart paste (feature 007): verified accessibility insertion, then a
//! receipt-driven, multi-type clipboard paste that restores the previous
//! clipboard only after the target confirmed a read.
//!
//! Layout:
//! - pure contracts and decisions: [`ids`], [`outcome`], [`timing`],
//!   [`lifecycle`], [`settlement`];
//! - time: [`clock`], injected everywhere so boundaries are testable;
//! - verified accessibility insertion: [`ax_port`], [`direct`], and the
//!   macOS adapter `macos_ax`;
//! - bounded multi-type snapshot and on-demand publication: [`snapshot`],
//!   [`snapshot_protocol`], [`pasteboard_port`], and on macOS the
//!   `snapshot_helper` process (reading through `macos_snapshot_stream`),
//!   `macos_pasteboard` and `main_thread`.
//!
//! Native effects (pasteboard, accessibility, keyboard) stay behind
//! injectable ports so the production decision code is exercised by tests
//! with fakes. Wiring into the paste pipeline waits for the destination
//! context of feature 006; settings integration waits for feature 008.

pub mod ax_port;
pub mod clock;
pub mod direct;
pub mod ids;
pub mod lifecycle;
#[cfg(target_os = "macos")]
pub mod macos_ax;
#[cfg(target_os = "macos")]
pub mod macos_pasteboard;
#[cfg(target_os = "macos")]
mod macos_snapshot_stream;
#[cfg(target_os = "macos")]
pub mod main_thread;
pub mod outcome;
pub mod pasteboard_port;
pub mod settlement;
pub mod snapshot;
#[cfg(target_os = "macos")]
pub mod snapshot_helper;
pub mod snapshot_protocol;
pub mod timing;
