//! Remembered default inputs (feature 013): one per application identity, kept in the setting
//! `destination_default_inputs`. A target is either an accessibility fingerprint of a text field
//! or a terminal addressed by a command-line tool shipped in the application's bundle. The badge,
//! the settings window and the settings CLI change the list only through the changes built here,
//! so one validation and one revisioned settings commit serve every surface.
use crate::destination::context::ApplicationIdentity;
use crate::destination::favorites::identity_value;
use crate::settings::AppSettings;
use crate::settings_service::domains::{ChangeKind, DomainChange};
use crate::settings_service::errors::{SettingsError, ValidationReason};
use serde::{Deserialize, Serialize};

/// Settings key of the remembered inputs.
pub const DEFAULT_INPUTS_KEY: &str = "destination_default_inputs";
/// Most remembered inputs (one per application): a bound for the settings list and the store
/// (spec decision `cap_twenty`).
pub const MAX_DEFAULT_INPUTS: usize = 20;
/// Longest stored string, in characters: accessibility names, placeholders, DOM ids and
/// workspace names longer than this are not identities a user would remember.
pub const MAX_TEXT_CHARS: usize = 200;

/// A command-line tool that addresses the terminals of its application (spec decision
/// `capability_by_bundle`): adding a type of application means adding a variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AddressingTool {
    /// cmux (Ghostty-based terminal): `Contents/Resources/bin/cmux`.
    Cmux,
}
impl AddressingTool {
    /// Every known tool, probed in this order.
    pub const ALL: [Self; 1] = [Self::Cmux];
    /// Path of the tool relative to the application bundle.
    pub fn bundle_relative_path(self) -> &'static str {
        match self {
            Self::Cmux => "Contents/Resources/bin/cmux",
        }
    }
}

/// Accessibility fingerprint of a text field: never its value, selected text or DOM classes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct FieldFingerprint {
    pub role: String,
    pub subrole: Option<String>,
    /// `AXTitle`, else `AXDescription`.
    pub name: Option<String>,
    /// `AXPlaceholderValue`.
    pub placeholder: Option<String>,
    /// `AXDOMIdentifier`, only when it does not look generated.
    pub dom_id: Option<String>,
}

/// A terminal of an application with an addressing tool: the workspace UUID (spec decision
/// `terminal_identity_workspace`), never a `workspace:N` reference.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct TerminalTarget {
    pub tool: AddressingTool,
    pub workspace_id: String,
    pub workspace_name: String,
}

/// Where Handy pastes for an application whose active element is not a text field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DefaultTarget {
    Field(FieldFingerprint),
    Terminal(TerminalTarget),
}

/// The remembered input of one application.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DefaultInput {
    pub identity: ApplicationIdentity,
    pub display_name: String,
    pub target: DefaultTarget,
}

// @spec FR-004: One shared validation for every writer — .specs/features/013-default-input-per-app/spec.md#fr-004
/// The list unchanged when it holds at most [`MAX_DEFAULT_INPUTS`] entries with unique, non-blank
/// identities and valid names and targets.
///
/// # Errors
/// `validation_failed` on `destination_default_inputs`: `default_inputs_full` above the cap,
/// `default_input_invalid` for a duplicate identity or an invalid entry.
pub fn validate_default_inputs(inputs: &[DefaultInput]) -> Result<(), SettingsError> {
    if inputs.len() > MAX_DEFAULT_INPUTS {
        return Err(refusal(ValidationReason::DefaultInputsFull));
    }
    for (index, input) in inputs.iter().enumerate() {
        let duplicate = inputs[..index]
            .iter()
            .any(|earlier| earlier.identity == input.identity);
        if duplicate || !is_valid(input) {
            return Err(refusal(ValidationReason::DefaultInputInvalid));
        }
    }
    Ok(())
}

fn is_valid(input: &DefaultInput) -> bool {
    !identity_value(&input.identity).trim().is_empty()
        && is_text(&input.display_name)
        && match &input.target {
            DefaultTarget::Field(field) => is_valid_field(field),
            DefaultTarget::Terminal(terminal) => {
                super::terminal::is_handle(&terminal.workspace_id)
                    && is_text(&terminal.workspace_name)
            }
        }
}

/// Business rule (spec AC-002): a fingerprint needs a role and at least one of a name, a
/// placeholder or a DOM id, or it could not be told apart from the other fields of its role.
fn is_valid_field(field: &FieldFingerprint) -> bool {
    let distinctive = [&field.name, &field.placeholder, &field.dom_id]
        .iter()
        .any(|value| value.as_deref().is_some_and(is_text));
    is_text(&field.role)
        && distinctive
        && [
            &field.subrole,
            &field.name,
            &field.placeholder,
            &field.dom_id,
        ]
        .iter()
        .all(|value| optional_ok(value.as_ref()))
}

fn is_text(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS
}

fn optional_ok(value: Option<&String>) -> bool {
    value.is_none_or(|value| value.chars().count() <= MAX_TEXT_CHARS)
}

fn refusal(reason: ValidationReason) -> SettingsError {
    SettingsError::validation(DEFAULT_INPUTS_KEY, reason)
}

/// Business rule (spec AC-004): remembering an application that already has an input replaces
/// it in place; a new application is appended.
///
/// # Errors
/// The refusals of [`validate_default_inputs`] for the resulting list (a 21st application).
pub fn remembered(
    inputs: &[DefaultInput],
    input: &DefaultInput,
) -> Result<Vec<DefaultInput>, SettingsError> {
    let mut next = inputs.to_vec();
    match next
        .iter()
        .position(|known| known.identity == input.identity)
    {
        Some(position) => next[position] = input.clone(),
        None => next.push(input.clone()),
    }
    validate_default_inputs(&next)?;
    Ok(next)
}

/// The entry of the application `identity`, if one is remembered.
pub fn input_for<'a>(
    inputs: &'a [DefaultInput],
    identity: &ApplicationIdentity,
) -> Option<&'a DefaultInput> {
    inputs.iter().find(|input| &input.identity == identity)
}

/// Position of the entry whose identity value is `identifier` (any identity kind).
pub fn find_identifier(inputs: &[DefaultInput], identifier: &str) -> Option<usize> {
    inputs
        .iter()
        .position(|input| identity_value(&input.identity) == identifier)
}

/// Settings change that remembers `input` on the locked latest settings.
pub fn remember_change(input: DefaultInput) -> DomainChange {
    let apply = Box::new(move |locked: &mut AppSettings| {
        locked.destination_default_inputs = remembered(&locked.destination_default_inputs, &input)?;
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

/// Settings change that forgets the entry named `identifier`, decided on the locked settings.
///
/// # Errors
/// At commit, `not_found` when no entry has this identity value.
pub fn remove_change(identifier: String) -> DomainChange {
    let apply = Box::new(move |locked: &mut AppSettings| {
        let position = find_identifier(&locked.destination_default_inputs, &identifier)
            .ok_or_else(SettingsError::no_default_input)?;
        locked.destination_default_inputs.remove(position);
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

/// Settings change that forgets every remembered input (a no-op on an empty list).
pub fn clear_change() -> DomainChange {
    let apply = Box::new(|locked: &mut AppSettings| {
        locked.destination_default_inputs.clear();
        Ok(())
    });
    DomainChange(ChangeKind::Apply(apply))
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
