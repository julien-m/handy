//! A test-only domain (`probe`) whose plans are live operations, built on the production
//! framework: a live read, a live write through `LiveContext::commit_change`, a dry run that does
//! not act, a missing manager, and a platform refusal decided by the pure plan before liveness.
//! It is never registered in `production()`.
use super::live_operation::{managed_state, LiveAnswer, LiveCall, LiveContext, LiveOperation};
use super::{execute, ChangeKind, Domain, DomainCall, DomainChange, DomainPlan, DomainRequest};
use crate::settings::AppSettings;
use crate::settings_service::{
    errors::{ErrorReason, SettingsError, SettingsErrorCode},
    live::LiveSettings,
    operations::{Reply, SettingsBackend},
    ownership::LegacyPathProbe,
    platform_capability::{PlatformCapabilities, PlatformCapability},
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Counts the live operations that actually ran.
#[derive(Default)]
struct ProbeDomain {
    runs: Arc<AtomicUsize>,
}
impl Domain for ProbeDomain {
    fn name(&self) -> &'static str {
        "probe"
    }
    fn actions(&self) -> &'static [&'static str] {
        &["status", "remember", "manager", "pinned"]
    }
    fn plan(
        &self,
        request: &DomainRequest<'_>,
        _settings: &AppSettings,
    ) -> Result<DomainPlan, SettingsError> {
        // `_settings` is unused: every probe answer is computed live, inside the instance.
        let kind = match (request.action, request.arguments) {
            ("status", []) => ProbeKind::Status,
            ("remember", [word]) => ProbeKind::Remember(word.clone()),
            ("manager", []) => ProbeKind::Manager,
            ("pinned", []) => {
                let requirement = Some(PlatformCapability::Destination);
                request.platform.check(requirement, None)?;
                ProbeKind::Status
            }
            _ => return Err(SettingsError::new(SettingsErrorCode::UsageError)),
        };
        let runs = Arc::clone(&self.runs);
        Ok(DomainPlan::Live(Box::new(ProbeOperation { kind, runs })))
    }
}
enum ProbeKind {
    Status,
    Remember(String),
    Manager,
}
struct ProbeOperation {
    kind: ProbeKind,
    runs: Arc<AtomicUsize>,
}
/// A manager no test instance registers.
struct AbsentManager;
impl LiveOperation for ProbeOperation {
    fn run(&self, context: &dyn LiveContext, call: &LiveCall) -> Result<LiveAnswer, SettingsError> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        match &self.kind {
            ProbeKind::Status => {
                let words = context.read()?.snapshot.settings.custom_words.len();
                Ok(LiveAnswer::Read(json!({ "words": words })))
            }
            ProbeKind::Remember(word) if call.dry_run => {
                Ok(LiveAnswer::Read(json!({ "dry_run": true, "word": word })))
            }
            ProbeKind::Remember(word) => remember(context, word, call.expected),
            ProbeKind::Manager => {
                managed_state::<AbsentManager>(context)?;
                Ok(LiveAnswer::Read(json!({})))
            }
        }
    }
}
fn remember(
    context: &dyn LiveContext,
    word: &str,
    expected: Option<Revision>,
) -> Result<LiveAnswer, SettingsError> {
    let word = word.to_owned();
    let edit = Box::new(move |settings: &mut AppSettings| {
        settings.custom_words.push(word.clone());
        Ok(())
    });
    let read = context.commit_change(&DomainChange(ChangeKind::Apply(edit)), expected)?;
    let data = json!({ "custom_words": read.snapshot.settings.custom_words });
    Ok(LiveAnswer::Committed {
        read: Box::new(read),
        data,
    })
}

