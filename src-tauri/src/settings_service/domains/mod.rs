//! Domain command framework (`handy settings <domain> <action>`): a domain decides, from the
//! current settings and the request, what to answer or which typed change to make; the framework
//! does the rest the same way for every domain: action aliases, live-only refusal, dry-run preview
//! through the commit path's own validation, and one atomic commit through the shared backend.
//! Only `output` is registered in this build; later features add their domain to `production`.
pub mod output;

use super::{
    edit::{SettingsEdit, SettingsEditFn},
    errors::{SettingsError, SettingsErrorCode},
    operations::SettingsBackend,
    protocol::Revision,
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use serde_json::{json, Value};

/// Typed edit closure a domain hands to the shared commit path.
pub type BoxedEdit = Box<dyn Fn(&mut AppSettings) -> Result<(), SettingsError> + Send + Sync>;

/// One typed change, committed atomically on the latest settings by the shared path.
pub struct DomainChange(pub ChangeKind);
/// What a change does.
pub enum ChangeKind {
    /// One scalar validated by the descriptor validator shared with the UI and `set`.
    Scalar(SettingsMutation),
    /// A whole-domain edit (bulk import, list update), validated by the domain against the
    /// locked settings; it must be pure because it may run twice (preview, then commit).
    Apply(BoxedEdit),
}
impl DomainChange {
    /// The repository edit of this change.
    pub(crate) fn edit(&self) -> SettingsEdit<'_> {
        match &self.0 {
            ChangeKind::Scalar(mutation) => SettingsEdit::Scalar(mutation.clone()),
            ChangeKind::Apply(apply) => {
                let apply: SettingsEditFn<'_> = apply.as_ref();
                SettingsEdit::Apply(apply)
            }
        }
    }
}
/// What a domain decided for one request.
pub enum DomainPlan {
    /// Answer from the current settings (get, export); nothing is written.
    Read(Value),
    /// Commit `change`; the reply shows the public value of `keys` after the commit (or, for a
    /// dry run, the preview).
    Change {
        change: DomainChange,
        keys: Vec<&'static str>,
    },
    /// Answer that only a running instance can give (live state); offline it is `unavailable`.
    LiveOnly(Value),
}
/// One request to a domain; flags are already typed.
pub struct DomainRequest<'a> {
    pub action: &'a str,
    pub arguments: &'a [String],
    pub input: Option<&'a str>,
}
/// A registered domain.
pub trait Domain: Sync {
    /// Command name (`handy settings <name>`).
    fn name(&self) -> &'static str;
    /// Canonical actions of this build (aliases are resolved before).
    fn actions(&self) -> &'static [&'static str];
    /// Decide `request` against `settings`: pure, no I/O and no live effect.
    ///
    /// # Errors
    /// `usage_error`, `not_found` or `validation_failed`, decided before anything is written.
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError>;
}

/// Domains of this build, in help order.
pub fn production() -> &'static [&'static dyn Domain] {
    &[&output::OutputDomain]
}
/// Shared verb aliases: every domain accepts `edit` for `update` and `delete` for `remove`.
pub fn canonical_action(action: &str) -> &str {
    match action {
        "edit" => "update",
        "delete" => "remove",
        other => other,
    }
}
/// Typed fields of a domain operation, as carried by the wire request.
pub struct DomainCall<'a> {
    pub domain: &'a str,
    pub action: &'a str,
    pub arguments: &'a [String],
    pub input: Option<&'a str>,
    pub dry_run: bool,
    pub expected: Option<Revision>,
}
/// Successful result: revision, data and the recovery warning of the settings read.
pub type Answer = (Revision, Value, bool);

/// Run one domain operation against `backend` with the domains of `registry`.
///
/// # Errors
/// `usage_error` for an unknown domain or action; the domain's refusals; `unavailable` for a
/// live-only answer without a running instance; every refusal of the shared commit path.
// @spec FR-005: Domain framework, dry run and atomic commit — .specs/features/008-settings-cli/spec.md#fr-005
pub fn execute(
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    registry: &[&'static dyn Domain],
) -> Result<Answer, SettingsError> {
    let (domain, action) = resolve(call, registry)?;
    let read = backend.read()?;
    let request = DomainRequest {
        action,
        arguments: call.arguments,
        input: call.input,
    };
    let plan = domain.plan(&request, &read.snapshot.settings)?;
    let answer = |data| Ok((read.snapshot.revision, data, read.warning));
    match plan {
        DomainPlan::Read(data) => answer(data),
        DomainPlan::LiveOnly(data) if backend.is_live() => answer(data),
        DomainPlan::LiveOnly(_) => Err(SettingsError::new(SettingsErrorCode::Unavailable)),
        DomainPlan::Change { change, keys } if call.dry_run => {
            let current = read.snapshot.revision;
            let preview = dry_run(&read.snapshot.settings, &change, call.expected, current)?;
            answer(json!({ "dry_run": true, "changes": public_keys(&preview, &keys)? }))
        }
        DomainPlan::Change { change, keys } => {
            let committed = backend.commit_change(&change, call.expected)?;
            let data = public_keys(&committed.snapshot.settings, &keys)?;
            Ok((committed.snapshot.revision, data, committed.warning))
        }
    }
}
/// The registered domain and canonical action of `call`, or `usage_error`.
fn resolve<'a>(
    call: &'a DomainCall<'_>,
    registry: &[&'static dyn Domain],
) -> Result<(&'static dyn Domain, &'a str), SettingsError> {
    let usage = || SettingsError::new(SettingsErrorCode::UsageError);
    let domain = registry
        .iter()
        .copied()
        .find(|domain| domain.name() == call.domain)
        .ok_or_else(usage)?;
    let action = canonical_action(call.action);
    if !domain.actions().contains(&action) {
        return Err(usage());
    }
    Ok((domain, action))
}
/// Business rule: a dry run validates exactly like the commit (same edit pipeline, same
/// compare-and-set) but writes nothing and runs no live effect.
fn dry_run(
    settings: &AppSettings,
    change: &DomainChange,
    expected: Option<Revision>,
    current: Revision,
) -> Result<AppSettings, SettingsError> {
    if expected.is_some_and(|expected| expected != current) {
        return Err(SettingsError::revision_conflict(current));
    }
    super::edit::preview(settings, &change.edit())
}
/// Public (redacted) values of `keys`.
pub fn public_keys(settings: &AppSettings, keys: &[&'static str]) -> Result<Value, SettingsError> {
    let public = serde_json::to_value(super::public::PublicSettings::from_settings(settings))
        .map_err(|_| SettingsError::new(SettingsErrorCode::InternalError))?;
    Ok(keys
        .iter()
        .map(|key| ((*key).to_owned(), public[*key].clone()))
        .collect::<serde_json::Map<_, _>>()
        .into())
}
#[cfg(test)]
#[path = "fixture_tests.rs"]
mod fixture_tests;
