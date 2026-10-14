//! Custom AI prompts (feature 005): the validated prompt library and its one-time starter
//! migration, provider readiness, and the bounded AI step, shared by the settings UI commands,
//! the `handy settings prompts` domain and the dictation pipeline.
pub mod formatting;
pub mod library;
pub mod migration;
pub mod native;
pub mod processor;
pub mod provider;
pub mod test_run;
pub mod types;
pub mod view;

#[cfg(test)]
mod processing_http_tests;
