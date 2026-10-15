//! Classification rules and the single 50 ms deadline with an injected clock: every query's
//! timeout is the remaining budget (never 0), a spent budget stops at once, a missing
//! permission and a foreign process make no further query.
use super::*;
use std::sync::{Arc, Mutex};

/// Clock advanced by each query of the fake element.
#[derive(Clone)]
struct StepClock {
    now: Arc<Mutex<Instant>>,
}
impl StepClock {
    fn new() -> Self {
        Self {
            now: Arc::new(Mutex::new(Instant::now())),
        }
    }
    fn advance(&self, by: Duration) {
        if let Ok(mut now) = self.now.lock() {
            *now += by;
        }
    }
}
impl Clock for StepClock {
    fn now(&self) -> Instant {
        self.now
            .lock()
            .map(|now| *now)
            .unwrap_or_else(|_| Instant::now())
    }
}

/// Scripted element: one answer per query, each query costing `cost` of the budget.
#[derive(Clone)]
struct Script {
    trusted: bool,
    focused: Result<(), AxQueryError>,
    pid: u32,
    role: Result<&'static str, AxQueryError>,
    subrole: Option<&'static str>,
    settable: [Result<bool, AxQueryError>; 3],
    cost: Duration,
}
fn script(role: &'static str) -> Script {
    Script {
        trusted: true,
        focused: Ok(()),
        pid: 42,
        role: Ok(role),
        subrole: None,
        settable: [Ok(false), Ok(false), Ok(false)],
        cost: Duration::from_millis(1),
    }
}
struct Port {
    script: Script,
    clock: StepClock,
    timeouts: Arc<Mutex<Vec<Duration>>>,
}
struct Element {
    script: Script,
    clock: StepClock,
    timeouts: Arc<Mutex<Vec<Duration>>>,
}
fn spend(clock: &StepClock, timeouts: &Mutex<Vec<Duration>>, timeout: Duration, cost: Duration) {
    if let Ok(mut seen) = timeouts.lock() {
        seen.push(timeout);
    }
    clock.advance(cost);
}
impl FocusedElementPort for Port {
    fn is_trusted(&mut self) -> bool {
        self.script.trusted
    }
    fn focused_element(
        &mut self,
        _pid: u32,
        timeout: Duration,
    ) -> Result<Box<dyn FocusedElement>, AxQueryError> {
        spend(&self.clock, &self.timeouts, timeout, self.script.cost);
        self.script.focused?;
        Ok(Box::new(Element {
            script: self.script.clone(),
            clock: self.clock.clone(),
            timeouts: Arc::clone(&self.timeouts),
        }))
    }
}
impl FocusedElement for Element {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.script.pid)
    }
    fn role(&mut self, timeout: Duration) -> Result<String, AxQueryError> {
        spend(&self.clock, &self.timeouts, timeout, self.script.cost);
        self.script.role.map(str::to_owned)
    }
    fn subrole(&mut self, timeout: Duration) -> Result<Option<String>, AxQueryError> {
        spend(&self.clock, &self.timeouts, timeout, self.script.cost);
        Ok(self.script.subrole.map(str::to_owned))
    }
    fn is_settable(
        &mut self,
        attribute: EditAttribute,
        timeout: Duration,
    ) -> Result<bool, AxQueryError> {
        spend(&self.clock, &self.timeouts, timeout, self.script.cost);
        let index = EDIT_ATTRIBUTES.iter().position(|known| *known == attribute);
        index.map_or(Err(AxQueryError::Failed), |index| {
            self.script.settable[index]
        })
    }
}
fn run(script: Script) -> (FocusClassification, Vec<Duration>) {
    let clock = StepClock::new();
    let timeouts = Arc::new(Mutex::new(Vec::new()));
    let mut port = Port {
        script,
        clock: clock.clone(),
        timeouts: Arc::clone(&timeouts),
    };
    let classification = classify(&mut port, 42, &clock);
    let seen = timeouts.lock().map(|seen| seen.clone()).unwrap_or_default();
    (classification, seen)
}

