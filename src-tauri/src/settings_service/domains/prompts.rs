//! `handy settings prompts` (feature 005): list the prompt library with each prompt's shortcuts
//! and provider readiness (a live operation: Apple Intelligence availability is only checked by
//! the running instance), and create, update, remove, set-default and set-provider-override
//! with the same `prompts::library` rules as the settings UI. Mutations work offline; routed to
//! a running Handy, removing a prompt with shortcuts releases them before the commit.
//!
//! Wire arguments (built by `settings_service::cli`): `create NAME [TEXT]`, `update ID
//! [name=NAME] [text=TEXT]`, `remove ID`, `set-default ID|none`, `set-provider-override ID
//! provider=P model=M` or `ID clear=true`. A text read from stdin (`--text -`) is the request
//! input instead of an argument.
use super::live_operation::{LiveAnswer, LiveCall, LiveContext, LiveOperation};
use super::{ChangeKind, Domain, DomainChange, DomainPlan, DomainRequest, Render};
use crate::prompts::provider::{AvailabilityProbe, NativeAvailability};
use crate::prompts::{library, view};
use crate::settings::{AppSettings, PromptProviderOverride};
use crate::settings_service::errors::{SettingsError, SettingsErrorCode};
use crate::shortcut::prompt_targets::shortcuts_of;
use serde_json::json;
use std::collections::BTreeMap;

/// The prompts domain.
pub struct PromptsDomain;
impl Domain for PromptsDomain {
    fn name(&self) -> &'static str {
        "prompts"
    }
    // `delete` and `edit` reach `remove` and `update` through `canonical_action`.
    fn actions(&self) -> &'static [&'static str] {
        &[
            "list",
            "create",
            "update",
            "remove",
            "set-default",
            "set-provider-override",
        ]
    }
    // @spec FR-006: Prompts CLI list, create, update, delete, default, override — .specs/features/005-custom-ai-prompts/spec.md#fr-006
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        match (request.action, request.arguments) {
            ("list", []) => Ok(DomainPlan::Live(Box::new(PromptsList::native()))),
            ("create", [name, rest @ ..]) => create(name, text_of(rest, request.input)?),
            ("update", [id, rest @ ..]) => update(settings, id, rest, request.input),
            ("remove", [id]) => remove(settings, id),
            ("set-default", [id]) => set_default(id),
            ("set-provider-override", [id, rest @ ..]) => set_override(request, id, rest),
            _ => Err(usage()),
        }
    }
}

