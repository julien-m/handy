//! AC-002 insert-as-usual cases of the verdict: no classification off macOS, with the setting
//! off, without a named target or for the four remote-view destinations; only "not editable"
//! recovers.
use super::*;
use crate::destination::context::ApplicationIdentity;

fn target(bundle: &str) -> ResolvedTarget {
    ResolvedTarget {
        pid: 31,
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
    }
}

#[test]
fn macos_with_the_setting_on_classifies_the_named_destination() {
    let notes = target("com.apple.Notes");
    assert_eq!(
        classification_target(RecoveryPlatform::MacOs, true, Some(&notes)),
        Some(31)
    );
}
#[test]
fn other_platforms_the_setting_off_and_no_target_classify_nothing() {
    let notes = target("com.apple.Notes");
    for platform in [RecoveryPlatform::Windows, RecoveryPlatform::Linux] {
        assert_eq!(classification_target(platform, true, Some(&notes)), None);
    }
    assert_eq!(
        classification_target(RecoveryPlatform::MacOs, false, Some(&notes)),
        None
    );
    assert_eq!(
        classification_target(RecoveryPlatform::MacOs, true, None),
        None
    );
}
#[test]
fn the_four_remote_view_destinations_are_never_classified() {
    for bundle in REMOTE_VIEW_BUNDLES {
        let remote = target(bundle);
        assert_eq!(
            classification_target(RecoveryPlatform::MacOs, true, Some(&remote)),
            None,
            "{bundle}"
        );
    }
    assert!(REMOTE_VIEW_BUNDLES.contains(&"com.p5sys.jump.mac.viewer"));
}
#[test]
fn only_not_editable_recovers() {
    use super::super::classifier::Undetermined;
    assert_eq!(
        verdict_of(FocusClassification::NotEditable),
        PreInsertionVerdict::Recover(RecoveryReason::NoTextField)
    );
    for inserted in [
        FocusClassification::Editable,
        FocusClassification::PermissionMissing,
        FocusClassification::Undetermined(Undetermined::Timeout),
        FocusClassification::Undetermined(Undetermined::ForeignProcess),
    ] {
        assert_eq!(verdict_of(inserted), PreInsertionVerdict::Insert);
    }
}
