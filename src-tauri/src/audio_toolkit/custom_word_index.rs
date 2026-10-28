//! Index of the custom-word keys of the approximate correction (models without a decode prompt).
//!
//! With a vocabulary of up to 5,000 terms (feature 003, decision of 2026-10-28) comparing every
//! n-gram of a dictation with every key costs hundreds of milliseconds. The index returns exactly
//! the match of the original scan (same keys, length rule, scores and tie-break), faster:
//! - keys are grouped by length, so a group outside the 25 % length rule is skipped whole;
//! - Soundex codes are computed once per key instead of once per comparison;
//! - the edit distance is bounded: a key whose distance cannot beat the best score so far (or
//!   the threshold) is abandoned after a few cells instead of a full matrix.

use super::build_match_key;

/// Phonetic matches weigh their normalized edit distance by this factor (original scoring).
const PHONETIC_WEIGHT: f64 = 0.3;
/// Candidates longer than this are never corrected (original guard against long n-grams).
const MAX_CANDIDATE_CHARS: usize = 50;

/// One normalized comparison key of a custom word (a word with `&` has a second one).
pub(super) struct CustomWordMatchKey {
    pub(super) word_index: usize,
    pub(super) key: String,
}

/// Comparison keys of `word`, the custom word at `word_index`.
pub(super) fn build_custom_word_match_keys(
    word: &str,
    word_index: usize,
) -> Vec<CustomWordMatchKey> {
    let primary_key = build_match_key(word);
    let mut keys = Vec::with_capacity(2);

    // The fallback matcher is intentionally limited to ASCII terms. Its
    // whitespace tokenization and Soundex scoring are not suitable for CJK
    // scripts. Unicode custom words remain available to models that accept
    // them as native decode prompts; they are simply skipped by this fallback.
    if is_supported_fuzzy_key(&primary_key) {
        keys.push(CustomWordMatchKey {
            word_index,
            key: primary_key.clone(),
        });
    }

    if word.contains('&') {
        let expanded_key = build_match_key(&word.replace('&', " and "));
        if is_supported_fuzzy_key(&expanded_key) && expanded_key != primary_key {
            keys.push(CustomWordMatchKey {
                word_index,
                key: expanded_key,
            });
        }
    }

    keys
}

/// Whether `key` takes part in the approximate correction (ASCII letters and digits only).
pub(super) fn is_supported_fuzzy_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Whether `key` can get a phonetic boost (ASCII letters only: Soundex is English).
pub(super) fn supports_soundex(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphabetic())
}

/// Soundex digit of a lowercase ASCII letter; `9` (h, w) and `0` (vowels) are dropped later.
fn soundex_digit(letter: u8) -> u8 {
    match letter {
        b'b' | b'f' | b'p' | b'v' => b'1',
        b'c' | b'g' | b'j' | b'k' | b'q' | b's' | b'x' | b'z' => b'2',
        b'd' | b't' => b'3',
        b'l' => b'4',
        b'm' | b'n' => b'5',
        b'r' => b'6',
        b'h' | b'w' => b'9',
        _ => b'0',
    }
}

/// Soundex code of a non-empty lowercase ASCII word, exactly as `natural::phonetics::soundex`
/// encodes it (two words sound alike when their codes are equal): the first letter as is, the
/// digits of the others without h and w, runs collapsed, vowels dropped, then cut or padded with
/// `0` to four symbols.
pub(super) fn soundex_code(word: &[u8]) -> [u8; 4] {
    let mut kept: Vec<u8> = Vec::with_capacity(word.len());
    for (position, &letter) in word.iter().enumerate() {
        let symbol = if position == 0 {
            letter
        } else {
            soundex_digit(letter)
        };
        // Order matters: h and w are removed before collapsing runs, vowels after.
        if symbol != b'9' && kept.last() != Some(&symbol) {
            kept.push(symbol);
        }
    }
    let mut code = [b'0'; 4];
    for (slot, symbol) in code.iter_mut().zip(kept.into_iter().filter(|&s| s != b'0')) {
        *slot = symbol;
    }
    code
}

/// Levenshtein distance of two byte strings when it is at most `limit`, else `None`. Only the
/// diagonal band of width `2 * limit + 1` is computed, and the scan stops as soon as a whole row
/// exceeds `limit` (no later cell can be smaller). `row` is a scratch buffer reused across calls.
pub(super) fn bounded_levenshtein(
    a: &[u8],
    b: &[u8],
    limit: usize,
    row: &mut Vec<usize>,
) -> Option<usize> {
    if a.len().abs_diff(b.len()) > limit {
        return None;
    }
    let beyond = limit + 1;
    row.clear();
    row.extend((0..=b.len()).map(|column| column.min(beyond)));
    for (index, &letter) in a.iter().enumerate() {
        let line = index + 1;
        let low = line.saturating_sub(limit).max(1);
        let high = (line + limit).min(b.len());
        let mut diagonal = row[low - 1];
        let mut left = if low == 1 { line.min(beyond) } else { beyond };
        row[low - 1] = left;
        let mut line_min = left;
        for column in low..=high {
            let up = row[column];
            let cost = usize::from(letter != b[column - 1]);
            let value = (diagonal + cost).min(up + 1).min(left + 1).min(beyond);
            diagonal = up;
            row[column] = value;
            left = value;
            line_min = line_min.min(value);
        }
        // Early exit: every path to the last cell crosses this row above the limit.
        if line_min > limit {
            return None;
        }
    }
    Some(row[b.len()]).filter(|&distance| distance <= limit)
}

