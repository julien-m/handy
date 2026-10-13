//! Smart paste (feature 007): verified accessibility insertion, then a
//! receipt-driven, multi-type clipboard paste that restores the previous
//! clipboard only after the target confirmed a read.
//!
//! Layout:
//! - pure contracts and decisions: [`ids`], [`outcome`], [`timing`],
//!   [`lifecycle`], [`settlement`];
//! - time: [`clock`], injected everywhere so boundaries are testable;
//! - verified accessibility insertion: [`ax_port`], [`direct`], and the
//!   macOS adapter `macos_ax`.
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
pub mod outcome;
pub mod settlement;
pub mod timing;
