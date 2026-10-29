//! Context read rules of spec AC-005 to AC-007 with a scripted accessibility port.
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
    /// The field text; the caret is at `caret` (UTF-16 units).
    text: Vec<u16>,
    caret: usize,
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

impl CaretElement for Element {
    fn facts(&mut self) -> &mut dyn FocusedElement {
        self
    }
    fn selection_start(&mut self, _: Duration) -> Result<usize, AxQueryError> {
        self.spend();
        Ok(self.script.caret)
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
        self.script
            .text
            .get(location..location + length)
            .map(<[u16]>::to_vec)
            .ok_or(AxQueryError::Failed)
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

fn read(script: &Script, trusted: bool) -> CursorContext {
    let clock = Arc::new(FakeClock::new());
    let mut port = Port {
        trusted,
        script: script.clone(),
        clock: Arc::clone(&clock),
    };
    read_frontmost(&mut port, PID, clock.as_ref())
}

fn before(text: &str, reaches_start: bool) -> CursorContext {
    CursorContext::Before {
        text: text.to_owned(),
        reaches_start,
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
    let script = Script::field("Hello, world", 7);
    assert_eq!(read(&script, true), before("Hello, ", true));
}

#[test]
fn untrusted_is_absent() {
    assert_eq!(read(&Script::field("Hi", 2), false), CursorContext::Absent);
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
        assert!(script.asked().is_empty());
    }
}

#[test]
fn non_editable_is_absent() {
    let mut read_only = Script::field("$ ls", 4);
    read_only.settable = false;
    let mut unknown = Script::field("$ ls", 4);
    unknown.role = "AXGroup";
    for script in [read_only, unknown] {
        assert_eq!(read(&script, true), CursorContext::Absent);
        assert!(script.asked().is_empty());
    }
}

#[test]
fn foreign_process_is_absent() {
    let mut script = Script::field("Hi", 2);
    script.pid = PID + 1;
    assert_eq!(read(&script, true), CursorContext::Absent);
}

#[test]
fn budget_spent_is_absent() {
    let mut script = Script::field("Hello, world", 7);
    script.query_cost = CONTEXT_BUDGET / 3;
    assert_eq!(read(&script, true), CursorContext::Absent);
}

#[test]
fn oversized_answer_is_absent() {
    assert_eq!(
        context_from_units(&[0x61; 5], (0, 4)),
        CursorContext::Absent
    );
}

#[test]
fn unsupported_port_is_absent() {
    let clock = FakeClock::new();
    assert_eq!(
        read_frontmost(&mut UnsupportedCaretPort, PID, &clock),
        CursorContext::Absent
    );
}