fn usage() -> SettingsError {
    SettingsError::new(SettingsErrorCode::UsageError)
}
/// The text of `create`: the second positional argument, or the request input (stdin).
fn text_of(rest: &[String], input: Option<&str>) -> Result<String, SettingsError> {
    match (rest, input) {
        ([text], None) => Ok(text.clone()),
        ([], Some(input)) => Ok(input.to_owned()),
        _ => Err(usage()),
    }
}
/// `key=value` arguments; an unknown, repeated or malformed one is `usage_error`.
fn named<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<BTreeMap<&'a str, &'a str>, SettingsError> {
    let mut values = BTreeMap::new();
    for argument in arguments {
        let (key, value) = argument.split_once('=').ok_or_else(usage)?;
        if !allowed.contains(&key) || values.insert(key, value).is_some() {
            return Err(usage());
        }
    }
    Ok(values)
}
/// A change of the library answered by `render` (the commit's, or the dry run's, settings).
fn change(apply: super::BoxedEdit, render: Render) -> Result<DomainPlan, SettingsError> {
    Ok(DomainPlan::Change {
        change: DomainChange(ChangeKind::Apply(apply)),
        keys: vec![crate::prompts::types::PROMPTS_KEY],
        render: Some(render),
    })
}
/// `{prompt: view}` of prompt `id` in the answered settings.
fn prompt_reply(id: String) -> Render {
    Box::new(move |settings: &AppSettings| {
        let prompt = library::find(settings, &id)
            .ok_or_else(|| SettingsError::from(crate::prompts::types::PromptError::NotFound))?;
        Ok(json!({ "prompt": view::prompt_view(settings, prompt) }))
    })
}
fn create(name: &str, text: String) -> Result<DomainPlan, SettingsError> {
    // Generated once here, outside the edit, which may run twice.
    let id = library::generate_id();
    let (name, created) = (name.to_owned(), id.clone());
    let apply = Box::new(move |settings: &mut AppSettings| {
        library::create(settings, &id, &name, &text).map_err(SettingsError::from)
    });
    change(apply, prompt_reply(created))
}
fn update(
    settings: &AppSettings,
    id: &str,
    rest: &[String],
    input: Option<&str>,
) -> Result<DomainPlan, SettingsError> {
    let values = named(rest, &["name", "text"])?;
    let name = values.get("name").map(|name| (*name).to_owned());
    let text = match (values.get("text"), input) {
        (Some(text), None) => Some((*text).to_owned()),
        (None, Some(input)) => Some(input.to_owned()),
        (None, None) if name.is_some() => None,
        _ => return Err(usage()),
    };
    library::find(settings, id).ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))?;
    let target = id.to_owned();
    let apply = Box::new(move |settings: &mut AppSettings| {
        library::update(settings, &target, name.as_deref(), text.as_deref())
            .map_err(SettingsError::from)
    });
    change(apply, prompt_reply(id.to_owned()))
}
/// Delete is explicit in the CLI, so it is its own confirmation; the reply names the removed
/// shortcuts.
fn remove(settings: &AppSettings, id: &str) -> Result<DomainPlan, SettingsError> {
    library::find(settings, id).ok_or_else(|| SettingsError::new(SettingsErrorCode::NotFound))?;
    let removed_shortcuts = shortcuts_of(settings, id);
    let target = id.to_owned();
    let apply = Box::new(move |settings: &mut AppSettings| {
        library::remove(settings, &target).map_err(SettingsError::from)
    });
    let removed = id.to_owned();
    let render: Render = Box::new(move |settings: &AppSettings| {
        Ok(json!({
            "removed": removed,
            "removed_shortcuts": removed_shortcuts,
            "default_prompt_id": settings.post_process_selected_prompt_id,
        }))
    });
    change(apply, render)
}
fn set_default(id: &str) -> Result<DomainPlan, SettingsError> {
    let choice = (id != "none").then(|| id.to_owned());
    let apply = Box::new(move |settings: &mut AppSettings| {
        library::set_default(settings, choice.as_deref()).map_err(SettingsError::from)
    });
    let render: Render = Box::new(|settings: &AppSettings| {
        Ok(json!({ "default_prompt_id": settings.post_process_selected_prompt_id }))
    });
    change(apply, render)
}
fn set_override(
    request: &DomainRequest<'_>,
    id: &str,
    rest: &[String],
) -> Result<DomainPlan, SettingsError> {
    let values = named(rest, &["provider", "model", "clear"])?;
    let choice = match (
        values.get("provider"),
        values.get("model"),
        values.get("clear"),
    ) {
        (Some(provider), Some(model), None) => Some(PromptProviderOverride {
            provider_id: (*provider).to_owned(),
            model: (*model).to_owned(),
        }),
        (None, None, Some(&"true")) => None,
        _ => return Err(usage()),
    };
    let (target, platform) = (id.to_owned(), request.platform);
    let apply = Box::new(move |settings: &mut AppSettings| {
        library::set_override(settings, &target, choice.clone(), platform)
            .map_err(SettingsError::from)
    });
    change(apply, prompt_reply(id.to_owned()))
}

/// `prompts list` inside the running instance: the library with shortcuts and readiness.
pub struct PromptsList {
    probe: Box<dyn AvailabilityProbe>,
}
impl PromptsList {
    /// The list with the device's own availability check.
    pub fn native() -> Self {
        Self::with_probe(Box::new(NativeAvailability))
    }
    /// The list with an injected availability check (tests).
    pub fn with_probe(probe: Box<dyn AvailabilityProbe>) -> Self {
        Self { probe }
    }
}
impl LiveOperation for PromptsList {
    // `call` carries no flag a read uses.
    fn run(
        &self,
        context: &dyn LiveContext,
        _call: &LiveCall,
    ) -> Result<LiveAnswer, SettingsError> {
        let read = context.read()?;
        let data = view::listed(&read.snapshot.settings, self.probe.as_ref())?;
        Ok(LiveAnswer::Read(data))
    }
}
#[cfg(test)]
#[path = "prompts_tests.rs"]
mod tests;