/// A key ready for comparison: its rank in the original scan order breaks score ties.
struct IndexedKey {
    rank: usize,
    word_index: usize,
    key: Vec<u8>,
    sound: Option<[u8; 4]>,
}

/// Best key found so far: lowest score, then earliest rank (the original scan kept the first).
#[derive(Clone, Copy)]
struct Best {
    score: f64,
    rank: usize,
    word_index: usize,
}

/// Keys of every custom word grouped by length, for `find_best`.
pub(super) struct CustomWordIndex {
    by_length: Vec<Vec<IndexedKey>>,
}

impl CustomWordIndex {
    /// Index the keys of `custom_words` (each key keeps the rank of the original scan).
    pub(super) fn new(custom_words: &[String]) -> Self {
        let mut by_length: Vec<Vec<IndexedKey>> = Vec::new();
        let keys = custom_words
            .iter()
            .enumerate()
            .flat_map(|(index, word)| build_custom_word_match_keys(word, index));
        for (rank, match_key) in keys.enumerate() {
            let sound =
                supports_soundex(&match_key.key).then(|| soundex_code(match_key.key.as_bytes()));
            let key = match_key.key.into_bytes();
            let length = key.len();
            if by_length.len() <= length {
                by_length.resize_with(length + 1, Vec::new);
            }
            by_length[length].push(IndexedKey {
                rank,
                word_index: match_key.word_index,
                key,
                sound,
            });
        }
        Self { by_length }
    }

    /// Custom word closest to `candidate` (a match key) with a score under `threshold`, and the
    /// score: the normalized edit distance, weighted by 0.3 when both sound alike (Soundex).
    pub(super) fn find_best(&self, candidate: &str, threshold: f64) -> Option<(usize, f64)> {
        if !is_supported_fuzzy_key(candidate) || candidate.len() > MAX_CANDIDATE_CHARS {
            return None;
        }
        let sound = supports_soundex(candidate).then(|| soundex_code(candidate.as_bytes()));
        let candidate = candidate.as_bytes();
        let mut best: Option<Best> = None;
        let mut row = Vec::with_capacity(MAX_CANDIDATE_CHARS + 1);
        for (length, keys) in self.by_length.iter().enumerate() {
            if !within_length_rule(candidate.len(), length) {
                continue;
            }
            for key in keys {
                let phonetic = sound.is_some() && key.sound == sound;
                if let Some(score) = score_key(candidate, key, phonetic, threshold, best, &mut row)
                {
                    best = Some(Best {
                        score,
                        rank: key.rank,
                        word_index: key.word_index,
                    });
                }
            }
        }
        best.map(|found| (found.word_index, found.score))
    }
}

/// Original length rule: at most 25 % length difference (at least 2 characters), so an n-gram
/// never matches a much shorter custom word ("openaigpt" vs "openai").
fn within_length_rule(candidate_len: usize, key_len: usize) -> bool {
    let max_len = candidate_len.max(key_len) as f64;
    candidate_len.abs_diff(key_len) as f64 <= (max_len * 0.25).max(2.0)
}

/// Score of `key` when it beats `best` (or ties it with an earlier rank) and the threshold.
fn score_key(
    candidate: &[u8],
    key: &IndexedKey,
    phonetic: bool,
    threshold: f64,
    best: Option<Best>,
    row: &mut Vec<usize>,
) -> Option<f64> {
    let max_len = candidate.len().max(key.key.len()) as f64;
    let weight = if phonetic { PHONETIC_WEIGHT } else { 1.0 };
    let bound = best.map_or(threshold, |found| found.score.min(threshold));
    // Largest distance whose score can still reach the bound, rounded up so that float rounding
    // never drops a key; the exact comparison below decides.
    let limit = (bound * max_len / weight).ceil() as usize;
    let distance = bounded_levenshtein(candidate, &key.key, limit, row)?;
    let score = distance as f64 / max_len * weight;
    let beats_best = best
        .is_none_or(|found| score < found.score || (score == found.score && key.rank < found.rank));
    (score < threshold && beats_best).then_some(score)
}

#[cfg(test)]
#[path = "custom_word_index_tests.rs"]
mod custom_word_index_tests;
