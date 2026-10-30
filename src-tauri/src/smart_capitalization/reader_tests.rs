//! Context read rules of spec 015 AC-005 to AC-007 and 017 AC-009 and AC-010 with a scripted
//! accessibility port.
use super::*;
use crate::destination::fake_workspace::FakeClock;
use crate::recovery::classifier::EditAttribute;
use std::sync::{Arc, Mutex};

const PID: u32 = 42;

/// What the scripted element answers, and what it was asked.
#[derive(Clone)]
struct Script {
    pid: u32,
    role: &'static str,
    subrole: Option<&'static str>,
    settable: bool,
    /// The field text; the selection starts at `caret` and spans `selected` (UTF-16 units).
    text: Vec<u16>,
    caret: usize,
    selected: usize,
    /// `AXNumberOfCharacters` answers.
    counts: bool,
    /// Most units `text_in` answers (an application that answers less than asked).
    answer_cap: usize,
    /// Time spent by each query.
    query_cost: Duration,
    /// Windows asked by `text_in`.
    asked: Arc<Mutex<Vec<(usize, usize)>>>,
}

impl Script {
    fn field(text: &str, caret: usize) -> Self {
        Self {
            pid: PID,
            role: "AXTextArea",
            subrole: None,
            settable: true,
            text: text.encode_utf16().collect(),
            caret,
            selected: 0,
            counts: true,
            answer_cap: usize::MAX,
            query_cost: Duration::ZERO,
            asked: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn asked(&self) -> Vec<(usize, usize)> {
        self.asked
            .lock()
            .map(|asked| asked.clone())
            .unwrap_or_default()
    }
}

struct Element {
    script: Script,
    clock: Arc<FakeClock>,
}

impl Element {
    fn spend(&self) {
        self.clock.advance(self.script.query_cost);
    }
}

impl FocusedElement for Element {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.script.pid)
    }
    fn role(&mut self, _: Duration) -> Result<String, AxQueryError> {
        self.spend();
        Ok(self.script.role.to_owned())
    }
    fn subrole(&mut self, _: Duration) -> Result<Option<String>, AxQueryError> {
        self.spend();
        Ok(self.script.subrole.map(str::to_owned))
    }
    fn is_settable(&mut self, _: EditAttribute, _: Duration) -> Result<bool, AxQueryError> {
        self.spend();
        Ok(self.script.settable)
    }
}

impl FieldText for Element {
    fn selection(&mut self, _: Duration) -> Result<(usize, usize), AxQueryError> {
        self.spend();
        Ok((self.script.caret, self.script.selected))
    }
    fn character_count(&mut self, _: Duration) -> Result<usize, AxQueryError> {
        self.spend();
        if self.script.counts {
            Ok(self.script.text.len())
        } else {
            Err(AxQueryError::Unsupported)
        }
    }
    fn text_in(
        &mut self,
        location: usize,
        length: usize,
        _: Duration,
    ) -> Result<Vec<u16>, AxQueryError> {
        self.spend();
        if let Ok(mut asked) = self.script.asked.lock() {
            asked.push((location, length));
        }
        let answered = length.min(self.script.answer_cap);
        self.script
            .text
            .get(location..location + answered)
            .map(<[u16]>::to_vec)
            .ok_or(AxQueryError::Failed)
    }
}

impl CaretElement for Element {
    fn facts(&mut self) -> &mut dyn FocusedElement {
        self
    }
}

struct Port {
    trusted: bool,
    script: Script,
    clock: Arc<FakeClock>,
}

impl CaretPort for Port {
    fn is_trusted(&mut self) -> bool {
        self.trusted
    }
    fn focused_element(
        &mut self,
        _pid: u32,
        _: Duration,
    ) -> Result<Box<dyn CaretElement>, AxQueryError> {
        Ok(Box::new(Element {
            script: self.script.clone(),
            clock: Arc::clone(&self.clock),
        }))
    }
}

fn read_with(script: &Script, trusted: bool, with_after: bool) -> InsertionContext {
    let clock = Arc::new(FakeClock::new());
    let mut port = Port {
        trusted,
        script: script.clone(),
        clock: Arc::clone(&clock),
    };
    read_frontmost(&mut port, PID, clock.as_ref(), with_after)
}

