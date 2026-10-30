//! The guard wait maps every answer of the recovery surface, and `paste-error` follows only
//! a missing consumer or a confirmed unpresentable bubble.
use super::*;

// @spec FR-002: One report per failure, bubble or paste-error — .specs/features/010-recovery-bubble/spec.md#fr-002
#[test]
fn a_failure_shows_paste_error_only_without_consumer_or_when_unpresentable() {
    assert!(toast_needed(ReasonShown::NoConsumer));
    assert!(toast_needed(ReasonShown::Unpresentable));
    assert!(!toast_needed(ReasonShown::Bubble));
    assert!(!toast_needed(ReasonShown::Pending));
}
#[test]
fn the_presentation_wait_maps_every_answer() -> std::io::Result<()> {
    use super::super::recovery::PresentationReceipt;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    assert_eq!(
        runtime.block_on(await_presentation(None)),
        ReasonShown::NoConsumer
    );
    let answers = [
        (PresentationResult::Presented, ReasonShown::Bubble),
        (PresentationResult::Superseded, ReasonShown::Bubble),
        (
            PresentationResult::Unpresentable,
            ReasonShown::Unpresentable,
        ),
    ];
    for (result, shown) in answers {
        let receipt = PresentationReceipt::resolved(result);
        assert_eq!(runtime.block_on(await_presentation(Some(receipt))), shown);
    }
    let (sender, receipt) = PresentationReceipt::pending();
    drop(sender);
    assert_eq!(
        runtime.block_on(await_presentation(Some(receipt))),
        ReasonShown::Unpresentable,
        "a dropped consumer is a confirmed failure"
    );
    Ok(())
}
