//! Domain command framework (`handy settings <domain> <action>`): a domain decides, from the
//! current settings and the request, what to answer or which typed change to make; the framework
//! does the rest the same way for every domain: action aliases, live-only refusal, dry-run preview
//! through the commit path's own validation, and one atomic commit through the shared backend.
//! Registered: `output` (008), `replacements` (002), `shortcuts` (004), `vocabulary` (003),
//! `prompts` (005), `microphones` (001) and, with the `destination-return` gate open only,
//! `destination` (009); later features add their domain to `production`.
#[cfg(feature = "destination-return")]
pub mod destination;
#[cfg(feature = "destination-return")]
mod destination_default_inputs;
#[cfg(feature = "destination-return")]
mod destination_favorites;
pub mod live_operation;
pub mod microphones;
pub mod output;
pub mod prompts;
pub mod replacements;
mod replacements_refusal;
pub mod shortcuts;
pub mod vocabulary;

use super::{
    edit::{SettingsEdit, SettingsEditFn},
    errors::{SettingsError, SettingsErrorCode},
    operations::SettingsBackend,
    platform_capability::PlatformCapabilities,
    protocol::Revision,
    store::SettingsRead,
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use live_operation::{LiveAnswer, LiveCall, LiveOperation};
use serde_json::{json, Value};

/// Typed edit closure a domain hands to the shared commit path.
pub type BoxedEdit = Box<dyn Fn(&mut AppSettings) -> Result<(), SettingsError> + Send + Sync>;

/// Reply of a reported change, built from the settings it produced (or would produce).
pub type ReportFn = Box<dyn Fn(&AppSettings) -> Result<Value, SettingsError> + Send + Sync>;

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
    /// dry run, the preview), or what `render` builds from those settings when it is set (for
    /// example the id of a created prompt, 005).
    Change {
        change: DomainChange,
        keys: Vec<&'static str>,
        render: Option<Render>,
    },
    /// Answer that only a running instance can give (live state); offline it is `unavailable`.
    LiveOnly(Value),
    /// Commit a change and answer with its report (see `ReportedChange`).
    Reported(ReportedChange),
    /// Operation run inside the running instance (native state, managers); offline it is
    /// `unavailable` and it never runs in the CLI process.
    Live(Box<dyn LiveOperation>),
    /// Change of descriptor `key` that only a running instance can apply, because its native
    /// side is prepared before the commit (for example the shortcut registrations): offline it is
    /// `unavailable` (`instance_required`), except a dry run, which never claims the native side.
    /// `render` builds the reply from the committed (or previewed) settings.
    LiveChange {
        key: &'static str,
        change: DomainChange,
        render: Render,
    },
}
/// A change answered with `report` of the resulting settings rather than public keys, so a
/// large collection never has to fit in one reply. When `bound`, the change is committed only at
/// the revision the plan was decided on (its report describes that read): a concurrent write is
/// then a revision conflict, never a silently different result.
pub struct ReportedChange {
    pub change: DomainChange,
    pub report: ReportFn,
    pub bound: bool,
}
/// Reply of a live change, built from the committed or previewed settings.
pub type Render = Box<dyn Fn(&AppSettings) -> Result<Value, SettingsError>>;
/// One request to a domain; flags are already typed.
pub struct DomainRequest<'a> {
    pub action: &'a str,
    pub arguments: &'a [String],
    pub input: Option<&'a str>,
    /// Capabilities of the build, so `plan` decides a platform refusal before any live routing.
    pub platform: PlatformCapabilities,
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
    &[
        &output::OutputDomain,
        &replacements::ReplacementsDomain,
        &shortcuts::ShortcutsDomain,
        &vocabulary::VocabularyDomain,
        &prompts::PromptsDomain,
        &microphones::MicrophonesDomain,
        // 009: gate closed, no `destination` domain exists (`usage_error`, exactly as on `dev`).
        #[cfg(feature = "destination-return")]
        &destination::DestinationDomain,
    ]
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
    /// Capabilities of the build (production passes `PlatformCapabilities::current()`).
    pub platform: PlatformCapabilities,
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
    let (domain, request) = resolve(call, registry)?;
    let read = backend.read()?;
    let plan = domain.plan(&request, &read.snapshot.settings)?;
    answer_plan(plan, call, backend, &read)
}
/// Answer, run or commit `plan` for `call`, from the settings `read` it was planned on.
fn answer_plan(
    plan: DomainPlan,
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    read: &SettingsRead,
) -> Result<Answer, SettingsError> {
    let answer = |data| Ok((read.snapshot.revision, data, read.warning));
    match plan {
        DomainPlan::Read(data) => answer(data),
        DomainPlan::LiveOnly(data) if backend.is_live() => answer(data),
        DomainPlan::LiveOnly(_) => Err(SettingsError::new(SettingsErrorCode::Unavailable)),
        DomainPlan::Live(operation) => run_live(operation.as_ref(), call, backend, read),
        DomainPlan::Change {
            change,
            keys,
            render,
        } => {
            let reply = (keys.as_slice(), render.as_ref());
            change_keys(call, backend, read, &change, reply)
        }
        DomainPlan::LiveChange {
            key,
            change,
            render,
        } => live_change(call, backend, read, (key, &change), &render),
        DomainPlan::Reported(change) => reported(call, backend, read, &change),
    }
}
/// Run a live operation inside the instance; offline, the same refusal as a live-only answer.
// @spec FR-006: Live operations only inside the instance — .specs/features/006-destination-application/spec.md#fr-006
fn run_live(
    operation: &dyn LiveOperation,
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    read: &SettingsRead,
) -> Result<Answer, SettingsError> {
    let context = backend
        .live_context()
        .ok_or_else(|| SettingsError::new(SettingsErrorCode::Unavailable))?;
    let live_call = LiveCall {
        dry_run: call.dry_run,
        expected: call.expected,
    };
    match operation.run(context, &live_call)? {
        LiveAnswer::Read(data) => Ok((read.snapshot.revision, data, read.warning)),
        LiveAnswer::Committed { read, data } => Ok((read.snapshot.revision, data, read.warning)),
    }
}
/// How a change is answered: the public values of `keys`, or `render` when set.
struct Reply<'a> {
    keys: &'a [&'static str],
    render: Option<&'a Render>,
}
impl Reply<'_> {
    fn data(&self, settings: &AppSettings) -> Result<Value, SettingsError> {
        match self.render {
            Some(render) => render(settings),
            None => public_keys(settings, self.keys),
        }
    }
}
/// A change answered with `reply`: previewed by a dry run, otherwise committed through the
/// shared path.
fn change_keys(
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    read: &SettingsRead,
    change: &DomainChange,
    (keys, render): (&[&'static str], Option<&Render>),
) -> Result<Answer, SettingsError> {
    let reply = Reply { keys, render };
    if call.dry_run {
        let current = read.snapshot.revision;
        let preview = dry_run(&read.snapshot.settings, change, call.expected, current)?;
        let data = json!({ "dry_run": true, "changes": reply.data(&preview)? });
        return Ok((current, data, read.warning));
    }
    let committed = backend.commit_change(change, call.expected)?;
    let data = reply.data(&committed.snapshot.settings)?;
    Ok((committed.snapshot.revision, data, committed.warning))
}
/// A live change: a dry run previews it anywhere (the native side unchecked); otherwise only a
/// running instance commits it.
fn live_change(
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    read: &SettingsRead,
    (key, change): (&'static str, &DomainChange),
    render: &Render,
) -> Result<Answer, SettingsError> {
    let current = read.snapshot.revision;
    if call.dry_run {
        let preview = dry_run(&read.snapshot.settings, change, call.expected, current)?;
        let data =
            json!({ "dry_run": true, "registration_checked": false, "changes": render(&preview)? });
        return Ok((current, data, read.warning));
    }
    if !backend.is_live() {
        return Err(SettingsError::instance_required());
    }
    let committed = backend.commit_live_change(key, change, call.expected)?;
    let data = render(&committed.snapshot.settings)?;
    Ok((committed.snapshot.revision, data, committed.warning))
}
/// A reported change: dry run (`{"dry_run": true, "changes": report}`) or commit (`report`).
fn reported(
    call: &DomainCall<'_>,
    backend: &dyn SettingsBackend,
    read: &super::store::SettingsRead,
    planned: &ReportedChange,
) -> Result<Answer, SettingsError> {
    let current = read.snapshot.revision;
    let expected = call.expected.or(planned.bound.then_some(current));
    if call.dry_run {
        let preview = dry_run(&read.snapshot.settings, &planned.change, expected, current)?;
        let data = json!({ "dry_run": true, "changes": (planned.report)(&preview)? });
        return Ok((current, data, read.warning));
    }
    let committed = backend.commit_change(&planned.change, expected)?;
    let data = (planned.report)(&committed.snapshot.settings)?;
    Ok((committed.snapshot.revision, data, committed.warning))
}
/// The registered domain and canonical action of `call`, or `usage_error`.
fn resolve<'a>(
    call: &'a DomainCall<'_>,
    registry: &[&'static dyn Domain],
) -> Result<(&'static dyn Domain, DomainRequest<'a>), SettingsError> {
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
    let request = DomainRequest {
        action,
        arguments: call.arguments,
        input: call.input,
        platform: call.platform,
    };
    Ok((domain, request))
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
#[cfg(test)]
#[path = "live_fixture_tests.rs"]
mod live_fixture_tests;
