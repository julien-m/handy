//! Real filesystem and process contracts for atomic settings storage.
#[path = "settings_transactions/driver.rs"]
mod driver;
#[path = "settings_transactions/faults.rs"]
mod faults;
#[path = "settings_transactions/locator.rs"]
mod locator;
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
