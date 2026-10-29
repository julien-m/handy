//! 010 AC-003 payload through the production `RecoveryService` with native boundaries faked:
//! Copy and drag carry exactly the text the paste would have inserted, kept by 006's real slot
//! (with the trailing space when `append_trailing_space` was on), while the preview stays the
//! transcript. The settings snapshot reaching the slot is tested in `destination/delivery_tests.rs`.
#[path = "recovery/support.rs"]
mod support;

use handy_app_lib::destination::recovery::OutcomeConsumer;
use handy_app_lib::recovery::types::DragResult::Released;
use handy_app_lib::recovery::types::RecoveryPlatform;
use support::{lock, Bench, SurfaceCall, TestResult};

// @spec AC-003: Copy and drag carry exactly the inserted text — .specs/features/010-recovery-bubble/spec.md#ac-003
#[test]
fn copy_and_drag_carry_the_paste_trailing_space_and_the_preview_never_shows_it() -> TestResult {
    const SHORT: &str = "short safe phrase";
    for append_trailing_space in [true, false] {
        let inserted = if append_trailing_space {
            format!("{SHORT} ")
        } else {
            SHORT.to_owned()
        };
        let bench = Bench::new(RecoveryPlatform::MacOs);
        let outcome = bench.failed_final(1, SHORT, append_trailing_space);
        bench.service.consume(&outcome);
        let view = bench.surface.presented().pop().ok_or("not presented")?;
        assert_eq!(
            view.preview, SHORT,
            "the preview never shows the paste space"
        );
        bench.service.rendered(view.bubble_id);
        bench.service.begin_drag(view.bubble_id);
        let begun = SurfaceCall::BeginDrag(view.bubble_id, inserted.clone());
        assert!(bench.surface.calls().contains(&begun), "drag payload");
        bench.service.drag_ended(view.bubble_id, Released);
        let copied = bench
            .service
            .copy(view.bubble_id, view.destination_generation);
        assert!(copied.is_ok());
        assert_eq!(lock(&bench.copier.written).clone(), vec![inserted]);
    }
    Ok(())
}
