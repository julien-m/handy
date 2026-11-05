//! Real filesystem and process contracts for atomic settings storage.

/// A child started by `driver` holds a copy of every descriptor open in this test binary between
/// its fork and its exec, a lease of another test included. A test that releases a lease and
/// takes it again at once holds this guard for reading; `Driver::start` spawns under the write
/// lock, so no half-started child keeps a released lease busy.
static FORK_GUARD: std::sync::RwLock<()> = std::sync::RwLock::new(());
#[path = "settings_transactions/driver.rs"]
mod driver;
#[path = "settings_transactions/faults.rs"]
mod faults;
#[path = "settings_transactions/locator.rs"]
mod locator;
#[path = "settings_transactions/normalized_writes.rs"]
mod normalized_writes;
#[path = "settings_transactions/ownership.rs"]
mod ownership;
#[path = "settings_transactions/processes.rs"]
mod processes;
#[path = "settings_transactions/reads.rs"]
mod reads;
#[path = "settings_transactions/substitution.rs"]
mod substitution;
#[path = "settings_transactions/transactions.rs"]
mod transactions;