/// The read of 015 alone (smart spacing off): the text before the caret.
fn read(script: &Script, trusted: bool) -> CursorContext {
    read_with(script, trusted, false).before().clone()
}

fn before(text: &str, reaches_start: bool) -> CursorContext {
    CursorContext::Before {
        text: text.to_owned(),
        reaches_start,
    }
}

fn after(text: &str, reaches_end: bool) -> AfterContext {
    AfterContext::After {
        text: text.to_owned(),
        reaches_end,
    }
}

/// The side after the selection of a field context; `None` for any other context.
fn after_side(context: InsertionContext) -> Option<AfterContext> {
    match context {
        InsertionContext::Field { after, .. } => Some(after),
        InsertionContext::Unavailable | InsertionContext::NotText => None,
    }
}

#[test]
fn reads_text_before_the_caret() {
    let script = Script::field("Hello, world", 7);
    assert_eq!(read(&script, true), before("Hello, ", true));
}

#[test]
fn caret_at_zero_is_empty_without_reading() {
    let script = Script::field("Hello", 0);
    assert_eq!(read(&script, true), before("", true));
    assert!(script.asked().is_empty());
}

#[test]
fn reads_at_most_window_units() {
    let long = "x".repeat(200);
    let script = Script::field(&long, 150);
    let context = read(&script, true);
    assert_eq!(
        script.asked(),
        vec![(150 - CONTEXT_WINDOW_UNITS, CONTEXT_WINDOW_UNITS)]
    );
    assert_eq!(context, before(&"x".repeat(CONTEXT_WINDOW_UNITS), false));
}

#[test]
fn selection_start_is_used() {
    // A selection "world" starting at 7: the context ends before it.
    let mut script = Script::field("Hello, world", 7);
    script.selected = 5;
    assert_eq!(read(&script, true), before("Hello, ", true));
}

#[test]
fn spacing_off_reads_nothing_after() {
    let script = Script::field("Hello world", 5);
    let context = read_with(&script, true, false);
    assert_eq!(
        context,
        InsertionContext::Field {
            before: before("Hello", true),
            after: AfterContext::Absent,
        }
    );
    assert_eq!(script.asked(), vec![(0, 5)]);
}

// @spec AC-009: Bounded read after the selection — .specs/features/017-smart-spacing/spec.md#ac-009
#[test]
fn reads_after_the_selection_end() {
    // "Bonjour[ tout le monde]demain": the selection is replaced, its two sides are read.
    let mut script = Script::field("Bonjour tout le mondedemain", 7);
    script.selected = 14;
    let context = read_with(&script, true, true);
    assert_eq!(
        context,
        InsertionContext::Field {
            before: before("Bonjour", true),
            after: after("demain", true),
        }
    );
    assert_eq!(script.asked(), vec![(0, 7), (21, 6)]);
}

#[test]
fn reads_at_most_after_window_units() {
    let script = Script::field(&"y".repeat(100), 10);
    let context = read_with(&script, true, true);
    assert_eq!(script.asked()[1], (10, AFTER_WINDOW_UNITS));
    assert_eq!(
        after_side(context),
        Some(after(&"y".repeat(AFTER_WINDOW_UNITS), false))
    );
}

#[test]
fn caret_at_the_end_is_the_field_end_without_reading_after() {
    let script = Script::field("Je souhaite", 11);
    let context = read_with(&script, true, true);
    assert_eq!(after_side(context), Some(after("", true)));
    assert_eq!(script.asked(), vec![(0, 11)]);
}

#[test]
fn unknown_length_leaves_the_side_after_absent() {
    let mut script = Script::field("Bonjour demain", 7);
    script.counts = false;
    let context = read_with(&script, true, true);
    assert_eq!(
        context,
        InsertionContext::Field {
            before: before("Bonjour", true),
            after: AfterContext::Absent,
        }
    );
}

#[test]
fn a_selection_beyond_the_field_leaves_the_side_after_absent() {
    let mut script = Script::field("Bonjour", 5);
    script.selected = 10;
    assert_eq!(
        after_side(read_with(&script, true, true)),
        Some(AfterContext::Absent)
    );
}

