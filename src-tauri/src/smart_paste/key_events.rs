//! Key-event port: the Cmd+V chord and Unicode typing (AC-008, AC-010).
//!
//! Posting succeeds or fails; success never means the field received the
//! text. The caller records the keystroke instant immediately before
//! [`KeyEventPort::post_paste_chord`], after resolving the key.

use std::time::Duration;

use super::keyboard::KeyboardError;

/// Modifier hold after the V key, at parity with the existing paste paths
/// (#165: some systems drop chords released too quickly). Reads served
/// while it is held count as qualifying.
pub const CHORD_HOLD: Duration = Duration::from_millis(100);
/// Unicode string length per key event: macOS truncates longer strings
/// attached to one keyboard event (20 UTF-16 units is the documented limit
/// used by Apple samples and enigo).
pub const MAX_UNICODE_UNITS_PER_EVENT: usize = 20;

/// Virtual key code that types `v` with Command in the active layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasteKey(pub u16);

/// Key-event posting.
pub trait KeyEventPort: Send {
    /// Resolves the V key for the layout active now (after any switch).
    fn resolve_paste_key(&mut self) -> Result<PasteKey, KeyboardError>;
    /// Posts Command down, V down, V up, waits `hold`, Command up.
    fn post_paste_chord(&mut self, key: PasteKey, hold: Duration) -> Result<(), KeyboardError>;
    /// Types `text` as Unicode key events without the clipboard.
    fn type_unicode(&mut self, text: &str) -> Result<(), KeyboardError>;
}

/// Splits UTF-16 text into event-sized chunks without splitting a surrogate
/// pair across two events.
pub fn unicode_chunks(units: &[u16], max_units: usize) -> Vec<&[u16]> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < units.len() {
        let mut end = (start + max_units.max(2)).min(units.len());
        let is_high_surrogate = |unit: u16| (0xD800..=0xDBFF).contains(&unit);
        // A chunk ending on a high surrogate would separate it from its low
        // surrogate: end one unit earlier.
        if end < units.len() && is_high_surrogate(units[end - 1]) {
            end -= 1;
        }
        chunks.push(&units[start..end]);
        start = end;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn long_text_is_split_into_event_sized_chunks() {
        let units = utf16(&"a".repeat(45));
        let sizes: Vec<_> = unicode_chunks(&units, 20).iter().map(|c| c.len()).collect();
        assert_eq!(sizes, vec![20, 20, 5]);
    }

    #[test]
    fn surrogate_pairs_are_never_split() {
        let text = format!("{}🦀b", "a".repeat(19));
        let units = utf16(&text);
        let chunks = unicode_chunks(&units, 20);
        let rebuilt: Vec<String> = chunks.iter().map(|c| String::from_utf16_lossy(c)).collect();
        assert_eq!(rebuilt, vec!["a".repeat(19), "🦀b".to_string()]);
    }

    #[test]
    fn empty_text_has_no_events() {
        assert!(unicode_chunks(&[], 20).is_empty());
    }
}
