//! Correlation identifiers and the private transcript carried by one insertion.
//!
//! Identifiers let automatic Enter (009) and the recovery bubble (010)
//! correlate outcomes and late-read diagnostics without ever seeing the text.

use std::fmt;

/// Identifies one dictation from transcription to insertion outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DictationId(pub u64);

/// Identifies one smart-paste transaction (one published on-demand item).
///
/// Reads served for an item of an earlier transaction never start or delay
/// the restoration of the current one (spec Clarifications, read attribution).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct TransactionId(pub u64);

/// Destination generation supplied by the destination feature (006).
///
/// The coordinator only carries and compares it; 006 owns its meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct DestinationGeneration(pub u64);

/// Final transcript text after trailing-space handling.
///
/// `Debug` never prints the content: transcripts must not reach any log.
#[derive(Clone, PartialEq, Eq)]
pub struct Transcript(String);

impl Transcript {
    /// Wraps the final text that smart paste must insert.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Returns the text for native adapters; callers must not log it.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the text as UTF-16 code units, the unit of AX ranges.
    pub fn to_utf16(&self) -> Vec<u16> {
        self.0.encode_utf16().collect()
    }

    /// Returns the UTF-8 length, used for byte-bounded bookkeeping.
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for Transcript {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Transcript(<{} bytes redacted>)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_debug_never_contains_the_text() {
        let transcript = Transcript::new("secret dictation");
        let printed = format!("{transcript:?}");
        assert!(!printed.contains("secret"));
        assert!(printed.contains("16 bytes"));
    }

    #[test]
    fn transcript_counts_utf16_units_for_astral_characters() {
        let transcript = Transcript::new("a😀");
        assert_eq!(transcript.to_utf16().len(), 3);
    }
}
