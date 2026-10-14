//! The ordered microphone choices as one settings change: validated the same way for the
//! settings UI and the CLI, then committed as a whole list on the locked latest settings by the
//! shared transaction (one revision, no partial write). Presence of a device is runtime status,
//! never a validation rule: an unplugged choice stays valid and saved.
use crate::settings::AppSettings;
use crate::settings_service::{
    domains::{ChangeKind, DomainChange},
    errors::{ErrorItem, ErrorReason, SettingsError, ValidationReason},
};

/// Settings key of the ordered list.
pub const PRIORITY_KEY: &str = "microphone_priority";

// @spec FR-001: Unique ordered list of exact names — .specs/features/001-microphone-priority/spec.md#fr-001
/// The complete order, unchanged, when every name is a unique non-blank exact name. Names keep
/// their case and bytes; "default" is an ordinary device name.
///
/// # Errors
/// `validation_failed` on `microphone_priority`, with one `details.items` entry per refused
/// position (empty or duplicate), never echoing a name.
pub fn validate_priority(names: Vec<String>) -> Result<Vec<String>, SettingsError> {
    let refused = names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| refusal(&names[..index], name).map(|code| (index, code)))
        .map(|(index, code)| ErrorItem::new(index as u64, code))
        .collect::<Vec<_>>();
    if refused.is_empty() {
        return Ok(names);
    }
    Err(
        SettingsError::validation(PRIORITY_KEY, ValidationReason::CanonicalValue)
            .with_items(refused),
    )
}
/// Why `name` cannot follow `earlier` in the list, if it cannot.
fn refusal(earlier: &[String], name: &str) -> Option<ErrorReason> {
    if name.trim().is_empty() {
        Some(ErrorReason::MicrophoneNameEmpty)
    } else if earlier.iter().any(|previous| previous == name) {
        Some(ErrorReason::MicrophoneNameDuplicate)
    } else {
        None
    }
}

// @spec FR-007: One shared validated list change — .specs/features/001-microphone-priority/spec.md#fr-007
/// The change that sets the complete order, validated now; the edit only assigns it, so an
/// identical order is a no-op commit (no new revision).
///
/// # Errors
/// Same as `validate_priority`.
pub fn priority_change(names: Vec<String>) -> Result<DomainChange, SettingsError> {
    let order = validate_priority(names)?;
    let apply = Box::new(move |locked: &mut AppSettings| {
        locked.microphone_priority.clone_from(&order);
        Ok(())
    });
    Ok(DomainChange(ChangeKind::Apply(apply)))
}

/// Order chosen by the legacy single selector, applied to the locked `current` order: the chosen
/// device moves first and the other choices keep their order; the system default (`None`) means
/// an empty list. Backward compatibility for the selector of the settings window until the
/// priority editor replaces it (Step 6 of feature 001).
pub fn promoted(current: &[String], choice: Option<&str>) -> Vec<String> {
    let Some(choice) = choice else {
        return Vec::new();
    };
    std::iter::once(choice.to_owned())
        .chain(current.iter().filter(|name| *name != choice).cloned())
        .collect()
}
/// The legacy selector's change (see `promoted`), validated now and on the locked settings.
///
/// # Errors
/// `validation_failed` for a blank name.
pub fn selector_change(choice: Option<String>) -> Result<DomainChange, SettingsError> {
    if let Some(name) = &choice {
        validate_priority(vec![name.clone()])?;
    }
    let apply = Box::new(move |locked: &mut AppSettings| {
        let order = promoted(&locked.microphone_priority, choice.as_deref());
        locked.microphone_priority = validate_priority(order)?;
        Ok(())
    });
    Ok(DomainChange(ChangeKind::Apply(apply)))
}

#[cfg(test)]
#[path = "priority_tests.rs"]
mod tests;
