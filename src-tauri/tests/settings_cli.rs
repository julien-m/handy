//! Process tests of the `handy settings` binary: envelope, exit codes, channels, redaction,
//! offline routing and routing to a simulated running instance, on an isolated portable store
//! (Unix: the passive legacy probe is
//! redirected through the debug-only test seam).
#![cfg(unix)]

#[path = "settings_cli/contract.rs"]
mod contract;
#[path = "settings_cli/harness.rs"]
mod harness;
#[path = "settings_cli/offline.rs"]
mod offline;
#[path = "settings_cli/routed.rs"]
mod routed;
