//! Activation chain scenarios (N16): the workspace request declined or accepted without effect,
//! the accessibility fallback inside the same 1 s deadline, the guard still deciding before any
//! input, and never an insertion elsewhere.
use super::*;

/// Workspace answer `answer` and accessibility delay `accessibility` on the default fixture.
fn chain_fixture(
    delay: Option<Duration>,
    answer: ActivationRequest,
    accessibility: Option<Duration>,
) -> Fixture {
    let fixture = fixture(
        delay,
        Arc::new(crate::destination::context::NoRecoveryConsumer),
    );
    *fixture
        .workspace
        .workspace_answer
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = answer;
    *fixture
        .workspace
        .accessibility
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = accessibility;
    fixture
}
/// Times of the accessibility requests, measured from `origin`.
fn accessibility_requests(fixture: &Fixture, origin: std::time::Instant) -> Vec<Duration> {
    fixture
        .workspace
        .accessibility_at
        .lock()
        .map(|list| list.iter().map(|at| *at - origin).collect())
        .unwrap_or_default()
}
fn inserted_into(method: &Method) -> Option<u32> {
    inputs(method)
        .first()
        .and_then(|context| context.target.clone())
        .map(|target| target.pid)
}

#[test]
fn a_declined_workspace_request_falls_back_to_accessibility_at_once_and_inserts() {
    let fixture = chain_fixture(
        Some(Duration::from_millis(10)),
        ActivationRequest::Refused,
        Some(Duration::from_millis(10)),
    );
    let origin = fixture.clock.now();
    let method = method();
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(
        (report.status, report.reason),
        (InsertionStatus::InsertionAttempted, None)
    );
    assert_eq!(inserted_into(&method), Some(textedit().pid));
    assert_eq!(activations(&fixture), 1);
    assert_eq!(
        accessibility_requests(&fixture, origin),
        vec![Duration::ZERO]
    );
}
#[test]
fn an_accepted_request_that_brings_nothing_forward_escalates_after_300_ms() {
    let fixture = chain_fixture(
        None,
        ActivationRequest::Sent,
        Some(Duration::from_millis(20)),
    );
    let origin = fixture.clock.now();
    let method = method();
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(inserted_into(&method), Some(textedit().pid));
    assert_eq!(
        accessibility_requests(&fixture, origin),
        vec![crate::destination::activation::ESCALATION_DELAY]
    );
}
#[test]
fn a_workspace_request_that_works_sends_no_accessibility_request() {
    let fixture = chain_fixture(
        Some(Duration::from_millis(100)),
        ActivationRequest::Sent,
        Some(Duration::from_millis(10)),
    );
    let origin = fixture.clock.now();
    let report = fixture
        .orchestrator
        .deliver(start(&fixture, true), &method());
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert!(accessibility_requests(&fixture, origin).is_empty());
}
#[test]
fn every_strategy_declined_fails_at_once_and_inserts_nothing() {
    let fixture = chain_fixture(
        Some(Duration::from_millis(10)),
        ActivationRequest::Refused,
        None,
    );
    let origin = fixture.clock.now();
    let method = method();
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
        fixture.clock.now(),
        origin,
        "no wait once all were declined"
    );
    assert_eq!(disposition(&fixture), Some(RecoveryDisposition::Recover));
}
#[test]
fn the_fallback_shares_the_1_s_deadline() {
    for (accessibility_delay, accepted) in [(700, true), (701, false)] {
        let fixture = chain_fixture(
            None,
            ActivationRequest::Sent,
            Some(Duration::from_millis(accessibility_delay)),
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
        let label = format!("in front {accessibility_delay} ms after the 300 ms fallback");
        assert_eq!((report.status, report.reason), expected, "{label}");
        assert_eq!(inputs(&method).len(), usize::from(accepted), "{label}");
    }
}
#[test]
fn the_guard_still_refuses_a_switch_after_the_fallback_activation() {
    let fixture = chain_fixture(
        Some(Duration::from_millis(10)),
        ActivationRequest::Refused,
        Some(Duration::from_millis(10)),
    );
    let workspace = fixture.workspace.clone();
    let mut method = method();
    method.during_delay = Box::new(move || {
        *workspace
            .live
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(Foreground::Application(safari()));
    });
    let report = fixture.orchestrator.deliver(start(&fixture, true), &method);
    assert_eq!(
        (report.status, report.reason),
        (
            InsertionStatus::Failed,
            Some(DestinationFailure::ActivationFailed)
        )
    );
    assert!(inputs(&method).is_empty(), "nothing typed into Safari");
}
#[test]
fn an_already_frontmost_destination_gets_no_request_of_any_strategy() -> TestResult {
    let fixture = chain_fixture(
        Some(Duration::from_millis(10)),
        ActivationRequest::Refused,
        Some(Duration::from_millis(10)),
    );
    let origin = fixture.clock.now();
    let manager = &fixture.orchestrator.manager;
    let started = manager.commit_start(manager.capture(true));
    let request = request(
        started.dictation_id,
        started.destination_generation,
        false,
        Arc::new(AtomicBool::new(false)),
    );
    let report = fixture.orchestrator.deliver(request, &method());
    assert_eq!(report.status, InsertionStatus::InsertionAttempted);
    assert_eq!(activations(&fixture), 0);
    assert!(accessibility_requests(&fixture, origin).is_empty());
    Ok(())
}
