//! Smart paste (feature 007): verified accessibility insertion, then a
//! receipt-driven, multi-type clipboard paste that restores the previous
//! clipboard only after the target confirmed a read.
//!
//! Layout:
//! - pure contracts and decisions: [`ids`], [`outcome`], [`timing`],
//!   [`lifecycle`], [`settlement`];
//! - time: [`clock`], injected everywhere so boundaries are testable.
//!
//! Native effects (pasteboard, accessibility, keyboard) stay behind
//! injectable ports so the production decision code is exercised by tests
//! with fakes. Wiring into the paste pipeline waits for the destination
//! context of feature 006; settings integration waits for feature 008.

pub mod clock;
pub mod ids;
pub mod lifecycle;
pub mod outcome;
pub mod settlement;
pub mod timing;
