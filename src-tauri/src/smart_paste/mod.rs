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
//!   [`snapshot_protocol`], [`pasteboard_types`], [`pasteboard_port`], and
//!   on macOS the `snapshot_helper` process (reading through
//!   `macos_snapshot_stream`, kept started ahead by `warm_helper`),
//!   `macos_pasteboard` and `main_thread`;
//! - per-application strategy and keyboard: [`strategy`], [`keyboard`],
//!   [`key_events`], and the macOS adapters in `macos_keyboard`;
//! - the serialized insertion chain and clipboard lifecycle:
//!   [`coordinator`] with its [`request`] ports, owned by one [`actor`] thread.
//!
//! Native effects (pasteboard, accessibility, keyboard) stay behind
//! injectable ports so the production decision code is exercised by tests
//! with fakes. Wiring into the paste pipeline waits for the destination
//! context of feature 006; settings integration waits for feature 008.

pub mod actor;
pub mod ax_port;
pub mod clock;
pub mod coordinator;
pub mod direct;
pub mod ids;
pub mod key_events;
pub mod keyboard;
pub mod lifecycle;
#[cfg(target_os = "macos")]
pub mod macos_ax;
#[cfg(target_os = "macos")]
mod macos_cf;
#[cfg(target_os = "macos")]
pub mod macos_keyboard;
#[cfg(target_os = "macos")]
pub mod macos_pasteboard;
#[cfg(target_os = "macos")]
mod macos_snapshot_stream;
#[cfg(target_os = "macos")]
pub mod main_thread;
pub mod outcome;
pub mod pasteboard_port;
pub mod pasteboard_types;
mod pending;
pub mod request;
pub mod settlement;
pub mod snapshot;
#[cfg(target_os = "macos")]
pub mod snapshot_helper;
pub mod snapshot_protocol;
pub mod strategy;
pub mod timing;
#[cfg(target_os = "macos")]
mod warm_helper;
