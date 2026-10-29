//! Comparison of the remembered selection range with the input's selection now (feature 011,
//! native finding N18). Chromium and Electron answer a range write at once and apply it a moment
//! later, so the attempt reads the range back, bounded, until it settles; the guard then compares
//! once more. The comparison says only which parts differ, never a position, a length or content
//! (spec AC-008), so it can be logged.
use super::field::{FieldNode, SelectionRange, MIN_AX_TIMEOUT};
use super::workspace::Pause;
use crate::recovery::classifier::{AxQueryError, Deadline};
use std::time::Duration;

/// Wait between two read-backs while a range write settles.
pub const SETTLE_PERIOD: Duration = Duration::from_millis(5);

/// How the input's selection compares with the remembered one (logged; no value).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeSeen {
    /// Not compared (the attempt did not restore the range, or another element is focused).
    NotRead,
    /// Exactly the remembered selection.
    Same,
    /// The remembered caret (nothing selected) and a caret now, at another position.
    CaretMoved,
    /// Another selection: which parts differ and whether each side is a caret.
    Differs {
        same_location: bool,
        same_length: bool,
        remembered_caret: bool,
        seen_caret: bool,
    },
    /// Not equal, per the platform's own comparison (Windows text range endpoints).
    NotSame,
    /// The range could not be read (error, deadline).
    Unreadable(AxQueryError),
}

impl RangeSeen {
    // @spec AC-004: A moved caret pastes, a selection never — .specs/features/011-dictation-field-return/spec.md#ac-004
    /// Business rule (spec AC-004, decision `caret-moved-tolerance`): the remembered input may
    /// receive the paste when its selection is the remembered one, or when the remembered
    /// selection was a caret and the input still has only a caret, elsewhere: nothing is selected,
    /// so nothing can be overwritten. A non-empty selection on either side must be exact.
    pub fn accepts(self) -> bool {
        matches!(self, Self::Same | Self::CaretMoved)
    }
}

/// `seen` against the remembered offsets `remembered` (position, length).
fn classify(remembered: (usize, usize), seen: Option<(usize, usize)>) -> RangeSeen {
    let Some(seen) = seen else {
        return RangeSeen::NotSame;
    };
    if seen == remembered {
        return RangeSeen::Same;
    }
    let (remembered_caret, seen_caret) = (remembered.1 == 0, seen.1 == 0);
    if remembered_caret && seen_caret {
        return RangeSeen::CaretMoved;
    }
    RangeSeen::Differs {
        same_location: seen.0 == remembered.0,
        same_length: seen.1 == remembered.1,
        remembered_caret,
        seen_caret,
    }
}

/// The input's selection now against `remembered`, one bounded read.
pub(crate) fn compare(
    element: &mut dyn FieldNode,
    remembered: &SelectionRange,
    timeout: Duration,
) -> RangeSeen {
    match remembered.as_offsets() {
        Some(offsets) => match element.read_range(timeout) {
            Ok(seen) => classify(offsets, seen.as_offsets()),
            Err(error) => RangeSeen::Unreadable(error),
        },
        None => match element.range_is(remembered, timeout) {
            Ok(true) => RangeSeen::Same,
            Ok(false) => RangeSeen::NotSame,
            Err(error) => RangeSeen::Unreadable(error),
        },
    }
}

/// After a range write answered at once: read the range back every `SETTLE_PERIOD` until it is
/// the remembered one or `deadline` leaves no room for another read. Reads only; returns the
/// last comparison and the number of reads (both logged).
pub(crate) fn settle(
    element: &mut dyn FieldNode,
    remembered: &SelectionRange,
    deadline: &Deadline<'_>,
    pause: &dyn Pause,
) -> (RangeSeen, u8) {
    let mut seen = RangeSeen::NotRead;
    let mut reads = 0u8;
    while let Ok(timeout) = deadline.remaining() {
        seen = compare(element, remembered, timeout);
        reads = reads.saturating_add(1);
        let room = deadline.remaining().unwrap_or(Duration::ZERO);
        if seen == RangeSeen::Same || room < SETTLE_PERIOD + MIN_AX_TIMEOUT {
            break;
        }
        pause.pause(SETTLE_PERIOD);
    }
    (seen, reads)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_exact_selection_or_two_carets_are_accepted() {
        assert_eq!(classify((4, 0), Some((4, 0))), RangeSeen::Same);
        assert_eq!(classify((4, 2), Some((4, 2))), RangeSeen::Same);
        assert_eq!(classify((4, 0), Some((9, 0))), RangeSeen::CaretMoved);
        assert!(classify((4, 0), Some((9, 0))).accepts());
        // A selection now, or a selection remembered: never accepted unless exact.
        for (remembered, seen) in [((4, 0), (0, 4)), ((4, 2), (4, 0)), ((4, 2), (5, 2))] {
            let compared = classify(remembered, Some(seen));
            assert!(!compared.accepts(), "{remembered:?} {seen:?} {compared:?}");
        }
        assert_eq!(
            classify((4, 0), Some((4, 3))),
            RangeSeen::Differs {
                same_location: true,
                same_length: false,
                remembered_caret: true,
                seen_caret: false,
            }
        );
        assert!(!RangeSeen::NotSame.accepts());
        assert!(!RangeSeen::NotRead.accepts());
        assert!(!RangeSeen::Unreadable(AxQueryError::Timeout).accepts());
    }

    #[test]
    fn the_comparison_never_carries_a_position_or_a_length() {
        let logged = format!("{:?}", classify((1234, 0), Some((5678, 91))));
        for value in ["1234", "5678", "91"] {
            assert!(!logged.contains(value), "{logged}");
        }
    }
}
