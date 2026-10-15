//! `handy settings destination` planning and answers (009 AC-006): usage before platform, the
//! Linux refusal, live plans only, reply shapes (`{kind, value}` identity and display name, or
//! null), the lookup refusal with no change, and no settings write.
use super::*;
use crate::destination::fake_workspace::{app, FakeClock, FakeWorkspace};
use crate::destination::workspace::Foreground;
use crate::settings::get_default_settings;
use crate::settings_service::errors::ErrorReason;
use crate::settings_service::platform_capability::PlatformCapabilities;

fn request<'a>(action: &'a str, arguments: &'a [String], supported: bool) -> DomainRequest<'a> {
    let capabilities: &'static [PlatformCapability] = if supported {
        &[PlatformCapability::Destination]
    } else {
        &[]
    };
    DomainRequest {
        action,
        arguments,
        input: None,
        platform: PlatformCapabilities::only(capabilities),
    }
}
fn plan(action: &str, arguments: &[String], supported: bool) -> Result<DomainPlan, SettingsError> {
    DestinationDomain.plan(
        &request(action, arguments, supported),
        &get_default_settings(),
    )
}
fn manager() -> DestinationManager {
    let workspace = FakeWorkspace::new(
        Foreground::Application(app(33, "com.apple.Safari", "Safari")),
        vec![app(44, "ai.perplexity.comet", "Comet")],
    );
    let platform = PlatformCapabilities::only(&[PlatformCapability::Destination]);
    DestinationManager::new(Arc::new(workspace), Arc::new(FakeClock::new()), platform)
}

#[test]
fn usage_errors_come_before_the_platform_refusal() {
    let extra = ["a".to_owned(), "b".to_owned()];
    for (action, arguments) in [
        ("pin", &[][..]),
        ("pin", &extra[..]),
        ("status", &extra[..1]),
    ] {
        for supported in [true, false] {
            let refused = plan(action, arguments, supported)
                .err()
                .map(|error| error.code());
            assert_eq!(refused, Some(SettingsErrorCode::UsageError), "{action}");
        }
    }
}
#[test]
fn linux_refuses_all_three_actions_as_unsupported_platform_before_liveness() {
    let comet = ["ai.perplexity.comet".to_owned()];
    for (action, arguments) in [("pin", &comet[..]), ("unpin", &[][..]), ("status", &[][..])] {
        let Err(error) = plan(action, arguments, false) else {
            panic!("{action} accepted on a platform without destination support");
        };
        assert_eq!(error.code(), SettingsErrorCode::ValidationFailed);
        assert_eq!(error.exit_code(), 2);
        assert_eq!(
            error.detail_reason(),
            Some(ErrorReason::UnsupportedPlatform)
        );
    }
}
#[test]
fn supported_actions_are_live_operations_only() {
    let comet = ["ai.perplexity.comet".to_owned()];
    for (action, arguments) in [("pin", &comet[..]), ("unpin", &[][..]), ("status", &[][..])] {
        assert!(
            matches!(plan(action, arguments, true), Ok(DomainPlan::Live(_))),
            "{action}"
        );
    }
}
#[test]
fn pin_reports_the_identity_and_name_and_status_reads_it_back() -> Result<(), SettingsError> {
    let manager = manager();
    assert_eq!(status_answer(&manager), json!({"pinned": null}));
    let (reply, snapshot) = pin_answer(&manager, "ai.perplexity.comet")?;
    let expected = json!({"pinned": {
        "identity": {"kind": "bundle_id", "value": "ai.perplexity.comet"},
        "display_name": "Comet"
    }});
    assert_eq!(reply, expected);
    assert_eq!(snapshot, None, "no dictation: applies from the next one");
    assert_eq!(status_answer(&manager), expected);
    let (unpinned, _) = unpin_answer(&manager);
    assert_eq!(unpinned, json!({"pinned": null}));
    assert_eq!(status_answer(&manager), json!({"pinned": null}));
    Ok(())
}
#[test]
fn an_identifier_of_no_running_application_is_not_found_and_changes_nothing() {
    let manager = manager();
    let error = pin_answer(&manager, "com.openai.chat").expect_err("refused");
    assert_eq!(
        (error.code(), error.exit_code()),
        (SettingsErrorCode::NotFound, 4)
    );
    assert_eq!(error.message(), "No running application matches.");
    assert_eq!(status_answer(&manager), json!({"pinned": null}));
}
#[test]
fn change_options_on_a_raw_live_call_are_refused_as_usage_errors() {
    let dry = LiveCall {
        dry_run: true,
        expected: None,
    };
    let expected = LiveCall {
        dry_run: false,
        expected: Some(crate::settings_service::protocol::Revision(12)),
    };
    for call in [dry, expected] {
        let refused = refuse_change_options(&call).err().map(|error| error.code());
        assert_eq!(refused, Some(SettingsErrorCode::UsageError));
    }
}
