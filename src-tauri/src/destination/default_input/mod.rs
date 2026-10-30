//! Default input per application (feature 013): when a dictation's destination has a remembered
//! input and its active element is not a text field, Handy focuses the remembered field (found
//! again by its accessibility fingerprint) or types into the remembered terminal through the
//! application's addressing tool. `model` holds the stored list and its changes, `fingerprint`,
//! `search` and `tree` the field type, `terminal` and `runner` the terminal type, `capture` the
//! badge's "Remember", `route` and `dispatch` the insertion.
pub mod capture;
pub mod dispatch;
#[cfg(test)]
pub(crate) mod fake_tree;
pub mod fingerprint;
mod instance;
#[cfg(target_os = "macos")]
pub mod macos_bundle;
#[cfg(target_os = "macos")]
pub mod macos_tree;
pub mod model;
#[cfg(test)]
mod orchestrator_tests;
pub mod route;
pub mod runner;
pub mod search;
pub mod services;
pub mod terminal;
pub mod tree;

/// The services of this platform: macOS reads the accessibility tree and the bundles of running
/// applications; every other platform offers nothing (spec decision `macos_only`).
pub fn native_services(inputs: services::InputsSource) -> services::DefaultInputServices {
    #[cfg(target_os = "macos")]
    return services::DefaultInputServices {
        supported: true,
        inputs,
        tree: std::sync::Arc::new(macos_tree::MacTreeAccess),
        runner: std::sync::Arc::new(runner::ProcessRunner),
        bundles: std::sync::Arc::new(macos_bundle::MacBundles),
    };
    #[cfg(not(target_os = "macos"))]
    {
        // `inputs` is unused: nothing is offered or applied on this platform.
        let _ = inputs;
        services::DefaultInputServices::none()
    }
}
