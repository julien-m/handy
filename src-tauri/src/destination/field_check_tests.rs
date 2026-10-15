//! The guard table of 011 (spec AC-004, plan "Attempt after activation, decision at the
//! guard"), every row and every combination: `Verified` only for the remembered input focused
//! with its range after a restored attempt; `Unchanged` only for the noted element still
//! focused with no window raised and no write unanswered; `Diverted` otherwise.
use super::*;

const STAGES: [AttemptStage; 8] = [
    AttemptStage::Restored,
    AttemptStage::RangeMissing,
    AttemptStage::NotNoted,
    AttemptStage::FocusRefused,
    AttemptStage::WriteUnanswered,
    AttemptStage::RaiseFailed,
    AttemptStage::RangeNotRestored,
    AttemptStage::BudgetSpent,
];
const SEEN: [Option<FocusAtGuard>; 6] = [
    None,
    Some(FocusAtGuard::Remembered {
        range_verified: true,
    }),
    Some(FocusAtGuard::Remembered {
        range_verified: false,
    }),
    Some(FocusAtGuard::Noted),
    Some(FocusAtGuard::Other),
    Some(FocusAtGuard::Unreadable),
];

type Row = (AttemptStage, bool, bool, Option<FocusAtGuard>, FieldCheck);
const fn remembered(range_verified: bool) -> Option<FocusAtGuard> {
    Some(FocusAtGuard::Remembered { range_verified })
}
const NOTED: Option<FocusAtGuard> = Some(FocusAtGuard::Noted);
const fn changed(stage: AttemptStage) -> FieldCheck {
    FieldCheck::Diverted(Divergence::Changed(stage))
}
const NOT_VERIFIED: FieldCheck = FieldCheck::Diverted(Divergence::RangeNotVerified);
/// The rows of the plan's guard table, first match (stage, raised, unsettled, seen, decision).
const ROWS: [Row; 11] = {
    use AttemptStage::{FocusRefused, NotNoted, RangeMissing, Restored, WriteUnanswered};
    [
        (
            RangeMissing,
            false,
            false,
            None,
            FieldCheck::Diverted(Divergence::Attempt(RangeMissing)),
        ),
        (
            NotNoted,
            false,
            false,
            None,
            FieldCheck::Diverted(Divergence::Attempt(NotNoted)),
        ),
        (
            Restored,
            false,
            false,
            Some(FocusAtGuard::Unreadable),
            FieldCheck::Diverted(Divergence::FocusUnreadable),
        ),
        (
            Restored,
            true,
            false,
            remembered(true),
            FieldCheck::Verified,
        ),
        (Restored, false, false, remembered(false), NOT_VERIFIED),
        (FocusRefused, false, false, remembered(true), NOT_VERIFIED),
        (
            FocusRefused,
            false,
            false,
            NOTED,
            FieldCheck::Unchanged(FocusRefused),
        ),
        (
            Restored,
            false,
            false,
            NOTED,
            FieldCheck::Unchanged(Restored),
        ),
        (Restored, true, false, NOTED, changed(Restored)),
        (
            WriteUnanswered,
            false,
            true,
            NOTED,
            changed(WriteUnanswered),
        ),
        (
            Restored,
            false,
            false,
            Some(FocusAtGuard::Other),
            changed(Restored),
        ),
    ]
};

#[test]
fn each_row_of_the_guard_table() {
    for (stage, raised, unsettled, seen, expected) in ROWS {
        let decided = decide_field_check(stage, raised, unsettled, seen);
        assert_eq!(decided, expected, "{stage:?} {raised} {unsettled} {seen:?}");
    }
}

/// The table over every combination, against the rules stated in plain words.
#[test]
fn verified_and_unchanged_are_granted_only_by_their_exact_conditions() {
    for stage in STAGES {
        for raised in [false, true] {
            for unsettled in [false, true] {
                for seen in SEEN {
                    let check = decide_field_check(stage, raised, unsettled, seen);
                    let attempted =
                        !matches!(stage, AttemptStage::RangeMissing | AttemptStage::NotNoted);
                    let verified = attempted
                        && stage == AttemptStage::Restored
                        && !unsettled
                        && seen
                            == Some(FocusAtGuard::Remembered {
                                range_verified: true,
                            });
                    let unchanged =
                        attempted && !raised && !unsettled && seen == Some(FocusAtGuard::Noted);
                    let case = format!("{stage:?} {raised} {unsettled} {seen:?} -> {check:?}");
                    assert_eq!(check == FieldCheck::Verified, verified, "{case}");
                    assert_eq!(
                        matches!(check, FieldCheck::Unchanged(_)),
                        unchanged,
                        "{case}"
                    );
                    assert_ne!(check, FieldCheck::Absent, "{case}");
                }
            }
        }
    }
}

/// Spec AC-004 (b): the remembered input focused without its verified range never pastes and
/// never counts as unchanged, even when it was already focused when noted.
#[test]
fn the_remembered_input_without_its_verified_range_is_never_unchanged() {
    for stage in STAGES {
        let check = decide_field_check(
            stage,
            false,
            false,
            Some(FocusAtGuard::Remembered {
                range_verified: false,
            }),
        );
        assert!(matches!(check, FieldCheck::Diverted(_)), "{stage:?}");
    }
}

/// The context's field never reaches a serialized event (`#[serde(skip)]`).
#[test]
fn the_field_check_is_never_serialized_with_the_context() -> Result<(), serde_json::Error> {
    let context = crate::destination::context::DestinationContext {
        dictation_id: crate::destination::context::DictationId(1),
        destination_generation: 1,
        source: crate::destination::context::DestinationSource::Captured,
        target: None,
        field: FieldCheck::Diverted(Divergence::RangeNotVerified),
    };
    let json = serde_json::to_string(&context)?;
    assert!(!json.contains("field"), "{json}");
    assert!(!json.contains("range"), "{json}");
    Ok(())
}