#[test]
fn a_known_non_text_focus_without_edit_affordance_is_not_editable() {
    for role in ["AXList", "AXOutline", "AXImage", "AXButton"] {
        assert_eq!(
            run(script(role)).0,
            FocusClassification::NotEditable,
            "{role}"
        );
    }
}
#[test]
fn a_text_role_with_a_settable_edit_attribute_is_editable() {
    for index in 0..3 {
        let mut field = script("AXTextArea");
        field.settable[index] = Ok(true);
        assert_eq!(run(field).0, FocusClassification::Editable);
    }
}
// @spec AC-002: Undetermined and foreign focus insert as usual — .specs/features/010-recovery-bubble/spec.md#ac-002
#[test]
fn focus_in_another_process_is_undetermined_even_with_a_non_text_role() {
    let mut foreign = script("AXList");
    foreign.pid = 7;
    let (classification, timeouts) = run(foreign);
    assert_eq!(
        classification,
        FocusClassification::Undetermined(Undetermined::ForeignProcess)
    );
    assert_eq!(timeouts.len(), 1, "no role query after the process check");
}
#[test]
fn no_focused_element_errors_secure_unknown_and_read_only_text_are_undetermined() {
    let mut none = script("AXList");
    none.focused = Err(AxQueryError::NoValue);
    let mut failed = script("AXList");
    failed.role = Err(AxQueryError::Failed);
    let mut secure = script("AXTextField");
    secure.subrole = Some("AXSecureTextField");
    let mut timed_out = script("AXList");
    timed_out.settable[1] = Err(AxQueryError::Timeout);
    let cases = [
        (none, Undetermined::NoFocusedElement),
        (failed, Undetermined::AxError),
        (secure, Undetermined::SecureField),
        (script("AXSecureTextField"), Undetermined::SecureField),
        (script("AXGroup"), Undetermined::UnknownRole),
        (script("AXTextArea"), Undetermined::TextWithoutAffordance),
        (timed_out, Undetermined::Timeout),
    ];
    for (case, cause) in cases {
        assert_eq!(run(case).0, FocusClassification::Undetermined(cause));
    }
}
// @spec AC-002: Chromium and Electron containers never show the bubble — .specs/features/010-recovery-bubble/spec.md#ac-002
#[test]
fn chromium_and_electron_containers_are_undetermined() {
    // Comet, Chrome, ChatGPT, Slack and VS Code may report the window, the web area or a group
    // while a web field has the DOM focus: never a "no text field" bubble for them.
    for role in [
        "AXWebArea",
        "AXWindow",
        "AXGroup",
        "AXApplication",
        "AXScrollArea",
    ] {
        assert_eq!(
            run(script(role)).0,
            FocusClassification::Undetermined(Undetermined::UnknownRole),
            "{role}"
        );
    }
}
#[test]
fn an_unreadable_edit_attribute_is_a_doubt_never_a_proof() {
    for error in [
        AxQueryError::Unsupported,
        AxQueryError::NoValue,
        AxQueryError::Failed,
    ] {
        let mut list = script("AXList");
        list.settable[1] = Err(error);
        assert_eq!(
            run(list).0,
            FocusClassification::Undetermined(Undetermined::AxError),
            "{error:?}"
        );
    }
    let mut slider = script("AXSlider");
    slider.settable[0] = Ok(true);
    assert_eq!(
        run(slider).0,
        FocusClassification::Undetermined(Undetermined::UnexpectedAffordance)
    );
}
#[test]
fn a_missing_permission_answers_before_any_query() {
    let mut untrusted = script("AXList");
    untrusted.trusted = false;
    let (classification, timeouts) = run(untrusted);
    assert_eq!(classification, FocusClassification::PermissionMissing);
    assert!(timeouts.is_empty());
}
#[test]
fn each_query_timeout_is_the_remaining_budget_and_never_zero() {
    let mut list = script("AXList");
    list.cost = Duration::from_millis(10);
    let (classification, timeouts) = run(list);
    assert_eq!(classification, FocusClassification::NotEditable);
    let expected: Vec<Duration> = [50, 40, 30, 20, 10]
        .into_iter()
        .map(Duration::from_millis)
        .collect();
    assert_eq!(timeouts, expected);
}
/// A deadline reached after `spent` of the 50 ms by the focused-element query.
fn after_first_query(spent: u64) -> (FocusClassification, Vec<Duration>) {
    let mut list = script("AXList");
    list.cost = Duration::from_millis(spent);
    run(list)
}
#[test]
fn the_deadline_is_one_budget_of_fifty_milliseconds() {
    let (at_49, timeouts) = after_first_query(49);
    assert_eq!(
        at_49,
        FocusClassification::Undetermined(Undetermined::BudgetSpent),
        "1 ms left after the first query, spent by the role read"
    );
    assert_eq!(timeouts[1], Duration::from_millis(1));
    for spent in [50, 51] {
        let (classification, timeouts) = after_first_query(spent);
        assert_eq!(
            classification,
            FocusClassification::Undetermined(Undetermined::BudgetSpent)
        );
        assert_eq!(timeouts.len(), 1, "no query once the budget is spent");
    }
}
