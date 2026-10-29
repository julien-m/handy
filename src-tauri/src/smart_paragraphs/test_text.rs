//! Test texts of feature 016: sentences with an exact word count, and a reader of the
//! paragraphs of an arranged text.

use unicode_segmentation::UnicodeSegmentation;

const WORDS: [&str; 12] = [
    "we",
    "reviewed",
    "the",
    "quarterly",
    "budget",
    "with",
    "finance",
    "and",
    "agreed",
    "on",
    "next",
    "steps",
];

/// A sentence of exactly `words` words, capitalized and ending with a full stop.
pub(crate) fn sentence(words: usize) -> String {
    let body: Vec<&str> = WORDS.iter().cycle().take(words).copied().collect();
    let mut text = body.join(" ");
    if let Some(first) = text.get(..1) {
        text.replace_range(..1, &first.to_uppercase());
    }
    text.push('.');
    text
}

/// Sentences of the given word counts joined by one space.
pub(crate) fn prose(word_counts: &[usize]) -> String {
    let sentences: Vec<String> = word_counts.iter().map(|&count| sentence(count)).collect();
    sentences.join(" ")
}

/// Word count of each paragraph of `text` (paragraphs separated by a blank line).
pub(crate) fn paragraph_words(text: &str) -> Vec<usize> {
    text.split("\n\n")
        .map(|paragraph| paragraph.unicode_words().count())
        .collect()
}
