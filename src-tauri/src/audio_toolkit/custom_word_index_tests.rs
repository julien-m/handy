//! The index returns exactly what the original scan returned (every key, `strsim` Levenshtein,
//! `natural` Soundex, strict `<` so the first best key wins), on realistic vocabularies and
//! dictations, for several thresholds.

use super::super::custom_words_perf_tests::{as_text_spans, dictation, replaced_spans, vocabulary};
use super::super::{apply_custom_words, apply_custom_words_outside_protected};
use super::*;

/// The scan of every key used before the index (feature 003 limit of 500), kept as reference.
fn reference_best_match(
    candidate: &str,
    keys: &[CustomWordMatchKey],
    threshold: f64,
) -> Option<(usize, f64)> {
    if !is_supported_fuzzy_key(candidate) || candidate.chars().count() > 50 {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    for key in keys {
        let candidate_len = candidate.chars().count();
        let key_len = key.key.chars().count();
        let max_len = candidate_len.max(key_len) as f64;
        if candidate_len.abs_diff(key_len) as f64 > (max_len * 0.25).max(2.0) {
            continue;
        }
        let score = strsim::levenshtein(candidate, &key.key) as f64 / max_len;
        let phonetic = supports_soundex(candidate)
            && supports_soundex(&key.key)
            && natural::phonetics::soundex(candidate, &key.key);
        let combined = if phonetic { score * 0.3 } else { score };
        if combined < threshold && best.is_none_or(|(_, found)| combined < found) {
            best = Some((key.word_index, combined));
        }
    }
    best
}

fn match_keys(words: &[String]) -> Vec<CustomWordMatchKey> {
    words
        .iter()
        .enumerate()
        .flat_map(|(index, word)| build_custom_word_match_keys(word, index))
        .collect()
}

/// Match keys of every 1- to 3-word n-gram of `text`, as `apply_custom_words` builds them.
fn ngram_candidates(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    (0..words.len())
        .flat_map(|start| (1..=3).map(move |n| (start, n)))
        .filter(|(start, n)| start + n <= words.len())
        .map(|(start, n)| super::super::build_ngram(&words[start..start + n]))
        .collect()
}

#[test]
fn soundex_codes_group_words_as_natural_does() {
    let mut words: Vec<String> = vocabulary(400, 7)
        .iter()
        .map(|term| super::super::build_match_key(term))
        .filter(|key| supports_soundex(key))
        .collect();
    words.extend(
        [
            "h", "hw", "ashcraft", "tymczak", "pfister", "rubin", "robert", "rupert", "awhh",
        ]
        .map(String::from),
    );
    for first in &words {
        for second in words.iter().step_by(7) {
            assert_eq!(
                soundex_code(first.as_bytes()) == soundex_code(second.as_bytes()),
                natural::phonetics::soundex(first, second),
                "{first} / {second}"
            );
        }
    }
}

#[test]
fn bounded_levenshtein_is_exact_up_to_its_limit() {
    let samples = [
        "",
        "a",
        "ab",
        "kalozen",
        "kalazen",
        "torvida",
        "vidator",
        "zenzen",
        "quobelnar",
    ];
    let mut row = Vec::new();
    for first in samples {
        for second in samples {
            let exact = strsim::levenshtein(first, second);
            for limit in 0..10 {
                assert_eq!(
                    bounded_levenshtein(first.as_bytes(), second.as_bytes(), limit, &mut row),
                    (exact <= limit).then_some(exact),
                    "{first} / {second} limit {limit}"
                );
            }
        }
    }
}

#[test]
fn the_index_finds_what_the_full_scan_found() {
    let terms = vocabulary(800, 11);
    let keys = match_keys(&terms);
    let index = CustomWordIndex::new(&terms);
    let mut candidates = ngram_candidates(&dictation(&terms));
    candidates.extend(terms.iter().step_by(9).map(|term| {
        let key = super::super::build_match_key(term);
        let middle = key.len() / 2;
        format!("{}e{}", &key[..middle], &key[middle + 1..])
    }));
    for threshold in [0.1, 0.18, 0.3, 0.5] {
        for candidate in &candidates {
            assert_eq!(
                index.find_best(candidate, threshold),
                reference_best_match(candidate, &keys, threshold),
                "{candidate} at {threshold}"
            );
        }
    }
}

#[test]
fn ties_keep_the_first_custom_word_and_ampersand_keys_still_match() {
    let terms: Vec<String> = ["Kalo", "Kali", "R&D Lab"].map(String::from).to_vec();
    let index = CustomWordIndex::new(&terms);
    // "kale" is one edit from both "kalo" and "kali": the first listed word wins, as before.
    assert_eq!(index.find_best("kale", 0.5).map(|(word, _)| word), Some(0));
    assert_eq!(
        index.find_best("randdlab", 0.18).map(|(word, _)| word),
        Some(2)
    );
}

#[test]
fn a_huge_threshold_is_capped_and_matches_the_full_scan() {
    let terms = vocabulary(300, 13);
    let keys = match_keys(&terms);
    let index = CustomWordIndex::new(&terms);
    let candidates = ngram_candidates(&dictation(&terms));
    // A hand-edited store can hold any number; the scan accepted every key below it.
    for threshold in [1e20, f64::MAX, f64::INFINITY] {
        for candidate in &candidates {
            assert_eq!(
                index.find_best(candidate, threshold),
                reference_best_match(candidate, &keys, threshold),
                "{candidate} at {threshold}"
            );
        }
    }
    let mut row = Vec::new();
    assert_eq!(
        bounded_levenshtein(b"kalo", b"zen", usize::MAX, &mut row),
        Some(4)
    );
}

#[test]
fn spans_around_replacements_share_one_index_with_the_same_result() {
    let terms = vocabulary(400, 17);
    let text = dictation(&terms);
    let owned = replaced_spans(&text);
    // Former behavior: each unprotected span corrected on its own, edge whitespace kept.
    let expected: String = owned
        .iter()
        .map(|(span, is_protected)| {
            let core = span.trim();
            if *is_protected || core.is_empty() {
                return span.clone();
            }
            let start = span.len() - span.trim_start().len();
            let end = span.trim_end().len();
            format!(
                "{}{}{}",
                &span[..start],
                apply_custom_words(core, &terms, 0.18),
                &span[end..]
            )
        })
        .collect();
    assert_eq!(
        apply_custom_words_outside_protected(&as_text_spans(&owned), &terms, 0.18),
        expected
    );
    assert_ne!(expected, text, "the dictation names misheard terms");
}
