//! Stage harness of one dictation (AC-007): the real native text pipeline (replacements,
//! approximate correction, fillers, normalization) feeds the real output stages, whose AI,
//! history and insertion ports are recorded. It checks which text each stage receives: the AI
//! input and the history transcription include the replacements, the AI rewrite is outside the
//! guarantee, and the history result reaches insertion.

use std::cell::{Cell, RefCell};

use super::{deliver, Delivery, OutputPorts, ProcessedTranscription};
use crate::audio_toolkit::OutputLanguageEvidence;
use crate::managers::transcription::post_process_transcription_text;
use crate::settings::AppSettings;
use crate::word_replacements::{dictation::compile_for_dictation, model::ReplacementRule};

/// Recorded output ports; `ai` stands for the configured LLM (None: post-processing disabled).
#[derive(Default)]
struct Recorded {
    ai: Option<fn(&str) -> String>,
    cancel_during_ai: bool,
    /// The dictation's `smart_paragraphs_enabled` (016).
    paragraphs: bool,
    cancelled: Cell<bool>,
    ai_inputs: RefCell<Vec<String>>,
    history: RefCell<Vec<(String, Option<String>)>>,
    abandoned: Cell<bool>,
    /// Inserted text with the history receipt it was handed (the count of saved entries).
    inserted: RefCell<Vec<(String, usize)>>,
    /// Whether each insertion was told its case was decided by the AI step (015).
    case_decided: RefCell<Vec<bool>>,
}
impl OutputPorts for Recorded {
    type History = usize;
    async fn post_process(&self, native: &str) -> Option<ProcessedTranscription> {
        let Some(ai) = self.ai else {
            return Some(processed(native, None));
        };
        self.ai_inputs.borrow_mut().push(native.to_owned());
        self.cancelled.set(self.cancel_during_ai);
        (!self.cancel_during_ai).then(|| processed(&ai(native), Some(ai(native))))
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
    fn save_history(&self, native: String, processed: &ProcessedTranscription) -> usize {
        let entry = (native, processed.post_processed_text.clone());
        self.history.borrow_mut().push(entry);
        self.history.borrow().len()
    }
    fn arrange_paragraphs(&self, text: String) -> String {
        crate::smart_paragraphs::for_dictation(self.paragraphs, text)
    }
    fn abandon(&self) {
        self.abandoned.set(true);
    }
    async fn insert(&self, text: String, case_decided: bool, history: usize) {
        self.case_decided.borrow_mut().push(case_decided);
        self.inserted.borrow_mut().push((text, history));
    }
}
fn processed(final_text: &str, ai: Option<String>) -> ProcessedTranscription {
    ProcessedTranscription {
        post_processed_text: ai,
        ..ProcessedTranscription::raw(final_text)
    }
}
fn shout(text: &str) -> String {
    text.to_uppercase()
}

/// The native pipeline of one English dictation with `rules` (recognized, replacement).
fn native(raw: &str, settings: &AppSettings, rules: &[(&str, &str)]) -> String {
    let rules: Vec<ReplacementRule> = rules
        .iter()
        .enumerate()
        .map(|(index, (recognized, replacement))| ReplacementRule {
            id: format!("rule-{index}").into(),
            recognized: (*recognized).into(),
            replacement: (*replacement).into(),
        })
        .collect();
    let english = OutputLanguageEvidence::UserSelected("en".into());
    let matcher = compile_for_dictation(&rules);
    post_process_transcription_text(raw.into(), settings, &matcher, false, &english, &[])
}
fn run(ports: &Recorded, native: String) -> Delivery {
    tauri::async_runtime::block_on(deliver(ports, native))
}
const CLAUDE: [(&str, &str); 1] = [("cloud code", "Claude Code")];

#[test]
fn without_ai_history_and_insertion_receive_the_replaced_text() {
    let text = native("open cloud code now", &AppSettings::default(), &CLAUDE);
    let ports = Recorded::default();
    assert_eq!(run(&ports, text), Delivery::Inserted);
    let expected = "open Claude Code now".to_owned();
    assert_eq!(*ports.history.borrow(), [(expected.clone(), None)]);
    assert_eq!(*ports.inserted.borrow(), [(expected, 1)]);
    assert_eq!(
        *ports.case_decided.borrow(),
        [false],
        "015: the native case is not decided"
    );
}

#[test]
fn with_ai_its_input_and_the_history_text_hold_the_replacement() {
    let text = native("open cloud code now", &AppSettings::default(), &CLAUDE);
    let ports = Recorded {
        ai: Some(shout),
        ..Default::default()
    };
    assert_eq!(run(&ports, text), Delivery::Inserted);
    let native = "open Claude Code now";
    assert_eq!(*ports.ai_inputs.borrow(), [native]);
    let rewritten = "OPEN CLAUDE CODE NOW";
    let history = [(native.to_owned(), Some(rewritten.to_owned()))];
    assert_eq!(*ports.history.borrow(), history);
    assert_eq!(*ports.inserted.borrow(), [(rewritten.to_owned(), 1)]);
    assert_eq!(
        *ports.case_decided.borrow(),
        [true],
        "015: the AI result keeps its case"
    );
}

#[test]
fn a_cancel_during_ai_writes_no_history_and_inserts_nothing() {
    let ports = Recorded {
        ai: Some(shout),
        cancel_during_ai: true,
        ..Default::default()
    };
    assert_eq!(run(&ports, "text".into()), Delivery::Cancelled);
    assert!(ports.history.borrow().is_empty() && ports.inserted.borrow().is_empty());
    assert!(!ports.abandoned.get());
}

#[test]
fn an_empty_final_text_keeps_history_and_ends_without_insertion() {
    let ports = Recorded::default();
    assert_eq!(run(&ports, String::new()), Delivery::NothingToInsert);
    assert_eq!(*ports.history.borrow(), [(String::new(), None)]);
    assert!(ports.inserted.borrow().is_empty() && ports.abandoned.get());
}

#[test]
fn replacements_never_chain_and_approximate_correction_spares_their_spans() {
    let settings = AppSettings {
        custom_words: vec!["C1".into()],
        word_correction_threshold: 0.9,
        ..Default::default()
    };
    // Control: the same approximate step turns an unprotected B1 into the vocabulary word.
    assert_eq!(native("say B1 then", &settings, &[]), "say C1 then");
    let text = native("say A1 then", &settings, &[("A1", "B1"), ("B1", "C1")]);
    assert_eq!(text, "say B1 then");
}

#[test]
fn downstream_filler_removal_may_still_drop_a_replacement() {
    let settings = AppSettings {
        filler_word_removal_enabled: true,
        custom_filler_words: Some(vec!["um".into()]),
        ..Default::default()
    };
    let text = native("say you know hello", &settings, &[("you know", "um")]);
    assert_eq!(text, "say hello");
}

#[test]
fn an_empty_rule_set_leaves_the_native_text_unchanged() {
    let settings = AppSettings::default();
    let with_none = native("open cloud code now", &settings, &[]);
    assert_eq!(with_none, "open cloud code now");
}

/// A dictation long enough for two paragraphs (016): four substantial sentences, then one.
const LONG: &str = "We reviewed the budget with finance today. Cloud spending grew faster than \
    planned this quarter. New projects are frozen until March. Marketing presents a revised plan \
    next week. Two engineering roles stay open for now.";

fn rephrase(text: &str) -> String {
    text.replace("today", "this morning")
}

// @spec AC-008: After AI and history, before insertion — .specs/features/016-smart-paragraphs/spec.md#ac-008
#[test]
fn smart_paragraphs_arrange_the_ai_text_after_history_and_before_insertion() {
    let ports = Recorded {
        ai: Some(rephrase),
        paragraphs: true,
        ..Default::default()
    };
    assert_eq!(run(&ports, LONG.to_owned()), Delivery::Inserted);
    let rewritten = rephrase(LONG);
    // History keeps the native and AI texts as they were; only insertion gets the paragraphs.
    let history = [(LONG.to_owned(), Some(rewritten.clone()))];
    assert_eq!(*ports.history.borrow(), history);
    let (inserted, receipt) = ports.inserted.borrow()[0].clone();
    assert_eq!(receipt, 1);
    assert!(inserted.contains("next week.\n\nTwo"), "{inserted}");
    assert_eq!(inserted.replace("\n\n", " "), rewritten);
}

#[test]
fn with_smart_paragraphs_off_insertion_receives_the_final_text_unchanged() {
    let ports = Recorded::default();
    assert_eq!(run(&ports, LONG.to_owned()), Delivery::Inserted);
    assert_eq!(*ports.inserted.borrow(), [(LONG.to_owned(), 1)]);
}