#[test]
fn spent_budget_after_the_before_read_is_partial() {
    // Each query costs a sixth of the budget: role, subrole, settable, selection, the read
    // before and the count fit; the read after finds the budget spent.
    let mut script = Script::field("Bonjour demain", 7);
    script.query_cost = CONTEXT_BUDGET / 6;
    let context = read_with(&script, true, true);
    assert_eq!(context.before(), &before("Bonjour", true));
    assert!(matches!(
        context,
        InsertionContext::Field {
            after: AfterContext::Absent,
            ..
        }
    ));
}

#[test]
fn untrusted_is_unavailable() {
    let script = Script::field("Hi", 2);
    assert_eq!(
        read_with(&script, false, true),
        InsertionContext::Unavailable
    );
}

#[test]
fn secure_field_is_never_read() {
    let mut role = Script::field("password", 8);
    role.role = "AXSecureTextField";
    let mut subrole = Script::field("password", 8);
    subrole.role = "AXTextField";
    subrole.subrole = Some("AXSecureTextField");
    for script in [role, subrole] {
        assert_eq!(read(&script, true), CursorContext::Absent);
        assert_eq!(read_with(&script, true, true), InsertionContext::NotText);
        assert!(script.asked().is_empty());
    }
}

// @spec AC-010: Not a text field — .specs/features/017-smart-spacing/spec.md#ac-010
#[test]
fn non_editable_is_not_text() {
    let mut read_only = Script::field("$ ls", 4);
    read_only.settable = false;
    let mut unknown = Script::field("$ ls", 4);
    unknown.role = "AXGroup";
    for script in [read_only, unknown] {
        assert_eq!(read(&script, true), CursorContext::Absent);
        assert_eq!(read_with(&script, true, true), InsertionContext::NotText);
        assert!(script.asked().is_empty());
    }
}

#[test]
fn foreign_process_is_unavailable() {
    let mut script = Script::field("Hi", 2);
    script.pid = PID + 1;
    assert_eq!(
        read_with(&script, true, true),
        InsertionContext::Unavailable
    );
}

#[test]
fn budget_spent_is_unavailable() {
    let mut script = Script::field("Hello, world", 7);
    script.query_cost = CONTEXT_BUDGET / 3;
    assert_eq!(
        read_with(&script, true, true),
        InsertionContext::Unavailable
    );
}

#[test]
fn oversized_answer_is_absent() {
    assert_eq!(
        context_from_units(&[0x61; 5], (0, 4)),
        CursorContext::Absent
    );
    assert_eq!(
        after_from_units(&[0x61; 5], (0, 4), 9),
        AfterContext::Absent
    );
}

#[test]
fn a_short_answer_before_the_caret_is_unavailable() {
    // Asked for the 30 units before the caret, the application answers 0 or 10: they start at
    // the window start, so they are not the text right before the caret (017 review W2).
    for cap in [0, 10] {
        let mut script = Script::field(&"z".repeat(40), 30);
        script.answer_cap = cap;
        assert_eq!(
            read_with(&script, true, true),
            InsertionContext::Unavailable
        );
        assert_eq!(read(&script, true), CursorContext::Absent);
    }
}

#[test]
fn a_short_answer_after_the_selection_is_kept_from_its_start() {
    let mut script = Script::field("Bonjour demain et plus encore", 7);
    script.answer_cap = 7;
    assert_eq!(
        after_side(read_with(&script, true, true)),
        Some(after(" demain", false))
    );
}

#[test]
fn windows_after_are_bounded_by_the_field() {
    assert_eq!(window_after(7, 27), Some((7, AFTER_WINDOW_UNITS)));
    assert_eq!(window_after(7, 10), Some((7, 3)));
    assert_eq!(window_after(7, 7), Some((7, 0)));
    assert_eq!(window_after(8, 7), None);
}

#[test]
fn unsupported_port_is_unavailable() {
    let clock = FakeClock::new();
    assert_eq!(
        read_frontmost(&mut UnsupportedCaretPort, PID, &clock, true),
        InsertionContext::Unavailable
    );
}
