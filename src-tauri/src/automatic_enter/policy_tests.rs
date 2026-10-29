//! Exhaustive option × source × completion matrix of the pure eligibility (SC-001), each false
//! predicate alone, and the route rule of the focus check.
use super::*;
use AutomaticEnterPolicy::{Always, Never, PinOnly};
use DestinationSource::{Badge, Captured, FrontmostAtEnd, Pinned};

const POLICIES: [AutomaticEnterPolicy; 3] = [Never, Always, PinOnly];
const SOURCES: [Option<DestinationSource>; 5] = [
    None,
    Some(Captured),
    Some(Badge),
    Some(FrontmostAtEnd),
    Some(Pinned),
];
const SMART: [MethodOutcome; 9] = [
    MethodOutcome::InsertedVerified,
    MethodOutcome::ReadObserved,
    MethodOutcome::Typed,
    MethodOutcome::NotRead,
    MethodOutcome::PermissionMissing,
    MethodOutcome::Uncertain,
    MethodOutcome::ClipboardHandedOver,
    MethodOutcome::InsertionFailed,
    MethodOutcome::FieldUnchanged,
];

fn completions() -> Vec<Completion> {
    let mut all = vec![
        Completion::Classic { succeeded: true },
        Completion::Classic { succeeded: false },
        Completion::ReliableConfirmed,
        Completion::NoInsertion,
    ];
    all.extend(SMART.map(Completion::SmartPaste));
    all
}

/// Independent statement of the expected count (0 or 1) before the claim and focus check.
fn expected(
    policy: AutomaticEnterPolicy,
    source: Option<DestinationSource>,
    done: Completion,
) -> u8 {
    let inserted = matches!(
        done,
        Completion::Classic { succeeded: true }
            | Completion::ReliableConfirmed
            | Completion::SmartPaste(
                MethodOutcome::InsertedVerified
                    | MethodOutcome::ReadObserved
                    | MethodOutcome::Typed
            )
    );
    let allowed = match policy {
        Never => false,
        Always => true,
        PinOnly => source == Some(Pinned),
    };
    u8::from(inserted && source.is_some() && allowed)
}

#[test]
fn the_full_option_source_completion_matrix_matches_the_expected_count() {
    let mut cases = 0;
    for policy in POLICIES {
        for source in SOURCES {
            for completion in completions() {
                let facts = EnterFacts {
                    policy,
                    source,
                    completion,
                };
                let got = u8::from(evaluate_enter(facts).is_ok());
                assert_eq!(got, expected(policy, source, completion), "{facts:?}");
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 3 * 5 * 13);
}
#[test]
fn each_false_predicate_alone_suppresses_with_its_reason() {
    let eligible = EnterFacts {
        policy: Always,
        source: Some(Captured),
        completion: Completion::Classic { succeeded: true },
    };
    assert_eq!(evaluate_enter(eligible), Ok(()));
    let cases = [
        (
            EnterFacts {
                completion: Completion::NoInsertion,
                ..eligible
            },
            Suppression::PasteMethodNone,
        ),
        (
            EnterFacts {
                source: None,
                ..eligible
            },
            Suppression::NotDispatched,
        ),
        (
            EnterFacts {
                completion: Completion::Classic { succeeded: false },
                ..eligible
            },
            Suppression::MethodFailed,
        ),
        (
            EnterFacts {
                policy: Never,
                ..eligible
            },
            Suppression::PolicyNever,
        ),
        (
            EnterFacts {
                policy: PinOnly,
                ..eligible
            },
            Suppression::NotPinned,
        ),
    ];
    for (facts, reason) in cases {
        assert_eq!(evaluate_enter(facts), Err(reason), "{facts:?}");
    }
}
#[test]
fn only_three_smart_paste_outcomes_are_successful_insertions() {
    for outcome in SMART {
        let facts = EnterFacts {
            policy: Always,
            source: Some(Pinned),
            completion: Completion::SmartPaste(outcome),
        };
        let eligible = matches!(
            outcome,
            MethodOutcome::InsertedVerified | MethodOutcome::ReadObserved | MethodOutcome::Typed
        );
        let result = evaluate_enter(facts);
        if eligible {
            assert_eq!(result, Ok(()), "{outcome:?}");
        } else {
            assert_eq!(result, Err(Suppression::SmartPasteOutcome(outcome)));
        }
    }
}
#[test]
fn pin_only_never_qualifies_a_captured_badge_or_frontmost_source() {
    for source in [Captured, Badge, FrontmostAtEnd] {
        let facts = EnterFacts {
            policy: PinOnly,
            source: Some(source),
            completion: Completion::Classic { succeeded: true },
        };
        assert_eq!(evaluate_enter(facts), Err(Suppression::NotPinned));
    }
}
#[test]
fn only_the_frontmost_at_end_route_skips_the_focus_check() {
    assert!(!route_has_destination(FrontmostAtEnd));
    for source in [Captured, Badge, Pinned] {
        assert!(route_has_destination(source));
    }
}
