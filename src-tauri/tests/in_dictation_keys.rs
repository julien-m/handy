//! The keys pressed during a dictation (014) through 006's real manager, orchestrator and guard
//! and 009's real Enter transaction over a scripted workspace. ChatGPT is pinned and Comet is in
//! front: « Send to the app in front » makes the dictation go to Comet (frontmost at the end)
//! while the next one goes to ChatGPT again; « Insert without Enter » suppresses the Enter of
//! that dictation (`UserSkipped`) whatever the policy, and Never stays Never.
#[path = "pin_enter/support.rs"]
mod support;

use handy_app_lib::automatic_enter::policy::{Completion, Suppression};
use handy_app_lib::automatic_enter::transaction::EnterResult;
use handy_app_lib::automatic_enter::AutomaticEnterPolicy as Policy;
use handy_app_lib::destination::context::DestinationSource;
use support::{chatgpt, comet, safari, Method, Rig, TestResult};

/// One pinned dictation with the keys pressed as asked, inserted, then its Enter decided.
fn dictation(
    policy: Policy,
    rerouted: bool,
    skipped: bool,
) -> Result<(Rig, EnterResult, DestinationSource), Box<dyn std::error::Error>> {
    let rig = Rig::new(chatgpt(), vec![chatgpt(), comet(), safari()]);
    rig.manager.pin_identifier("com.openai.chat")?;
    rig.workspace.bring(comet());
    let started = rig.start(true);
    let (id, generation) = (started.dictation_id, started.destination_generation);
    if rerouted {
        rig.manager.toggle_active_app(id, generation)?;
    }
    if skipped {
        rig.manager.toggle_skip_enter(id, generation)?;
    }
    let method = Method::succeeding();
    let report = rig.deliver(&started, &method, false);
    let context = report
        .context
        .map(|context| *context)
        .ok_or("not dispatched")?;
    let enter = rig.enter(&started, policy);
    let result = enter.complete(
        Completion::Classic { succeeded: true },
        Some(&context),
        rig.clock_now(),
    );
    Ok((rig, result, context.source))
}

// @spec AC-001: Re-routed dictation reaches the app in front — .specs/features/014-in-dictation-keys/spec.md#ac-001
#[test]
fn a_rerouted_dictation_reaches_the_app_in_front_and_the_next_one_the_pin() -> TestResult {
    let (rig, _, source) = dictation(Policy::Never, true, false)?;
    assert_eq!(source, DestinationSource::FrontmostAtEnd);
    let next = rig.start(true);
    let method = Method::succeeding();
    rig.deliver(&next, &method, false);
    let dispatched = method.dispatched();
    assert_eq!(dispatched[0].source, DestinationSource::Pinned);
    let target = dispatched[0].target.clone().ok_or("no target")?;
    assert_eq!(target.identity, chatgpt().identity);
    Ok(())
}

// @spec AC-002: Enter follows the final route, AC-003: skipped Enter — .specs/features/014-in-dictation-keys/spec.md#ac-002
#[test]
fn the_enter_follows_the_final_route_and_the_skip_key() -> TestResult {
    let skipped = EnterResult::Suppressed(Suppression::UserSkipped);
    let cases = [
        (Policy::PinOnly, false, false, EnterResult::Sent),
        (
            Policy::PinOnly,
            true,
            false,
            EnterResult::Suppressed(Suppression::NotPinned),
        ),
        (Policy::PinOnly, false, true, skipped.clone()),
        (Policy::Always, true, false, EnterResult::Sent),
        (Policy::Always, false, true, skipped.clone()),
        (Policy::Always, true, true, skipped),
        (
            Policy::Never,
            false,
            true,
            EnterResult::Suppressed(Suppression::PolicyNever),
        ),
    ];
    for (policy, rerouted, skip, expected) in cases {
        let (rig, result, _) = dictation(policy, rerouted, skip)?;
        let case = format!("{policy:?} rerouted={rerouted} skipped={skip}");
        assert_eq!(result, expected, "{case}");
        let pressed = usize::from(expected == EnterResult::Sent);
        assert_eq!(rig.keys.count(), pressed, "{case}");
    }
    Ok(())
}
