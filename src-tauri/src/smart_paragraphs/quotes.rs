//! Quotation balance across sentences (feature 016 AC-006): a paragraph break is never placed
//! inside an open quotation.

/// The closing marks still expected, innermost last.
#[derive(Debug, Default)]
pub(crate) struct QuoteBalance {
    expected: Vec<char>,
}

impl QuoteBalance {
    /// Update the balance with the characters of `text`, in order.
    ///
    /// Each opening mark pushes the mark that closes it; a mark equal to the innermost expected
    /// closer pops it. `"` and `“` both open and close: they close when they are the expected
    /// closer (`„…“` in German, `"…"` everywhere) and open otherwise. Unpaired closers are
    /// ignored. Apostrophes and single quotes are ignored on purpose: they are ambiguous with
    /// elisions ("l'ami", "don't").
    pub(crate) fn feed(&mut self, text: &str) {
        for mark in text.chars() {
            if self.expected.last() == Some(&mark) {
                self.expected.pop();
            } else if let Some(closer) = closer_of(mark) {
                self.expected.push(closer);
            }
        }
    }

    /// Consume the expected closing marks (and the whitespace between them) at the start of
    /// `text`; returns the bytes consumed, up to the end of the last mark closed (0 when none).
    ///
    /// Stops at the first character that is neither whitespace nor the innermost expected
    /// closer, so an opening mark of the next sentence is left to [`Self::feed`].
    pub(crate) fn feed_leading_closers(&mut self, text: &str) -> usize {
        let mut closed_up_to = 0;
        for (offset, mark) in text.char_indices() {
            if self.expected.last() == Some(&mark) {
                self.expected.pop();
                closed_up_to = offset + mark.len_utf8();
            } else if !mark.is_whitespace() {
                break;
            }
        }
        closed_up_to
    }

    /// True when every quotation opened so far is closed.
    pub(crate) fn is_closed(&self) -> bool {
        self.expected.is_empty()
    }
}

/// The mark that closes the quotation opened by `mark`, if `mark` opens one.
fn closer_of(mark: char) -> Option<char> {
    match mark {
        '"' => Some('"'),
        '“' => Some('”'),
        '„' => Some('“'),
        '«' => Some('»'),
        '「' => Some('」'),
        '『' => Some('』'),
        _ => None,
    }
}

#[cfg(test)]
#[path = "quotes_tests.rs"]
mod tests;
