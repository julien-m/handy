//! Orchestrator scenarios (plan Step 4 Gherkin and fault list).
use super::*;

#[test]
fn a_switch_during_recording_activates_the_captured_app_before_dispatch() {
    let fixture = default_fixture();
    let method = method();
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(
        (report.status, report.reason),
        (InsertionStatus::InsertionAttempted, None)
    );
    let contexts = inputs(&method);
    let target = contexts.first().and_then(|context| context.target.clone());
    assert_eq!(target.map(|target| target.pid), Some(textedit().pid));
    assert_eq!(
        contexts.first().map(|context| context.source),
        Some(DestinationSource::Captured)
    );
    assert_eq!(
        (activations(&fixture), disposition(&fixture)),
        (1, Some(RecoveryDisposition::Insert))
    );
}
#[test]
fn a_closed_destination_inserts_nothing_anywhere_and_is_reported_app_closed() {
    let fixture = default_fixture();
    let request = start(&fixture, true);
    fixture.workspace.inner.set_running(vec![safari()]);
    let method = method();
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, report.reason),
        (InsertionStatus::Failed, Some(DestinationFailure::AppClosed))
    );
    assert_eq!(report.target, Some(textedit().identity));
    assert!(inputs(&method).is_empty());
    assert_eq!(
        (activations(&fixture), disposition(&fixture)),
        (0, Some(RecoveryDisposition::Recover))
    );
}
#[test]
fn the_deadline_accepts_an_observation_at_999_and_1000_ms_and_refuses_1001() {
    for (delay, accepted) in [(999, true), (1000, true), (1001, false)] {
        let fixture = fixture(
            Some(Duration::from_millis(delay)),
            Arc::new(crate::destination::context::NoRecoveryConsumer),
        );
        let method = method();
        let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
        let expected = if accepted {
            (InsertionStatus::InsertionAttempted, None)
        } else {
            (
                InsertionStatus::Failed,
                Some(DestinationFailure::ActivationFailed),
            )
        };
        assert_eq!((report.status, report.reason), expected, "{delay} ms");
        assert_eq!(
            (inputs(&method).len(), activations(&fixture)),
            (usize::from(accepted), 1),
            "{delay} ms"
        );
    }
}
#[test]
fn focus_lost_during_the_insertion_delay_dispatches_nothing_and_never_reactivates() {
    let fixture = default_fixture();
    let workspace = fixture.workspace.clone();
    let mut method = method();
    method.during_delay = Box::new(move || {
        *workspace
            .front_at
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        workspace
            .inner
            .set_foreground(Foreground::Application(safari()));
    });
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(
        (report.status, report.reason),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::ActivationFailed)
        )
    );
    assert!(inputs(&method).is_empty());
    assert_eq!(
        (activations(&fixture), disposition(&fixture)),
        (1, Some(RecoveryDisposition::Recover))
    );
}
#[test]
fn an_insertion_delay_longer_than_the_deadline_is_preserved_with_a_fresh_guard() {
    let fixture = default_fixture();
    let clock = fixture.clock.clone();
    let mut method = method();
    method.during_delay = Box::new(move || clock.advance(Duration::from_millis(1500)));
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(inputs(&method).len(), 1);
}
#[test]
fn paste_method_none_activates_nothing_and_is_skipped() {
    let fixture = default_fixture();
    let started = start(&fixture, true);
    let request = request(
        started.dictation_id,
        started.destination_generation,
        true,
        Arc::new(AtomicBool::new(false)),
    );
    let mut method = method();
    method.calls_guard = false;
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, activations(&fixture)),
        (InsertionStatus::Skipped, 0)
    );
    assert_ne!(disposition(&fixture), Some(RecoveryDisposition::Insert));
}
#[test]
fn destination_return_off_inserts_into_the_app_in_front_at_the_end() {
    let fixture = default_fixture();
    let method = method();
    let report = fixture
        .orchestrator
        .deliver(start(&fixture, false), &method);
    assert_eq!(
        (report.status, activations(&fixture)),
        (InsertionStatus::InsertionAttempted, 0)
    );
    let context = inputs(&method).first().cloned();
    assert_eq!(
        context.as_ref().map(|context| context.source),
        Some(DestinationSource::FrontmostAtEnd)
    );
    let target = context
        .and_then(|context| context.target)
        .map(|target| target.pid);
    assert_eq!(target, Some(safari().pid));
}
#[test]
fn handy_in_front_at_the_end_gives_a_context_without_target() {
    let fixture = default_fixture();
    let request = start(&fixture, false);
    fixture.workspace.inner.set_foreground(Foreground::Handy);
    let method = method();
    fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        inputs(&method)
            .first()
            .map(|context| context.target.clone()),
        Some(None)
    );
}
#[test]
fn cancellation_during_activation_stops_without_input() {
    let fixture = fixture(
        None,
        Arc::new(crate::destination::context::NoRecoveryConsumer),
    );
    let started = start(&fixture, true);
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = cancelled.clone();
    let clock = fixture.clock.clone();
    let deadline = clock.now() + Duration::from_millis(300);
    let request = InsertionRequest {
        cancelled: Box::new(move || {
            flag.store(clock.now() >= deadline, Ordering::SeqCst);
            cancelled.load(Ordering::SeqCst)
        }),
        ..started
    };
    let method = method();
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, inputs(&method).len()),
        (InsertionStatus::Cancelled, 0)
    );
}
#[test]
fn a_recover_verdict_dispatches_nothing_and_no_enter_can_be_claimed() {
    let fixture = fixture(Some(Duration::from_millis(50)), Arc::new(Recover));
    let request = start(&fixture, true);
    let (id, generation) = (request.dictation_id, request.destination_generation);
    let method = method();
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, inputs(&method).len()),
        (InsertionStatus::Failed, 0)
    );
    let claim = fixture.orchestrator.manager.claim_enter(id, generation);
    assert_eq!(
        claim,
        Err(EnterRefusal::Disposition(RecoveryDisposition::Recover))
    );
}
#[test]
fn a_method_error_after_dispatch_is_an_insertion_failure_to_recover() {
    let fixture = default_fixture();
    let mut method = method();
    method.result = MethodResult::Legacy(Err("Failed to press key".to_owned()));
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(
        (report.status, report.reason),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::InsertionFailed)
        )
    );
    assert_eq!(disposition(&fixture), Some(RecoveryDisposition::Recover));
}
#[test]
fn an_already_frontmost_destination_is_not_activated_but_still_rechecked() -> TestResult {
    let fixture = default_fixture();
    let manager = &fixture.orchestrator.manager;
    let started = manager.commit_start(manager.capture(true));
    let request = request(
        started.dictation_id,
        started.destination_generation,
        false,
        Arc::new(AtomicBool::new(false)),
    );
    let method = method();
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, activations(&fixture), inputs(&method).len()),
        (InsertionStatus::InsertionAttempted, 0, 1)
    );
    Ok(())
}
#[test]
fn a_stale_or_cancelled_request_never_activates() {
    let fixture = default_fixture();
    let request = start(&fixture, true);
    fixture.orchestrator.manager.cancel();
    let method = method();
    let report = fixture.orchestrator.deliver(request, &method);
    assert_eq!(
        (report.status, activations(&fixture), inputs(&method).len()),
        (InsertionStatus::Cancelled, 0, 0)
    );
}
