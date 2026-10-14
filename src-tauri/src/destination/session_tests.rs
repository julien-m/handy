//! Pure session transitions: versioned choices and removal, the insertion freeze, every allowed
//! and refused disposition transition, the single Enter claim and the badge predicate.
use super::*;
use crate::destination::context::RecoveryDisposition::{Cancelled, Insert, Pending, Recover};
use std::time::Instant;

fn destination(bundle: &str, source: DestinationSource) -> Selection {
    Selection::Application(Destination {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: bundle.to_owned(),
        icon_ref: None,
        captured_pid: 7,
        source,
    })
}
fn session() -> DestinationSession {
    DestinationSession {
        dictation_id: DictationId(1),
        destination_generation: 4,
        selection_version: 0,
        phase: SessionPhase::Recording,
        return_enabled: true,
        support: DestinationSupport::Supported,
        selection: destination("com.openai.chat", DestinationSource::Captured),
        captured_at: Instant::now(),
        disposition: Pending,
        enter_claimed: false,
    }
}
const ID: DictationId = DictationId(1);

#[test]
fn a_choice_needs_the_exact_dictation_and_version_and_bumps_the_version() {
    let mut session = session();
    let comet = destination("ai.perplexity.comet", DestinationSource::Badge);
    assert_eq!(
        session.choose(DictationId(2), 0, comet.clone()),
        Err(SelectionRefusal::Stale)
    );
    assert_eq!(
        session.choose(ID, 1, comet.clone()),
        Err(SelectionRefusal::Stale)
    );
    assert_eq!(session.choose(ID, 0, comet.clone()), Ok(()));
    assert_eq!(
        (session.selection.clone(), session.selection_version),
        (comet, 1)
    );
}
#[test]
fn removal_is_frontmost_at_end_and_repeating_it_or_the_same_app_is_a_no_op() {
    let mut session = session();
    let same = destination("com.openai.chat", DestinationSource::Badge);
    assert_eq!(session.choose(ID, 0, same), Ok(()));
    assert_eq!(session.selection_version, 0);
    assert_eq!(session.choose(ID, 0, Selection::FrontmostAtEnd), Ok(()));
    assert_eq!(session.choose(ID, 1, Selection::FrontmostAtEnd), Ok(()));
    assert_eq!(
        (session.selection_version, session.selection.source()),
        (1, DestinationSource::FrontmostAtEnd)
    );
}
#[test]
fn choices_are_accepted_through_processing_and_refused_once_frozen() {
    let mut session = session();
    session.phase = SessionPhase::Processing;
    assert_eq!(session.choose(ID, 0, Selection::FrontmostAtEnd), Ok(()));
    assert_eq!(session.freeze(ID, 4), Ok(Selection::FrontmostAtEnd));
    let late = destination("com.apple.Notes", DestinationSource::Badge);
    assert_eq!(session.choose(ID, 1, late), Err(SelectionRefusal::Frozen));
    assert_eq!(session.freeze(ID, 4), Err(DispositionRefusal::Stale));
    assert_eq!(session.selection, Selection::FrontmostAtEnd);
}
#[test]
fn every_allowed_disposition_transition_is_accepted() {
    for (from, to) in [
        (Pending, Insert),
        (Pending, Recover),
        (Pending, Cancelled),
        (Insert, Recover),
        (Insert, Cancelled),
        (Recover, Recover),
    ] {
        let mut session = session();
        session.disposition = from;
        assert_eq!(
            session.set_disposition(ID, 4, to),
            Ok(to),
            "{from:?}->{to:?}"
        );
    }
}
#[test]
fn backward_and_stale_disposition_changes_are_refused_unchanged() {
    for (from, to) in [
        (Insert, Pending),
        (Recover, Insert),
        (Recover, Pending),
        (Cancelled, Insert),
        (Cancelled, Recover),
    ] {
        let mut session = session();
        session.disposition = from;
        assert_eq!(
            session.set_disposition(ID, 4, to),
            Err(DispositionRefusal::Final(from))
        );
        assert_eq!(session.disposition, from);
    }
    let mut session = session();
    assert_eq!(
        session.set_disposition(ID, 3, Insert),
        Err(DispositionRefusal::Stale)
    );
    assert_eq!(session.disposition, Pending);
}
#[test]
fn enter_is_claimed_once_on_insert_and_a_later_recover_does_not_revoke_it() {
    let mut session = session();
    for refused in [Pending, Recover, Cancelled] {
        session.disposition = refused;
        assert_eq!(
            session.claim_enter(ID, 4),
            Err(EnterRefusal::Disposition(refused))
        );
    }
    session.disposition = Insert;
    assert_eq!(session.claim_enter(ID, 3), Err(EnterRefusal::Stale));
    let claim = session.claim_enter(ID, 4);
    assert_eq!(
        claim,
        Ok(EnterClaim {
            dictation_id: ID,
            destination_generation: 4
        })
    );
    assert_eq!(
        session.claim_enter(ID, 4),
        Err(EnterRefusal::AlreadyClaimed)
    );
    assert_eq!(session.set_disposition(ID, 4, Recover), Ok(Recover));
    assert!(session.enter_claimed);
}
#[test]
fn badge_is_visible_only_when_on_supported_and_shown() {
    let unreadable = DestinationSupport::Unsupported(SupportReason::FrontmostUnreadable);
    assert!(badge_visible(
        true,
        DestinationSupport::Supported,
        OverlayStyle::Minimal
    ));
    assert!(badge_visible(
        true,
        DestinationSupport::Supported,
        OverlayStyle::Live
    ));
    assert!(!badge_visible(
        false,
        DestinationSupport::Supported,
        OverlayStyle::Minimal
    ));
    assert!(!badge_visible(true, unreadable, OverlayStyle::Minimal));
    assert!(!badge_visible(
        true,
        DestinationSupport::Supported,
        OverlayStyle::None
    ));
}
