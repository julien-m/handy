//! "Remember" of feature 013 (spec AC-001 to AC-003): which entry the badge menu offers for the
//! destination's running instance, and the capture of its focused field's fingerprint or of its
//! focused terminal. The answers carry the application name and the kind only.
use super::fingerprint::{fingerprint_of, FingerprintRefusal};
use super::model::{input_for, DefaultInput, DefaultTarget};
use super::services::DefaultInputServices;
use super::terminal;
use crate::destination::context::ApplicationIdentity;
use crate::destination::field::MIN_AX_TIMEOUT;
use crate::destination::workspace::Clock;
use crate::recovery::classifier::{classify_focused, Deadline, FocusClassification, Undetermined};
use serde::Serialize;
use std::time::Duration;

/// Bound of the field capture (spec AC-002): a user action, off the dictation's hot path.
pub const CAPTURE_BUDGET: Duration = Duration::from_millis(300);

/// The destination's running instance a "Remember" applies to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestinationInstance {
    pub identity: ApplicationIdentity,
    pub display_name: String,
    pub pid: u32,
}

/// Kind of remembered target the menu offers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DefaultInputKind {
    Field,
    Terminal,
}

/// The "Remember" entry of the badge menu.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
pub struct RememberOffer {
    pub kind: DefaultInputKind,
    pub app_name: String,
    /// The application already has a remembered input (the entry says "again").
    pub remembered: bool,
}

/// Why nothing was remembered; the list is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum RememberRefusal {
    /// Another dictation or version, or no application destination.
    Stale,
    /// The focused element is not an editable text field of the instance.
    NotTextField,
    /// The focused element is a password field.
    PasswordField,
    /// No name, placeholder or stable DOM id.
    Unrecognizable,
    /// The addressing tool did not answer usably.
    TerminalUnreadable,
    /// 20 applications already have an input.
    Full,
    /// The settings write failed.
    Failed,
}

/// Answer of "Remember".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum RememberAnswer {
    Saved(RememberOffer),
    Refused(RememberRefusal),
}

// @spec FR-001: One Remember entry per destination type — .specs/features/013-default-input-per-app/spec.md#fr-001
/// The entry offered for `instance`: a terminal when its bundle holds a known addressing tool,
/// otherwise a field; none where the feature is not offered.
pub fn offer(
    services: &DefaultInputServices,
    instance: &DestinationInstance,
) -> Option<RememberOffer> {
    if !services.supported {
        return None;
    }
    let kind = match services.tool_for(instance.pid) {
        Some(_) => DefaultInputKind::Terminal,
        None => DefaultInputKind::Field,
    };
    let remembered = input_for(&services.remembered(), &instance.identity).is_some();
    Some(RememberOffer {
        kind,
        app_name: instance.display_name.clone(),
        remembered,
    })
}

/// The input to remember for `instance`, by its kind.
///
/// # Errors
/// The refusal shown in the menu; nothing is saved.
pub fn capture(
    services: &DefaultInputServices,
    instance: &DestinationInstance,
    clock: &dyn Clock,
) -> Result<DefaultInput, RememberRefusal> {
    if !services.supported {
        return Err(RememberRefusal::Stale);
    }
    let target = match services.tool_for(instance.pid) {
        Some((tool, cli)) => terminal::capture(services.runner.as_ref(), tool, &cli)
            .map(DefaultTarget::Terminal)
            .map_err(|error| {
                log::debug!("default_input terminal capture failed: {error:?}");
                RememberRefusal::TerminalUnreadable
            })?,
        None => DefaultTarget::Field(capture_field(services, instance.pid, clock)?),
    };
    Ok(DefaultInput {
        identity: instance.identity.clone(),
        display_name: instance.display_name.clone(),
        target,
    })
}

// @spec FR-002: Fingerprint without content — .specs/features/013-default-input-per-app/spec.md#fr-002
/// The fingerprint of the focused element of `pid`: an editable, non-password element of that
/// process (010's rule), read within `CAPTURE_BUDGET`.
fn capture_field(
    services: &DefaultInputServices,
    pid: u32,
    clock: &dyn Clock,
) -> Result<super::model::FieldFingerprint, RememberRefusal> {
    let access = services.tree.as_ref();
    if !access.is_trusted() {
        return Err(RememberRefusal::NotTextField);
    }
    let deadline = Deadline::new(clock, CAPTURE_BUDGET, MIN_AX_TIMEOUT);
    let timeout = deadline
        .remaining()
        .map_err(|_| RememberRefusal::NotTextField)?;
    let mut element = access
        .focused_element(pid, timeout)
        .map_err(|_| RememberRefusal::NotTextField)?;
    match classify_focused(element.as_mut(), pid, &deadline) {
        Ok(FocusClassification::Editable) => {}
        Err(Undetermined::SecureField) => return Err(RememberRefusal::PasswordField),
        _ => return Err(RememberRefusal::NotTextField),
    }
    let timeout = deadline
        .remaining()
        .map_err(|_| RememberRefusal::NotTextField)?;
    let read = element
        .read(timeout)
        .map_err(|_| RememberRefusal::NotTextField)?;
    fingerprint_of(&read.facts).map_err(|refusal| match refusal {
        FingerprintRefusal::Password => RememberRefusal::PasswordField,
        FingerprintRefusal::Unrecognizable => RememberRefusal::Unrecognizable,
    })
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod tests;
