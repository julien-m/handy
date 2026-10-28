//! Performance of the approximate custom-word correction with a full vocabulary (feature 003,
//! limit raised to 5,000 terms by Julien on 2026-10-28, native session finding N7): models that
//! take no prompt (Parakeet, Moonshine, …) correct each dictation with every stored term, so
//! one dictation of about 100 words must stay under 50 ms in an optimized build, with or without
//! 002 word replacements splitting it into protected spans. The timing tests
//! are ignored by plain `cargo test`, whose unoptimized build is about 15 times slower and whose
//! timings swing with the machine load; run it in the optimized test profile (no extra target
//! directory):
//! `cargo test --lib --config 'profile.test.package.handy.opt-level=3'
//!  --config 'profile.test.package.handy.debug-assertions=false'
//!  -- --ignored fuzzy_correction --nocapture`

use super::{apply_custom_words, apply_custom_words_outside_protected, TextSpan};
use std::time::{Duration, Instant};

/// Vocabulary size of the decision of 2026-10-28 (`recognition_vocabulary::validation::MAX_TERMS`).
const TERMS: usize = crate::recognition_vocabulary::validation::MAX_TERMS;
/// Budget of one dictation's correction in an optimized build (decision of 2026-10-28).
const BUDGET: Duration = Duration::from_millis(50);
/// Default `word_correction_threshold` of the settings.
const THRESHOLD: f64 = 0.18;
/// Timed runs; the median is compared with the budget so one scheduler hiccup cannot fail it.
const RUNS: usize = 9;

const SYLLABLES: [&str; 24] = [
    "ka", "lo", "ri", "zen", "tor", "mex", "vi", "da", "quo", "bel", "nar", "sy", "phi", "gran",
    "tel", "os", "um", "cor", "lex", "pra", "dun", "sta", "fi", "ber",
];

/// Deterministic pseudo-random sequence (64-bit LCG of Knuth's MMIX), so every run measures the
/// same vocabulary.
struct Sequence(u64);

impl Sequence {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }

    fn word(&mut self) -> String {
        let count = 2 + self.next(3);
        let word: String = (0..count)
            .map(|_| SYLLABLES[self.next(SYLLABLES.len())])
            .collect();
        let mut chars = word.chars();
        chars
            .next()
            .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
            .unwrap_or_default()
    }
}

/// `count` distinct vocabulary terms shaped like product, project and people names: mostly one
/// word of 4 to 12 letters, some two-word names, some with a version number.
pub(super) fn vocabulary(count: usize, seed: u64) -> Vec<String> {
    let mut sequence = Sequence(seed);
    let mut seen = std::collections::HashSet::new();
    let mut terms = Vec::with_capacity(count);
    while terms.len() < count {
        let term = match sequence.next(20) {
            0..=2 => format!("{} {}", sequence.word(), sequence.word()),
            3 => format!("{}{}", sequence.word(), 2 + sequence.next(30)),
            _ => sequence.word(),
        };
        if seen.insert(term.to_lowercase()) {
            terms.push(term);
        }
    }
    terms
}

/// One letter of `term` replaced, as a recognizer mishears a name (`Kalozen` → `Kalazen`).
fn misheard(term: &str) -> String {
    let mut chars: Vec<char> = term.chars().collect();
    let middle = chars.len() / 2;
    chars[middle] = if chars[middle] == 'a' { 'e' } else { 'a' };
    chars.into_iter().collect()
}

/// A dictation of about 100 words naming six vocabulary terms, four of them misheard.
pub(super) fn dictation(terms: &[String]) -> String {
    let pick = |index: usize| terms[index * terms.len() / 6].clone();
    format!(
        "Hi team, quick update before the planning meeting on Thursday. I spoke with {} about \
         the migration to {} and we agreed to move the staging cluster first. The {} dashboard \
         still shows latency spikes when the {} worker restarts, so I opened a ticket and \
         assigned it to Marco. Please review the draft proposal for {} by Friday and add your \
         comments directly in the document. Also remember that the customer demo with {} is \
         scheduled for next week, so we need the release candidate ready by Monday morning at \
         the latest. Thanks everyone.",
        misheard(&pick(0)),
        pick(1),
        misheard(&pick(2)),
        misheard(&pick(3)),
        pick(4),
        misheard(&pick(5)),
    )
}

/// Median duration of `RUNS` runs of `correct` (one warm-up run first).
fn median_time(correct: impl Fn() -> String) -> Duration {
    let _ = correct();
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            let _ = correct();
            start.elapsed()
        })
        .collect();
    times.sort();
    times[RUNS / 2]
}

/// Words of `text` as spans where every tenth word stands for the output of a 002 replacement
/// (protected), the others grouped in unprotected spans that keep their separating spaces.
pub(super) fn replaced_spans(text: &str) -> Vec<(String, bool)> {
    let mut spans: Vec<(String, bool)> = Vec::new();
    for (position, word) in text.split(' ').enumerate() {
        let separator = if position == 0 { "" } else { " " };
        if position % 10 == 5 {
            spans.push((separator.to_owned(), false));
            spans.push((word.to_owned(), true));
        } else if let Some((last, false)) = spans.last_mut() {
            last.push_str(separator);
            last.push_str(word);
        } else {
            spans.push((format!("{separator}{word}"), false));
        }
    }
    spans
}

/// Borrowed `TextSpan`s of `replaced_spans`.
pub(super) fn as_text_spans(spans: &[(String, bool)]) -> Vec<TextSpan<'_>> {
    spans
        .iter()
        .map(|(text, is_protected)| TextSpan {
            text,
            is_protected: *is_protected,
        })
        .collect()
}

fn report(path: &str, words: usize, median: Duration) {
    eprintln!(
        "fuzzy correction ({path}): {TERMS} terms, {words} words, median of {RUNS} runs {:.2} ms \
         ({} build)",
        median.as_secs_f64() * 1000.0,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "optimized"
        }
    );
    assert!(
        median < BUDGET,
        "median {median:?} over {BUDGET:?} (an unoptimized build is expected to fail)"
    );
}

#[test]
#[ignore = "timing budget of an optimized build: see the module documentation"]
fn fuzzy_correction_of_a_full_vocabulary_stays_under_50_ms() {
    let terms = vocabulary(TERMS, 20_261_028);
    let text = dictation(&terms);
    let words = text.split_whitespace().count();
    assert!((90..=110).contains(&words), "dictation has {words} words");
    let median = median_time(|| apply_custom_words(&text, &terms, THRESHOLD));
    report("plain text", words, median);
}

#[test]
#[ignore = "timing budget of an optimized build: see the module documentation"]
fn fuzzy_correction_around_word_replacements_stays_under_50_ms() {
    let terms = vocabulary(TERMS, 20_261_028);
    let text = dictation(&terms);
    let owned = replaced_spans(&text);
    let spans = as_text_spans(&owned);
    assert!(spans.iter().filter(|span| span.is_protected).count() >= 9);
    let median = median_time(|| apply_custom_words_outside_protected(&spans, &terms, THRESHOLD));
    report("002 replacements", text.split_whitespace().count(), median);
}
