//! `handy settings replacements` through the real binary on an isolated portable store: the
//! Story 5 scenarios (import dry run, invalid add), every action and alias with `--json`, file
//! and stdin input, export to stdout and to a file, the SC-002 export then replace-import round
//! trip for 1, 100 and 1000 rules, stale revisions, the same operations routed to a running
//! instance (an owned live service serving the production private endpoint), and rule sets and
//! imports larger than one 1 MiB channel message.
#![cfg(unix)]

#[path = "replacement_cli/exchange.rs"]
mod exchange;
#[path = "settings_cli/harness.rs"]
mod harness;
#[path = "replacement_cli/operations.rs"]
mod operations;
#[path = "replacement_cli/routed.rs"]
mod routed;
#[path = "replacement_cli/support.rs"]
mod support;
#[path = "replacement_cli/transfer.rs"]
mod transfer;
