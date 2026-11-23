//! Assembles the ChatGPT dictation transcript from per-utterance revisions.
//!
//! The server sends, per utterance, provisional `transcript.segment` revisions
//! then one `transcript.final`. Like the ChatGPT client, Handy keeps only the
//! highest revision of each utterance, prefers a final over any segment, and
//! joins utterances in arrival order with single spaces.

use std::collections::HashMap;

use crate::transcription_engine::LiveText;

#[derive(Default)]
struct Utterance {
    final_text: Option<(u64, String)>,
    partial: Option<(u64, String)>,
}

/// Latest known text of every utterance of one session.
#[derive(Default)]
pub(super) struct Transcript {
    order: Vec<String>,
    utterances: HashMap<String, Utterance>,
}

impl Transcript {
    fn entry(&mut self, id: &str) -> &mut Utterance {
        if !self.utterances.contains_key(id) {
            self.order.push(id.to_owned());
        }
        self.utterances.entry(id.to_owned()).or_default()
    }

    /// Record a provisional revision; ignored once the utterance is final or
    /// when an equal or newer revision is already known.
    pub(super) fn segment(&mut self, id: &str, revision: u64, text: &str) {
        let utterance = self.entry(id);
        let is_newer = utterance
            .partial
            .as_ref()
            .is_none_or(|(known, _)| revision >= *known);
        if utterance.final_text.is_none() && is_newer {
            utterance.partial = Some((revision, text.trim().to_owned()));
        }
    }

    /// Record the definitive text of an utterance.
    pub(super) fn finalize(&mut self, id: &str, revision: u64, text: &str) {
        let utterance = self.entry(id);
        if utterance
            .final_text
            .as_ref()
            .is_none_or(|(known, _)| revision >= *known)
        {
            utterance.final_text = Some((revision, text.trim().to_owned()));
            utterance.partial = None;
        }
    }

    /// Final utterances up to the first open one are committed; everything
    /// after it is tentative, because the open utterance may still change.
    pub(super) fn live(&self) -> LiveText {
        let mut committed = Vec::new();
        let mut tentative = Vec::new();
        for id in &self.order {
            let utterance = &self.utterances[id];
            match (&utterance.final_text, &utterance.partial) {
                (Some((_, text)), _) if tentative.is_empty() => committed.push(text.as_str()),
                (Some((_, text)), _) | (None, Some((_, text))) => tentative.push(text.as_str()),
                (None, None) => {}
            }
        }
        LiveText {
            committed: join_words(committed),
            tentative: join_words(tentative),
        }
    }

    /// Full text, including the latest provisional text of open utterances.
    pub(super) fn text(&self) -> String {
        let live = self.live();
        join_words(vec![live.committed.as_str(), live.tentative.as_str()])
    }
}

fn join_words(parts: Vec<&str>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_keeps_latest_revision_until_final() {
        let mut transcript = Transcript::default();
        transcript.segment("a", 1, " Bonjour");
        transcript.segment("a", 2, " Bonjour, ceci");
        transcript.segment("a", 1, " stale");
        assert_eq!(
            transcript.live(),
            LiveText {
                committed: String::new(),
                tentative: "Bonjour, ceci".into()
            }
        );
        transcript.finalize("a", 3, "Bonjour, ceci est un test.");
        transcript.segment("a", 4, "ignored after final");
        transcript.segment("b", 1, " Deuxième");
        assert_eq!(
            transcript.live(),
            LiveText {
                committed: "Bonjour, ceci est un test.".into(),
                tentative: "Deuxième".into()
            }
        );
        assert_eq!(transcript.text(), "Bonjour, ceci est un test. Deuxième");
    }

    #[test]
    fn final_after_open_utterance_stays_tentative() {
        let mut transcript = Transcript::default();
        transcript.segment("a", 1, "un");
        transcript.finalize("b", 1, "deux");
        assert_eq!(transcript.live().committed, "");
        assert_eq!(transcript.text(), "un deux");
    }

    #[test]
    fn empty_transcript_has_empty_text() {
        assert_eq!(Transcript::default().text(), "");
    }
}
