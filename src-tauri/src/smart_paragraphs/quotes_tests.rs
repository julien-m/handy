//! Quotation balance (016 AC-006).

use super::QuoteBalance;

fn closed_after(text: &str) -> bool {
    let mut balance = QuoteBalance::default();
    balance.feed(text);
    balance.is_closed()
}

#[test]
fn paired_marks_open_and_close() {
    for text in [
        "He said \"yes\" twice.",
        "He said “yes” twice.",
        "Er sagte „ja“ zweimal.",
        "Il a dit « oui » deux fois.",
        "彼は「はい」と言った。",
        "『本』を読んだ。",
    ] {
        assert!(closed_after(text), "{text}");
    }
    for text in [
        "He said \"yes",
        "He said “yes",
        "Il a dit « oui",
        "彼は「はい",
    ] {
        assert!(!closed_after(text), "{text}");
    }
}

#[test]
fn apostrophes_and_stray_closers_are_ignored() {
    assert!(closed_after("L'ami d'Anna don't care."));
    assert!(closed_after("A stray » mark."));
}

#[test]
fn nested_quotations_close_innermost_first() {
    assert!(!closed_after("« Il a dit “oui” et"));
    assert!(closed_after("« Il a dit “oui” et non »"));
}

#[test]
fn leading_closers_of_the_next_sentence_are_taken_into_the_boundary() {
    let mut balance = QuoteBalance::default();
    balance.feed("Il a dit : « Rien ne manque. ");
    assert_eq!(balance.feed_leading_closers("» Puis il part."), "»".len());
    assert!(balance.is_closed());
}

#[test]
fn an_opening_mark_of_the_next_sentence_is_not_a_closer() {
    let mut balance = QuoteBalance::default();
    assert_eq!(balance.feed_leading_closers(" « Bonjour »"), 0);
    assert!(balance.is_closed());
}
