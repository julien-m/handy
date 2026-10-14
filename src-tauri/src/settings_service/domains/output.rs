//! `handy settings output`: the existing output settings (paste method, clipboard handling,
//! automatic submit, paste delays, typing tool, external script, trailing space), read and changed
//! through the same descriptors and validator as generic `get`/`set` and the settings UI, plus
//! 006's destination return (`return_to_dictation_app`, refused on a platform without the
//! destination capability). Feature 007 extends this domain (smart paste).
use super::{ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest};
use crate::settings::AppSettings;
use crate::settings_service::{
    errors::{SettingsError, SettingsErrorCode},
    validation::{validate_cli, ValidationContext},
};

/// Output settings of this build, in help order.
pub const OUTPUT_KEYS: [&str; 11] = [
    "paste_method",
    "clipboard_handling",
    "auto_submit",
    "auto_submit_key",
    "paste_delay_ms",
    "paste_delay_after_ms",
    "reliable_paste",
    "typing_tool",
    "external_script_path",
    "append_trailing_space",
    "return_to_dictation_app",
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
    })
}
