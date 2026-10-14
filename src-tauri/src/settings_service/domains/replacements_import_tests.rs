//! Imports of the replacements domain: line-status reports, refusals with every line issue,
//! the commit bound to the previewed revision, the 1,000,000-byte bound and bounded reports.
use super::super::super::DomainChange;
use super::{offline, outcome, pairs, rules, texts, Call, TestResult};
use crate::settings::PasteMethod;
use crate::settings_service::{
    errors::SettingsError,
    live::LiveSettings,
    operations::SettingsBackend,
    ownership::LegacyPathProbe,
    protocol::Revision,
    store::{SettingsRead, SettingsRepository},
    transaction::{PersistObserver, PersistStage},
    validation::{validate_cli, SettingsMutation, ValidationContext},
};
use serde_json::json;

const TEAM: &str =
    "recognized;replacement\n# team rules\nget hub;GitHub!\ncloud code;Claude Code\n";

#[test]
fn import_dry_run_reports_line_statuses_and_writes_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["get hub", "GitHub"]).run(&service);
    let call = Call::new("import", &["merge"]).input(TEAM).dry_run();
    let (code, revision, data) = outcome(&call.run(&service));
    let changes = &data["changes"];
    assert_eq!(
        (code, revision, &changes["mode"]),
        (0, json!(1), &json!("merge"))
    );
    let counts = [&changes["added"], &changes["updated"], &changes["ignored"]];
    assert_eq!(counts, [&json!(1), &json!(1), &json!(2)]);
    assert_eq!(changes["lines"][0]["line"], 3);
    assert_eq!(changes["lines"][0]["status"], "update");
    assert_eq!(changes["lines"][1], json!({"line": 4, "status": "add"}));
    assert_eq!(texts(&rules(&service)), pairs(&[("get hub", "GitHub")]));
    Ok(())
}

#[test]
fn import_commits_the_previewed_set_in_one_revision() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["get hub", "GitHub"]).run(&service);
    let (code, revision, data) =
        outcome(&Call::new("import", &["merge"]).input(TEAM).run(&service));
    assert_eq!((code, revision, &data["count"]), (0, json!(2), &json!(2)));
    let expected = pairs(&[("get hub", "GitHub!"), ("cloud code", "Claude Code")]);
    assert_eq!(texts(&rules(&service)), expected);
    Ok(())
}

#[test]
fn invalid_imports_are_refused_whole_with_every_line_issue() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    Call::new("add", &["kept", "Kept"]).run(&service);
    let items = json!([{"line": 1, "field": "recognized", "kind": "duplicate_key"},
                       {"line": 2, "field": "line", "kind": "missing_separator"},
                       {"line": 3, "field": "recognized", "kind": "duplicate_key"}]);
    let call = Call::new("import", &["replace"]).input("a;1\nno separator\nA;2\n");
    for call in [call, call.dry_run()] {
        let (code, _, error) = outcome(&call.run(&service));
        assert_eq!((code, &error["details"]["items"]), (2, &items));
    }
    assert_eq!(texts(&rules(&service)), pairs(&[("kept", "Kept")]));
    assert_eq!(service.snapshot().revision, Revision(1));
    Ok(())
}

/// Backend whose commit lands after another writer committed an unrelated setting.
struct Interleaved<'a>(&'a LiveSettings);
impl SettingsBackend for Interleaved<'_> {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.0)
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::commit(self.0, mutation, expected)
    }
    fn commit_change(
        &self,
        change: &DomainChange,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let context = ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        };
        // Alternate the value so every call is a real write, never a no-op.
        let is_direct = self.0.snapshot().settings.paste_method == PasteMethod::Direct;
        let other = if is_direct { "none" } else { "direct" };
        SettingsBackend::commit(self.0, validate_cli("paste_method", other, &context)?, None)?;
        SettingsBackend::commit_change(self.0, change, expected)
    }
}

#[test]
fn an_import_is_committed_only_at_the_revision_it_previewed() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let call = Call::new("import", &["replace"]).input("cloud code;Claude Code\n");
    let (code, _, error) = outcome(&call.run(&Interleaved(&service)));
    assert_eq!((code, &error["code"]), (3, &json!("revision_conflict")));
    assert_eq!(error["details"]["current_revision"], 1);
    assert_eq!(rules(&service), json!([]));
    // A single-rule edit is decided on the locked settings instead, so it still commits.
    let added = Call::new("add", &["a", "b"]).run(&Interleaved(&service));
    assert_eq!(added.exit_code(), 0);
    assert_eq!(texts(&rules(&service)), pairs(&[("a", "b")]));
    Ok(())
}

#[test]
fn import_input_is_bounded_at_one_million_bytes_inclusive() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    // One comment line: exactly 1,000,000 bytes, then one byte more.
    let at_limit = format!("#{}\n", "x".repeat(999_998));
    let accepted = Call::new("import", &["merge"])
        .input(&at_limit)
        .run(&service);
    assert_eq!(outcome(&accepted).2["ignored"], 1);
    let over = format!("#{}\n", "x".repeat(999_999));
    let (code, _, error) = outcome(&Call::new("import", &["merge"]).input(&over).run(&service));
    assert_eq!((code, &error["code"]), (2, &json!("limit_exceeded")));
    assert_eq!(
        service.snapshot().revision,
        Revision(0),
        "a no-op import writes nothing"
    );
    Ok(())
}

#[test]
fn a_full_import_report_stays_far_below_one_reply_frame() -> TestResult {
    let directory = tempfile::tempdir()?;
    let service = offline(&directory)?;
    let input: String = (0..1000)
        .map(|index| format!("{}{index};{}\n", "字".repeat(20), "😀".repeat(200)))
        .collect();
    let reply = Call::new("import", &["replace"])
        .input(&input)
        .run(&service);
    assert_eq!(outcome(&reply).2["count"], 1000);
    assert!(reply.envelope().to_string().len() < 128 * 1024);
    Ok(())
}

/// Fails the durable write at one stage.
struct FailAt(PersistStage);
impl PersistObserver for FailAt {
    fn observe(&self, stage: PersistStage) -> std::io::Result<()> {
        if stage == self.0 {
            return Err(std::io::Error::other("injected filesystem failure"));
        }
        Ok(())
    }
}

#[test]
fn a_failed_durable_write_leaves_rules_store_and_revision_unchanged() -> TestResult {
    let directory = tempfile::tempdir()?;
    let store = directory.path().join("settings_store.json");
    let probe = LegacyPathProbe::new(directory.path().join("absent.sock"));
    let owned = LiveSettings::start_owned(SettingsRepository::new(store.clone()), &probe)?;
    Call::new("add", &["kept", "Kept"]).run(&owned);
    let (before, revision) = (std::fs::read(&store)?, owned.snapshot().revision);
    let failing = owned.with_observer(Box::new(FailAt(PersistStage::BeforeReplace)));
    let call = Call::new("import", &["replace"]).input("cloud code;Claude Code\n");
    let (code, _, error) = outcome(&call.run(&failing));
    assert_eq!((code, &error["code"]), (1, &json!("persistence_failed")));
    assert_eq!(std::fs::read(&store)?, before);
    assert_eq!(failing.snapshot().revision, revision);
    assert_eq!(texts(&rules(&failing)), pairs(&[("kept", "Kept")]));
    Ok(())
}
