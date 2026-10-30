//! Feature 001, Step 3: the ordered resolver against fake ports and a fake monotonic clock (no
//! sleeps). Each scenario checks the opened choice and reason, every failed attempt in order,
//! which native calls were made and when, and the budget rule: full 2 s per started attempt, no
//! configured attempt with less than 4 s left, the default started by t = 4 s.
use handy_app_lib::microphones::catalog::{Catalog, CatalogEntry};
use handy_app_lib::microphones::policy::{
    resolve, AttemptResult, OpenFailure, OpenTarget, Preferences, Resolution, ResolutionPorts,
};
use handy_app_lib::microphones::types::EffectiveReason;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}
const DEFAULT: &str = "<default>";

/// Scripted native behavior; every call advances the clock by its cost (capped by its budget).
#[derive(Default)]
struct Fake {
    clock: Duration,
    lid: Option<(Option<bool>, Duration)>,
    enumerations: VecDeque<(Option<Vec<&'static str>>, Duration)>,
    /// name (or `DEFAULT`) -> (cost, opens); unscripted targets open at once.
    opens: HashMap<&'static str, (Duration, bool)>,
    calls: Vec<String>,
}
impl Fake {
    fn present(names: &[&'static str]) -> Self {
        let mut fake = Self::default();
        fake.enumerations.push_back((Some(names.to_vec()), ms(50)));
        fake
    }
    fn open_as(mut self, name: &'static str, cost: Duration, opens: bool) -> Self {
        self.opens.insert(name, (cost, opens));
        self
    }
    /// Spend `cost`, or the whole `budget` when the call would outlast it.
    fn spend(&mut self, cost: Duration, budget: Duration) -> bool {
        self.clock += cost.min(budget);
        cost <= budget
    }
}
impl ResolutionPorts for Fake {
    type Device = String;
    type Stream = String;
    fn elapsed(&self) -> Duration {
        self.clock
    }
    fn lid_closed(&mut self, budget: Duration) -> Option<bool> {
        self.calls.push("lid".into());
        let (answer, cost) = self.lid?;
        self.spend(cost, budget).then_some(answer).flatten()
    }
    fn enumerate(&mut self, budget: Duration) -> Option<Catalog<String>> {
        self.calls
            .push(format!("enumerate@{}", self.clock.as_millis()));
        let (names, cost) = self.enumerations.pop_front()?;
        let entries = names?
            .into_iter()
            .map(|name| CatalogEntry {
                name: name.into(),
                device: name.into(),
            })
            .collect();
        self.spend(cost, budget).then(|| Catalog::new(entries))
    }
    fn open(
        &mut self,
        target: OpenTarget<'_, String>,
        budget: Duration,
    ) -> Result<String, OpenFailure> {
        let name = match target {
            OpenTarget::Device(device) => device.clone(),
            OpenTarget::SystemDefault => DEFAULT.to_owned(),
        };
        self.calls
            .push(format!("open {name}@{}", self.clock.as_millis()));
        let (cost, opens) = self
            .opens
            .get(name.as_str())
            .copied()
            .unwrap_or((ms(100), true));
        match (self.spend(cost, budget), opens) {
            (false, _) => Err(OpenFailure::TimedOut),
            (true, true) => Ok(name),
            (true, false) => Err(OpenFailure::Failed("refused".into())),
        }
    }
}

fn order(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}
fn run(fake: &mut Fake, priority: &[&str], clamshell: Option<&str>) -> Resolution<String> {
    let priority = order(priority);
    let preferences = Preferences {
        priority: &priority,
        clamshell,
    };
    resolve(&preferences, fake)
}
fn opened(resolution: &Resolution<String>) -> Option<(String, EffectiveReason)> {
    resolution
        .opened
        .as_ref()
        .map(|opened| (opened.stream.clone(), opened.reason))
}
fn failures(resolution: &Resolution<String>) -> Vec<(Option<&str>, AttemptResult)> {
    let attempts = resolution.attempts.iter();
    attempts
        .map(|attempt| (attempt.choice.as_deref(), attempt.result.clone()))
        .collect()
}

// @spec AC-003: Saved order, first present unambiguous device opens — .specs/features/001-microphone-priority/spec.md#ac-003
#[test]
fn the_first_present_choice_opens_without_trying_the_others() {
    let mut fake = Fake::present(&["USB headset", "MacBook microphone"]);
    let resolution = run(&mut fake, &["USB headset", "MacBook microphone"], None);
    assert_eq!(
        opened(&resolution),
        Some(("USB headset".into(), EffectiveReason::FirstChoice))
    );
    assert!(resolution.attempts.is_empty());
    assert_eq!(fake.calls, ["enumerate@0", "open USB headset@50"]);
}
#[test]
fn an_unplugged_first_choice_falls_back_to_the_next_without_an_open_call() {
    let mut fake = Fake::present(&["MacBook microphone"]);
    let resolution = run(&mut fake, &["USB headset", "MacBook microphone"], None);
    assert_eq!(
        opened(&resolution),
        Some(("MacBook microphone".into(), EffectiveReason::Fallback))
    );
    assert_eq!(
        failures(&resolution),
        [(Some("USB headset"), AttemptResult::Missing)]
    );
    assert_eq!(fake.calls, ["enumerate@0", "open MacBook microphone@50"]);
}
#[test]
fn a_name_shared_by_two_devices_is_a_failed_attempt_never_an_arbitrary_match() {
    let mut fake = Fake::present(&["USB headset", "USB headset", "MacBook microphone"]);
    let resolution = run(&mut fake, &["USB headset", "MacBook microphone"], None);
    assert_eq!(
        failures(&resolution),
        [(Some("USB headset"), AttemptResult::Ambiguous)]
    );
    assert_eq!(
        opened(&resolution).map(|(name, _)| name),
        Some("MacBook microphone".into())
    );
    assert!(!fake.calls.iter().any(|call| call.starts_with("open USB")));
}
#[test]
fn the_default_is_attempted_only_after_every_configured_choice_failed() {
    let mut fake = Fake::present(&["A"]).open_as("A", ms(300), false);
    let resolution = run(&mut fake, &["A", "Gone"], None);
    assert_eq!(
        opened(&resolution),
        Some((DEFAULT.into(), EffectiveReason::SystemDefault))
    );
    let expected = [
        (
            Some("A"),
            AttemptResult::Open(OpenFailure::Failed("refused".into())),
        ),
        (Some("Gone"), AttemptResult::Missing),
    ];
    assert_eq!(failures(&resolution), expected);
    assert_eq!(
        fake.calls,
        ["enumerate@0", "open A@50", "open <default>@350"]
    );
}
#[test]
fn a_failed_default_reports_the_error_after_the_configured_attempts() {
    let mut fake = Fake::present(&[]).open_as(DEFAULT, ms(10), false);
    let resolution = run(&mut fake, &["USB headset"], None);
    assert!(resolution.opened.is_none());
    let last = resolution
        .attempts
        .last()
        .map(|attempt| attempt.choice.clone());
    assert_eq!(last, Some(None));
    assert_eq!(resolution.attempts.len(), 2);
}
#[test]
fn an_empty_order_opens_the_default_without_enumerating() {
    let mut fake = Fake::present(&["USB headset"]);
    let resolution = run(&mut fake, &[], None);
    assert_eq!(
        opened(&resolution),
        Some((DEFAULT.into(), EffectiveReason::SystemDefault))
    );
    assert_eq!(fake.calls, ["open <default>@0"]);
}
#[test]
fn a_failed_enumeration_is_retried_once_then_choices_count_as_unresolved() {
    let mut fake = Fake::default();
    fake.enumerations.extend([(None, ms(20)), (None, ms(20))]);
    let resolution = run(&mut fake, &["USB headset"], None);
    assert!(resolution.enumeration_failed);
    assert_eq!(
        failures(&resolution),
        [(Some("USB headset"), AttemptResult::Unresolved)]
    );
    assert_eq!(
        fake.calls,
        ["enumerate@0", "enumerate@0", "open <default>@0"]
    );
    let mut recovered = Fake::default();
    recovered
        .enumerations
        .extend([(None, ms(20)), (Some(vec!["USB headset"]), ms(20))]);
    let resolution = run(&mut recovered, &["USB headset"], None);
    assert_eq!(
        opened(&resolution).map(|(_, reason)| reason),
        Some(EffectiveReason::FirstChoice)
    );
}

// @spec AC-003: Full 2 s per attempt, default by 4 s, 6 s total — .specs/features/001-microphone-priority/spec.md#ac-003
#[test]
fn slow_devices_get_full_attempts_and_the_default_starts_by_four_seconds() {
    let mut fake = Fake::present(&["A", "B", "C", "D"]);
    fake.enumerations[0].1 = Duration::ZERO;
    for name in ["A", "B", "C", "D"] {
        fake.opens.insert(name, (ms(9_000), true));
    }
    let resolution = run(&mut fake, &["A", "B", "C", "D"], None);
    assert_eq!(
        fake.calls,
        [
            "enumerate@0",
            "open A@0",
            "open B@2000",
            "open <default>@4000"
        ]
    );
    let results = failures(&resolution).into_iter().map(|(_, result)| result);
    let timed_out = AttemptResult::Open(OpenFailure::TimedOut);
    let deadline = AttemptResult::Deadline;
    let expected = [timed_out.clone(), timed_out, deadline.clone(), deadline];
    assert!(results.eq(expected));
    assert_eq!((fake.clock, resolution.opened.is_some()), (ms(4_100), true));
}
#[test]
fn fast_failures_leave_room_for_every_choice() {
    let names = ["A", "B", "C", "D", "E"];
    let mut fake = Fake::present(&names);
    for name in names {
        fake.opens.insert(name, (ms(150), false));
    }
    let resolution = run(&mut fake, &names, None);
    assert_eq!(resolution.attempts.len(), 5);
    assert_eq!(
        opened(&resolution).map(|(name, _)| name),
        Some(DEFAULT.into())
    );
    assert!(fake.calls.contains(&"open <default>@800".to_owned()));
}
#[test]
fn a_slow_enumeration_ends_by_four_seconds_and_keeps_the_default_slot() {
    let mut fake = Fake::default();
    fake.enumerations
        .push_back((Some(vec!["USB headset"]), ms(30_000)));
    let resolution = run(&mut fake, &["USB headset"], None);
    assert!(resolution.enumeration_failed);
    assert_eq!(fake.calls, ["enumerate@0", "open <default>@4000"]);
    assert_eq!(
        failures(&resolution)[0],
        (Some("USB headset"), AttemptResult::Deadline)
    );
}

// @spec AC-005: Closed lid uses the override, then the default — .specs/features/001-microphone-priority/spec.md#ac-005
#[test]
fn a_closed_lid_tries_only_the_override_then_the_default() {
    let mut fake = Fake::present(&["USB headset"]);
    fake.lid = Some((Some(true), ms(15)));
    let resolution = run(&mut fake, &["USB headset"], Some("Dock microphone"));
    assert_eq!(
        failures(&resolution),
        [(Some("Dock microphone"), AttemptResult::Missing)]
    );
    assert_eq!(
        opened(&resolution).map(|(_, reason)| reason),
        Some(EffectiveReason::SystemDefault)
    );
    assert!(!fake.calls.iter().any(|call| call.contains("USB headset")));
}
#[test]
fn a_closed_lid_with_an_empty_order_still_tries_the_override_first() {
    let mut fake = Fake::present(&["Dock microphone"]);
    fake.lid = Some((Some(true), ms(15)));
    let resolution = run(&mut fake, &[], Some("Dock microphone"));
    let opened = opened(&resolution);
    assert_eq!(
        opened,
        Some(("Dock microphone".into(), EffectiveReason::Clamshell))
    );
}
#[test]
fn an_open_unknown_or_slow_lid_uses_the_saved_order() {
    for lid in [
        Some((Some(false), ms(15))),
        Some((None, ms(15))),
        Some((Some(true), ms(9_000))),
    ] {
        let mut fake = Fake::present(&["USB headset", "Dock microphone"]);
        fake.lid = lid;
        let resolution = run(&mut fake, &["USB headset"], Some("Dock microphone"));
        let reason = opened(&resolution).map(|(_, reason)| reason);
        assert_eq!(reason, Some(EffectiveReason::FirstChoice), "{lid:?}");
    }
}
#[test]
fn without_an_override_the_lid_is_never_probed() {
    let mut fake = Fake::present(&["USB headset"]);
    fake.lid = Some((Some(true), ms(15)));
    run(&mut fake, &["USB headset"], None);
    assert!(!fake.calls.contains(&"lid".to_owned()));
}
#[test]
fn a_reconnected_first_choice_is_preferred_at_the_next_opening() {
    let mut unplugged = Fake::present(&["MacBook microphone"]);
    let first = run(&mut unplugged, &["USB headset", "MacBook microphone"], None);
    assert_eq!(
        opened(&first).map(|(_, reason)| reason),
        Some(EffectiveReason::Fallback)
    );
    let mut reconnected = Fake::present(&["MacBook microphone", "USB headset"]);
    let next = run(
        &mut reconnected,
        &["USB headset", "MacBook microphone"],
        None,
    );
    let opened = opened(&next);
    assert_eq!(
        opened,
        Some(("USB headset".into(), EffectiveReason::FirstChoice))
    );
}
#[test]
fn the_preference_check_names_what_would_open_without_opening_it() {
    use handy_app_lib::microphones::policy::preferred;
    let priority = order(&["USB headset", "MacBook microphone"]);
    let preferences = Preferences {
        priority: &priority,
        clamshell: None,
    };
    let mut fake = Fake::present(&["MacBook microphone"]);
    assert_eq!(
        preferred(&preferences, &mut fake),
        Some(Some("MacBook microphone".into()))
    );
    assert!(!fake.calls.iter().any(|call| call.starts_with("open")));
    let mut absent = Fake::present(&[]);
    assert_eq!(preferred(&preferences, &mut absent), Some(None));
    let mut failing = Fake::default();
    assert_eq!(preferred(&preferences, &mut failing), None);
    let empty = Preferences {
        priority: &[],
        clamshell: None,
    };
    assert_eq!(preferred(&empty, &mut Fake::default()), Some(None));
}
