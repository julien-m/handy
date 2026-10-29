//! Refusals added by 009 (PIN mode and Enter rule) and 012 (favorites) to the settings errors:
//! the application lookup of `destination pin`, the favorite lookup of
//! `destination favorite-remove` and the message of an invalid Automatic Enter value.
use super::{SettingsError, SettingsErrorCode, ValidationReason};

// @spec AC-006: Invalid Automatic Enter lists the values — .specs/features/009-pin-mode-and-enter-rule/spec.md#ac-006
/// Business rule (009 Story 5): an invalid Automatic Enter value is refused with the three
/// allowed values in its fixed message (never the candidate); every other issue keeps the
/// reason's message.
pub(super) fn issue_message(key: &str, reason: ValidationReason) -> &'static str {
    match (key, reason) {
        ("automatic_enter", ValidationReason::CanonicalValue) => {
            "Expected never, always or pin_only."
        }
        _ => reason.message(),
    }
}

impl SettingsError {
    /// `destination pin` named no running application (009 AC-006): `not_found`, exit 4, with a
    /// message about the application lookup rather than an unknown settings key; nothing changed.
    pub fn no_running_application() -> Self {
        let mut error = Self::new(SettingsErrorCode::NotFound);
        error.message = "No running application matches.";
        error
    }
    /// `destination default-input-remove` named no remembered input (013 AC-009): `not_found`,
    /// exit 4; the list is unchanged.
    pub fn no_default_input() -> Self {
        let mut error = Self::new(SettingsErrorCode::NotFound);
        error.message = "No remembered input matches.";
        error
    }
    /// `destination favorite-remove` named no favorite (012 AC-006): `not_found`, exit 4; the
    /// favorites are unchanged.
    pub fn no_favorite() -> Self {
        let mut error = Self::new(SettingsErrorCode::NotFound);
        error.message = "No favorite application matches.";
        error
    }
}
