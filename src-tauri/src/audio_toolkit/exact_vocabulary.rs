//! Locate literal vocabulary occurrences before fuzzy matching can consume their words.
use caseless::Caseless;
use std::collections::HashMap;
use std::ops::Range;

/// Case-folded terms grouped by their first token; rebuilt with the correction index.
pub(super) struct ExactVocabulary {
    by_first_word: HashMap<String, Vec<Vec<String>>>,
}

impl ExactVocabulary {
    pub(super) fn new(terms: &[String]) -> Self {
        let mut by_first_word: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        for term in terms {
            let words: Vec<String> = term.split_whitespace().map(fold).collect();
            if let Some(first) = words.first() {
                let key = bucket_keys(first).next().unwrap_or_default().to_owned();
                by_first_word.entry(key).or_default().push(words);
            }
        }
        Self { by_first_word }
    }

    /// Union of all exact occurrences, including overlapping and arbitrarily long terms.
    // @spec FR-007: Exact occurrences precede fuzzy matching — .specs/features/003-recognition-vocabulary/spec.md#fr-007
    pub(super) fn protected_words(&self, words: &[&str]) -> Vec<bool> {
        let folded: Vec<String> = words.iter().map(|word| fold(word)).collect();
        let mut protected = vec![false; words.len()];
        for (start, word) in folded.iter().enumerate() {
            for key in bucket_keys(word) {
                let Some(terms) = self.by_first_word.get(key) else {
                    continue; // No term starts at this word boundary.
                };
                for term in terms {
                    if matches_term(&folded[start..], term) {
                        protected[start..start + term.len()].fill(true);
                    }
                }
            }
        }
        protected
    }

    /// Byte ranges let the replacement pipeline protect terms crossing its span boundaries.
    pub(super) fn protected_ranges(&self, text: &str) -> Vec<Range<usize>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let protected = self.protected_words(&words);
        let mut offset = 0;
        words
            .iter()
            .zip(protected)
            .filter_map(|(word, protected)| {
                let remaining = &text[offset..];
                let start = offset + remaining.len() - remaining.trim_start().len();
                offset = start + word.len();
                protected.then_some(start..offset)
            })
            .collect()
    }
}

fn fold(word: &str) -> String {
    word.chars().default_case_fold().collect()
}

/// Probe every word boundary (including apostrophes), plus the bucket for symbol-only terms.
fn bucket_keys(word: &str) -> impl Iterator<Item = &str> {
    word.split(is_boundary)
        .filter(|part| !part.is_empty())
        .chain(std::iter::once(""))
}

fn is_boundary(character: char) -> bool {
    !character.is_alphanumeric() && character != '_'
}

fn matches_term(words: &[String], term: &[String]) -> bool {
    if words.len() < term.len() {
        return false;
    }
    let last = term.len() - 1;
    if last == 0 {
        // Check every start: a rejected occurrence can overlap a later valid one ("xa-a-a").
        return words[0].char_indices().any(|(start, _)| {
            words[0][..start]
                .chars()
                .next_back()
                .is_none_or(is_boundary)
                && words[0][start..]
                    .strip_prefix(&term[0])
                    .is_some_and(|suffix| suffix.chars().next().is_none_or(is_boundary))
        });
    }
    // Keep literal internal spelling; contractions and possessives can surround the occurrence.
    words[0]
        .strip_suffix(&term[0])
        .is_some_and(|prefix| prefix.chars().next_back().is_none_or(is_boundary))
        && words[last]
            .strip_prefix(&term[last])
            .is_some_and(|suffix| suffix.chars().next().is_none_or(is_boundary))
        && words[1..last] == term[1..last]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protected(terms: &[&str], text: &str) -> Vec<bool> {
        let terms = terms
            .iter()
            .map(|term| (*term).to_owned())
            .collect::<Vec<_>>();
        ExactVocabulary::new(&terms).protected_words(&text.split_whitespace().collect::<Vec<_>>())
    }

    #[test]
    fn literal_symbols_unicode_and_case_folded_names_are_protected() {
        assert_eq!(
            protected(
                &["C++", "cc-hub", "Straße", "東京"],
                "avec (C++), CC-HUB STRASSE 「東京」。"
            ),
            [false, true, true, true, true]
        );
        assert_eq!(protected(&["C++"], "C# C C++."), [false, false, true]);
        assert_eq!(
            protected(&["Codex"], "d'Codex Codex’s Codexes"),
            [true, true, false]
        );
        assert_eq!(protected(&["Claude Code"], "d'Claude Code’s"), [true, true]);
        assert_eq!(protected(&["+"], "(+)."), [true]);
    }

    #[test]
    fn partial_words_and_different_internal_punctuation_are_not_exact_matches() {
        assert_eq!(protected(&["Codex"], "Codexes my_Codex Code-x"), [false; 3]);
        assert_eq!(protected(&["Claude Code"], "Claude, Code"), [false; 2]);
    }

    #[test]
    fn overlapping_and_long_terms_protect_the_union_of_their_occurrences() {
        assert_eq!(protected(&["a-a"], "xa-a-a"), [true]);
        assert_eq!(
            protected(
                &["Alpha Beta", "Beta Gamma Delta Epsilon"],
                "Alpha\tBeta Gamma Delta Epsilon zeta"
            ),
            [true, true, true, true, true, false]
        );
        assert_eq!(protected(&["", "  "], "ordinary text"), [false; 2]);
    }
}
