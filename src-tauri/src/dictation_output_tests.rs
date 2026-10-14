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
    cancelled: Cell<bool>,
    ai_inputs: RefCell<Vec<String>>,
    history: RefCell<Vec<(String, Option<String>)>>,
    abandoned: Cell<bool>,
    /// Inserted text with the history receipt it was handed (the count of saved entries).
    inserted: RefCell<Vec<(String, usize)>>,
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
    fn abandon(&self) {
        self.abandoned.set(true);
    }
    async fn insert(&self, text: String, history: usize) {
        self.inserted.borrow_mut().push((text, history));
    }
}
fn processed(final_text: &str, ai: Option<String>) -> ProcessedTranscription {
    ProcessedTranscription {
        final_text: final_text.to_owned(),
        post_processed_text: ai,
        post_process_prompt: None,
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