/// The running instance stood in for by an owned service, without a Tauri handle.
struct Instance<'a>(&'a LiveSettings);
impl SettingsBackend for Instance<'_> {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit(
        &self,
        m: SettingsMutation,
        e: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(self.0, m, e)
    }
    fn is_live(&self) -> bool {
        true
    }
    fn live_context(&self) -> Option<&dyn LiveContext> {
        Some(self)
    }
}
impl LiveContext for Instance<'_> {
    fn app(&self) -> Option<&tauri::AppHandle> {
        None
    }
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit_change(self.0, change, expected)
    }
}
fn service(directory: &tempfile::TempDir) -> Result<LiveSettings, SettingsError> {
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let store = directory.path().join("settings_store.json");
    LiveSettings::start_offline(SettingsRepository::new(store), Box::new(probe))
}
/// One `probe` call with `platform`, as the CLI would render it.
fn call(
    domain: &'static ProbeDomain,
    backend: &dyn SettingsBackend,
    request: (&str, &[&str], bool),
    platform: PlatformCapabilities,
) -> Reply {
    let (action, arguments, dry_run) = request;
    let arguments = arguments
        .iter()
        .map(|a| (*a).to_owned())
        .collect::<Vec<_>>();
    let call = DomainCall {
        domain: "probe",
        action,
        arguments: &arguments,
        input: None,
        dry_run,
        expected: None,
        platform,
    };
    match execute(&call, backend, &[domain]) {
        Ok((revision, data, warning)) => Reply::success(revision, data, warning),
        Err(error) => Reply::failure(error),
    }
}
fn probe() -> &'static ProbeDomain {
    Box::leak(Box::new(ProbeDomain::default()))
}
fn current() -> PlatformCapabilities {
    PlatformCapabilities::current()
}
fn code(reply: &Reply) -> (i32, Value) {
    let envelope = reply.envelope();
    (reply.exit_code(), envelope["error"]["code"].clone())
}

#[test]
fn offline_a_live_operation_is_the_live_only_refusal_and_never_runs() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let domain = probe();
    for action in [
        ("status", &[][..], false),
        ("remember", &["cli"][..], false),
    ] {
        let refused = call(domain, &backend, action, current());
        assert_eq!(code(&refused), (5, json!("unavailable")));
        assert_eq!(refused.envelope()["error"]["details"], json!({}));
    }
    assert_eq!(domain.runs.load(Ordering::SeqCst), 0);
    assert!(!directory.path().join("settings_store.json").exists());
    Ok(())
}
#[test]
fn a_live_read_answers_at_the_revision_read_before_planning_without_writing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = service(&directory)?;
    let domain = probe();
    let read = call(
        domain,
        &Instance(&service),
        ("status", &[], false),
        current(),
    );
    assert_eq!(read.envelope()["data"], json!({ "words": 0 }));
    assert_eq!(read.envelope()["revision"], json!(0));
    assert!(!directory.path().join("settings_store.json").exists());
    assert_eq!(domain.runs.load(Ordering::SeqCst), 1);
    Ok(())
}
#[test]
fn a_live_write_commits_through_the_context_at_the_next_revision() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = service(&directory)?;
    let instance = Instance(&service);
    let first = call(
        probe(),
        &instance,
        ("remember", &["handy"], false),
        current(),
    );
    let second = call(probe(), &instance, ("remember", &["cli"], false), current());
    assert_eq!(
        second.envelope()["data"]["custom_words"],
        json!(["handy", "cli"])
    );
    let revisions = [&first, &second].map(|reply| reply.envelope()["revision"].as_u64());
    assert_eq!(revisions[1], revisions[0].map(|revision| revision + 1));
    Ok(())
}
#[test]
fn a_live_dry_run_does_not_act() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = service(&directory)?;
    let preview = call(
        probe(),
        &Instance(&service),
        ("remember", &["cli"], true),
        current(),
    );
    assert_eq!(preview.envelope()["data"]["dry_run"], json!(true));
    assert!(!directory.path().join("settings_store.json").exists());
    Ok(())
}
#[test]
fn a_missing_manager_is_unavailable_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = service(&directory)?;
    let refused = call(
        probe(),
        &Instance(&service),
        ("manager", &[], false),
        current(),
    );
    assert_eq!(code(&refused), (5, json!("unavailable")));
    assert!(!directory.path().join("settings_store.json").exists());
    Ok(())
}
#[test]
fn a_platform_refusal_is_decided_by_the_plan_before_liveness() -> TestResult {
    let directory = tempfile::tempdir()?;
    let backend = service(&directory)?;
    let domain = probe();
    let none = PlatformCapabilities::only(&[]);
    let refused = call(domain, &backend, ("pinned", &[], false), none);
    assert_eq!(code(&refused), (2, json!("validation_failed")));
    let details = &refused.envelope()["error"]["details"];
    assert_eq!(details["reason"], json!(ErrorReason::UnsupportedPlatform));
    assert_eq!(domain.runs.load(Ordering::SeqCst), 0);
    let destination = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    let offline = call(domain, &backend, ("pinned", &[], false), destination);
    assert_eq!(code(&offline), (5, json!("unavailable")));
    let answered = call(
        domain,
        &Instance(&backend),
        ("pinned", &[], false),
        destination,
    );
    assert_eq!(answered.exit_code(), 0);
    Ok(())
}
