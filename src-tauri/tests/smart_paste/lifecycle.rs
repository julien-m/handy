//! Boundary tests of the pure smart-paste read lifecycle (AC-008, AC-009).

use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::clock::ManualClock;
use handy_app_lib::smart_paste::lifecycle::{
    DeadlineOutcome, ReadClassification, ReadLifecycle, ReadPhase,
};
use handy_app_lib::smart_paste::timing::ClipboardTimings;

// @spec FR-005: First read after key-down — .specs/features/007-smart-paste/spec.md#fr-005
// @spec FR-006: Quiet, cap and no-read timing — .specs/features/007-smart-paste/spec.md#fr-006

struct Timeline {
    clock: ManualClock,
}

impl Timeline {
    fn new() -> Self {
        Self {
            clock: ManualClock::new(),
        }
    }

    fn ms(&self, millis: u64) -> Instant {
        self.clock.at(Duration::from_millis(millis))
    }
}

fn lifecycle_with_key_down(timeline: &Timeline, timings: ClipboardTimings) -> ReadLifecycle {
    let mut lifecycle = ReadLifecycle::new(timings);
    lifecycle.record_key_down(timeline.ms(0));
    lifecycle
}

#[test]
fn first_qualifying_read_is_latched_once() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    let first = lifecycle.record_read(timeline.ms(20));
    let second = lifecycle.record_read(timeline.ms(40));
    assert_eq!(first.classification, ReadClassification::FirstQualifying);
    assert_eq!(second.classification, ReadClassification::LaterQualifying);
}

#[test]
fn read_at_key_down_instant_qualifies() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    let update = lifecycle.record_read(timeline.ms(0));
    assert_eq!(update.classification, ReadClassification::FirstQualifying);
}

#[test]
fn reads_before_key_down_never_count() {
    let timeline = Timeline::new();
    let mut lifecycle = ReadLifecycle::new(ClipboardTimings::DEFAULT);
    let published = lifecycle.record_read(timeline.ms(0));
    lifecycle.record_key_down(timeline.ms(50));
    let stale = lifecycle.record_read(timeline.ms(40));
    assert_eq!(published.classification, ReadClassification::BeforeKeyDown);
    assert_eq!(stale.classification, ReadClassification::BeforeKeyDown);
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(850)),
        DeadlineOutcome::DeclaredNotRead
    );
}

#[test]
fn restoration_waits_exactly_300_ms_after_the_last_read() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    lifecycle.record_read(timeline.ms(10));
    assert_eq!(lifecycle.next_deadline(), Some(timeline.ms(310)));
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(309)),
        DeadlineOutcome::Pending
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(310)),
        DeadlineOutcome::SettlementDue
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(311)),
        DeadlineOutcome::SettlementDue
    );
}

#[test]
fn probe_then_read_restores_after_the_second_read() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    lifecycle.record_read(timeline.ms(10));
    lifecycle.record_read(timeline.ms(160));
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(310)),
        DeadlineOutcome::Pending
    );
    assert_eq!(lifecycle.next_deadline(), Some(timeline.ms(460)));
}

#[test]
fn no_read_is_declared_exactly_at_800_ms() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(799)),
        DeadlineOutcome::Pending
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(800)),
        DeadlineOutcome::DeclaredNotRead
    );
    assert_eq!(lifecycle.phase(), ReadPhase::NotRead);
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(801)),
        DeadlineOutcome::Pending
    );
}

#[test]
fn read_stamped_at_the_no_read_deadline_is_processed_before_the_timer() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    let update = lifecycle.record_read(timeline.ms(800));
    assert!(!update.declared_not_read);
    assert_eq!(update.classification, ReadClassification::FirstQualifying);
}

#[test]
fn read_stamped_after_the_deadline_declares_not_read_first() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    let update = lifecycle.record_read(timeline.ms(801));
    assert!(update.declared_not_read);
    assert_eq!(
        update.classification,
        ReadClassification::LateAfterNotRead {
            first_late_read: true
        }
    );
}

#[test]
fn late_reads_after_not_read_are_reported_once_and_never_restore() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    lifecycle.poll_deadline(timeline.ms(800));
    let first = lifecycle.record_read(timeline.ms(900));
    let second = lifecycle.record_read(timeline.ms(950));
    assert_eq!(
        first.classification,
        ReadClassification::LateAfterNotRead {
            first_late_read: true
        }
    );
    assert_eq!(
        second.classification,
        ReadClassification::LateAfterNotRead {
            first_late_read: false
        }
    );
    assert_eq!(lifecycle.next_deadline(), None);
    assert!(!lifecycle.is_restoration_due(timeline.ms(60_000)));
}

#[test]
fn jump_desktop_quiet_boundary_is_1500_ms() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::JUMP_DESKTOP);
    lifecycle.record_read(timeline.ms(100));
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(1599)),
        DeadlineOutcome::Pending
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(1600)),
        DeadlineOutcome::SettlementDue
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(1601)),
        DeadlineOutcome::SettlementDue
    );
}

#[test]
fn jump_desktop_no_read_boundary_is_3000_ms() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::JUMP_DESKTOP);
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(2999)),
        DeadlineOutcome::Pending
    );
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(3000)),
        DeadlineOutcome::DeclaredNotRead
    );
}

#[test]
fn continuous_reads_are_capped_8_s_after_the_first_read() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    for millis in (50..=10_050).step_by(200) {
        lifecycle.record_read(timeline.ms(millis));
    }
    assert_eq!(lifecycle.next_deadline(), Some(timeline.ms(8_050)));
    assert_eq!(
        lifecycle.poll_deadline(timeline.ms(8_050)),
        DeadlineOutcome::SettlementDue
    );
}

#[test]
fn exit_restoration_requires_quiet_or_cap() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    lifecycle.record_read(timeline.ms(10));
    assert!(!lifecycle.is_restoration_due(timeline.ms(200)));
    assert!(lifecycle.is_restoration_due(timeline.ms(310)));
}

#[test]
fn reads_after_settlement_are_ignored() {
    let timeline = Timeline::new();
    let mut lifecycle = lifecycle_with_key_down(&timeline, ClipboardTimings::DEFAULT);
    lifecycle.record_read(timeline.ms(10));
    lifecycle.mark_settled();
    let update = lifecycle.record_read(timeline.ms(20));
    assert_eq!(update.classification, ReadClassification::AfterSettlement);
    assert_eq!(lifecycle.next_deadline(), None);
}
