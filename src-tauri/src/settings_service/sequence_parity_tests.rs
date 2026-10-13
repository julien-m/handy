//! AC-003 parity of CLI writes routed to the instance for keys the settings UI prepares
//! (`vad_backend`, `selected_channel`): the same preparation, refusal and compensation as the UI
//! write, checked through the CLI `set` operation.
use super::{owned, store, stored, SpyPort, TestResult};
use crate::settings::AppSettings;
use crate::settings_service::{
    errors::SettingsError,
    live::LiveSettings,
    operations::{execute, Reply, SettingsBackend},
    protocol::{Revision, SettingsOperation, SettingsSnapshot},
    sequence::{write_prepared, write_scalar_prepared, PreparedChange},
    store::SettingsRead,
    validation::SettingsMutation,
};
use serde_json::{json, Value};

/// The instance's routed backend, with the preparation the settings UI uses for the same key.
struct Routed<'a> {
    live: &'a LiveSettings,
    port: &'a SpyPort,
    native: &'a dyn Fn() -> PreparedChange<'static>,
}
impl SettingsBackend for Routed<'_> {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        SettingsBackend::read(self.live)
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let commit =
            write_scalar_prepared(self.live, self.port, mutation, expected, (self.native)())?;
        Ok(SettingsRead {
            snapshot: SettingsSnapshot {
                settings: commit.after,
                revision: commit.revision,
            },
            warning: false,
        })
    }
    fn is_live(&self) -> bool {
        true
    }
}
/// The live state refusing a change of `key` (a recording is in progress).
fn recording_refusal(key: &'static str) -> PreparedChange<'static> {
    PreparedChange {
        prepare: Box::new(move |_, _| Err(SettingsError::live_state_refused(key))),
        compensate: Box::new(|_| {}),
    }
}
fn cli_set(backend: &dyn SettingsBackend, key: &str, value: &str, expected: Option<u64>) -> Value {
    let operation = SettingsOperation::Set {
        key: key.into(),
        value: value.into(),
        expected_revision: expected.map(Revision),
    };
    execute(&operation, backend).envelope().clone()
}
/// Routed CLI `set` of `key` to `value` with the preparation `native`.
fn routed_set(
    live: &LiveSettings,
    port: &SpyPort,
    native: &dyn Fn() -> PreparedChange<'static>,
    (key, value): (&str, &str),
    expected: Option<u64>,
) -> Value {
    let routed = Routed { live, port, native };
    cli_set(&routed, key, value, expected)
}

/// AC-003 parity: during a recording the live state refuses a VAD or channel switch; a CLI write
/// routed to the instance gets the UI's decision (`unavailable` naming the key, exit 5) and, like
/// the UI, writes nothing, changes no native state and emits no event.
#[test]
fn routed_cli_and_ui_writes_of_a_prepared_key_reach_the_same_refusal() -> TestResult {
    let cases = [
        ("vad_backend", "earshot", json!("earshot")),
        ("selected_channel", "1", json!(1)),
    ];
    for (key, cli, ui) in cases {
        let directory = tempfile::tempdir()?;
        let (live, port) = (owned(&directory)?, SpyPort::default());
        let bytes = std::fs::read(store(&directory))?;
        let ui_error = write_prepared(&live, &port, key, ui, recording_refusal(key))
            .err()
            .ok_or("UI write committed")?;
        let refusal = || recording_refusal(key);
        let cli_envelope = routed_set(&live, &port, &refusal, (key, cli), None);
        assert_eq!(cli_envelope, *Reply::failure(ui_error).envelope(), "{key}");
        let error = &cli_envelope["error"];
        assert_eq!(
            (&error["code"], &error["details"]["field"]),
            (&json!("unavailable"), &json!(key))
        );
        assert_eq!(std::fs::read(store(&directory))?, bytes);
        assert!(port.events().is_empty());
    }
    Ok(())
}

/// The requested value reaches the preparation, which a stale revision never reaches.
fn expect_earshot() -> PreparedChange<'static> {
    PreparedChange {
        prepare: Box::new(|_, requested: &AppSettings| {
            assert_eq!(requested.vad_backend, crate::settings::VadBackend::Earshot);
            Ok(())
        }),
        compensate: Box::new(|_| {}),
    }
}
#[test]
fn routed_prepared_write_checks_the_revision_before_preparing_and_skips_the_key_effect(
) -> TestResult {
    let directory = tempfile::tempdir()?;
    let (live, port) = (owned(&directory)?, SpyPort::default());
    let current = live.snapshot().revision.0;
    let refusal = || recording_refusal("vad_backend");
    let request = ("vad_backend", "earshot");
    let stale = routed_set(&live, &port, &refusal, request, Some(current + 7));
    assert_eq!(stale["error"]["code"], json!("revision_conflict"));
    let accepted = routed_set(&live, &port, &expect_earshot, request, Some(current));
    assert_eq!(accepted["ok"], json!(true));
    assert_eq!(stored(&directory, "vad_backend")?, json!("earshot"));
    // The prepared key's post-commit effect is skipped, exactly as for the UI write.
    let events = port.events();
    assert!(!events.iter().any(|event| event == "effect:vad_backend"));
    Ok(())
}
