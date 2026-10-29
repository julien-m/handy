//! `handy settings output`: the existing output settings (paste method, clipboard handling,
//! automatic submit, paste delays, typing tool, external script, trailing space), read and changed
//! through the same descriptors and validator as generic `get`/`set` and the settings UI (with the
//! `destination-return` gate open, 009's `automatic_enter` replaces `auto_submit`), plus
//! 006's destination return (`return_to_dictation_app`, refused on a platform without the
//! destination capability). Feature 007 adds `paste_method smart_paste` through that shared
//! validation: accepted only where `smart_paste::setting_policy` makes it selectable, refused
//! elsewhere with `validation_failed` (exit 2) before any write, dry runs included. Feature
//! 010 adds `show_recovery_bubble_without_text_field` with the gate open (refused off macOS).
//! 007 AC-019 adds the experimental boolean `paste_verification` (smart paste only).
use super::{ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest};
use crate::settings::AppSettings;
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode},
    validation::{validate_cli, ValidationContext},
};

/// The automatic Enter key of this build: `auto_submit` as on `dev` with the `destination-return`
/// gate closed, `automatic_enter` (009) with it open.
#[cfg(not(feature = "destination-return"))]
const ENTER_KEY: &str = "auto_submit";
#[cfg(feature = "destination-return")]
const ENTER_KEY: &str = "automatic_enter";

/// Output settings of this build, in help order (010 adds its no-text-field key when the
/// `destination-return` gate is open).
#[cfg(not(feature = "destination-return"))]
pub const OUTPUT_KEYS: [&str; 12] = [
    "paste_method",
    "clipboard_handling",
    ENTER_KEY,
    "auto_submit_key",
    "paste_delay_ms",
    "paste_delay_after_ms",
    "reliable_paste",
    "paste_verification",
    "typing_tool",
    "external_script_path",
    "append_trailing_space",
    "return_to_dictation_app",
];
/// Output settings of this build, in help order.
#[cfg(feature = "destination-return")]
pub const OUTPUT_KEYS: [&str; 13] = [
    "paste_method",
    "clipboard_handling",
    ENTER_KEY,
    "auto_submit_key",
    "paste_delay_ms",
    "paste_delay_after_ms",
    "reliable_paste",
    "paste_verification",
    "typing_tool",
    "external_script_path",
    "append_trailing_space",
    "return_to_dictation_app",
    "show_recovery_bubble_without_text_field",
];

/// The output domain.
pub struct OutputDomain;
impl Domain for OutputDomain {
    fn name(&self) -> &'static str {
        "output"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["get", "set"]
    }
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        match (request.action, request.arguments) {
            ("get", []) => Ok(DomainPlan::Read(super::public_keys(
                settings,
                &OUTPUT_KEYS,
            )?)),
            ("get", [key]) => Ok(DomainPlan::Read(super::public_keys(
                settings,
                &[output_key(key)?],
            )?)),
            ("set", [key, value]) => set(request, settings, output_key(key)?, value),
            _ => Err(SettingsError::new(SettingsErrorCode::UsageError)),
        }
    }
}
/// A key of this domain; other keys (settings of other domains included) are not found here.
fn output_key(key: &str) -> Result<&'static str, SettingsError> {
    OUTPUT_KEYS
        .iter()
        .copied()
        .find(|known| *known == key)
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))
}
/// Same validator as `handy settings set` (prompt membership and build policy from `settings`,
/// platform capabilities from the request).
fn set(
    request: &DomainRequest<'_>,
    settings: &AppSettings,
    key: &'static str,
    value: &str,
) -> Result<DomainPlan, SettingsError> {
    let prompt_ids = settings
        .post_process_prompts
        .iter()
        .map(|prompt| prompt.id.as_str())
        .collect::<Vec<_>>();
    let context = ValidationContext {
        prompt_ids: &prompt_ids,
        updates_allowed: !crate::settings::update_checks_forced_disabled(),
        platform: request.platform,
    };
    let mutation = validate_cli(key, value, &context)?;
    Ok(DomainPlan::Change {
        change: DomainChange(ChangeKind::Scalar(mutation)),
        keys: vec![key],
        render: None,
    })
}
