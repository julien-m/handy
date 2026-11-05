//! Long dictations through transcribe-cpp (feature 020).
//!
//! An encoder-decoder model given minutes of audio at once tends to start
//! repeating itself, and transcribe-cpp then stops the run. A long recording is
//! therefore cut at its quietest moments into pieces of at most
//! [`MAX_CHUNK_SECS`], each piece is transcribed in turn and the texts are
//! joined. A piece that still stops early is cut in two and retried; a piece too
//! short to cut keeps the text decoded before it stopped.

use crate::engine_supervisor::EngineError;
use log::{info, warn};
use std::ops::Range;

/// PCM sample rate of every dictation (16 kHz mono).
const SAMPLE_RATE: usize = 16_000;
/// Longest piece sent to the model in one run.
pub(super) const MAX_CHUNK_SECS: usize = 30;
/// A cut is searched for in the last [`CUT_WINDOW_SECS`] of a full piece.
const CUT_WINDOW_SECS: usize = 10;
/// A piece that stopped early is cut in two only when both halves keep at
/// least this much audio.
const MIN_PIECE_SECS: usize = 10;
/// Window over which the loudness is measured to find a quiet moment (20 ms).
const QUIET_WINDOW: usize = SAMPLE_RATE / 50;

/// Text and audio-detected language of one run.
pub(super) struct PieceText {
    pub text: String,
    pub language: Option<String>,
}

/// The joined text of a recording and the first language a run detected.
pub(super) struct ChunkedText {
    pub text: String,
    pub language: Option<String>,
}

// @spec FR-001: Cut long dictations at quiet moments — .specs/features/020-long-dictation-chunks/spec.md#fr-001
/// Pieces of `audio`, in order and covering every sample: one piece when the
/// recording fits in [`MAX_CHUNK_SECS`], otherwise each cut falls on the
/// quietest moment of the last [`CUT_WINDOW_SECS`] before the limit.
pub(super) fn chunk_ranges(audio: &[f32]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while let Some(end) = next_cut(audio, start) {
        ranges.push(start..end);
        start = end;
    }
    ranges.push(start..audio.len());
    ranges
}

// @spec FR-001: A cut depends only on the audio before it — .specs/features/022-transcribe-while-recording/spec.md#fr-001
/// End of the piece starting at `start`, or `None` while the audio after
/// `start` still fits in one piece. The cut reads no sample past
/// `start + MAX_CHUNK_SECS`, so a recording still growing is cut exactly as
/// the finished one will be.
pub(super) fn next_cut(audio: &[f32], start: usize) -> Option<usize> {
    let max = MAX_CHUNK_SECS * SAMPLE_RATE;
    let window = CUT_WINDOW_SECS * SAMPLE_RATE;
    (audio.len().saturating_sub(start) > max)
        .then(|| quietest_point(audio, start + max - window..start + max))
}

/// Start of the quietest [`QUIET_WINDOW`] inside `search`, or the middle of
/// `search` when it is shorter than one window.
fn quietest_point(audio: &[f32], search: Range<usize>) -> usize {
    let last = search.end.saturating_sub(QUIET_WINDOW);
    (search.start..=last)
        .step_by(QUIET_WINDOW)
        .min_by(|&a, &b| loudness(audio, a).total_cmp(&loudness(audio, b)))
        .unwrap_or((search.start + search.end) / 2)
}

fn loudness(audio: &[f32], start: usize) -> f32 {
    audio[start..(start + QUIET_WINDOW).min(audio.len())]
        .iter()
        .map(|sample| sample.abs())
        .sum()
}

/// Runs pieces through the model: one at a time, or several in one call.
pub(super) trait PieceEngine {
    fn run(&mut self, piece: &[f32]) -> Result<PieceText, EngineError>;

    /// One result per piece, in order; the call fails only when no piece could
    /// run. By default the pieces run one after another.
    fn run_many(
        &mut self,
        pieces: &[&[f32]],
    ) -> Result<Vec<Result<PieceText, EngineError>>, EngineError> {
        Ok(pieces.iter().map(|piece| self.run(piece)).collect())
    }
}

impl<F> PieceEngine for F
where
    F: FnMut(&[f32]) -> Result<PieceText, EngineError>,
{
    fn run(&mut self, piece: &[f32]) -> Result<PieceText, EngineError> {
        self(piece)
    }
}

// @spec FR-002: A piece that loops is cut again, then keeps its text — .specs/features/020-long-dictation-chunks/spec.md#fr-002
// @spec FR-002: Pieces transcribed during the recording are reused — .specs/features/022-transcribe-while-recording/spec.md#fr-002
// @spec FR-005: Remaining pieces in one batch — .specs/features/022-transcribe-while-recording/spec.md#fr-005
/// Transcribe `audio` piece by piece, reusing the pieces of `ready` (already
/// transcribed, with their exact range) and sending the others to `engine` in
/// one batch when there are several. A cancelled or failed run stops the whole
/// transcription; a run that stopped early is handled by [`resolve`].
pub(super) fn transcribe_in_chunks(
    audio: &[f32],
    ready: Vec<(Range<usize>, PieceText)>,
    engine: &mut impl PieceEngine,
) -> Result<ChunkedText, EngineError> {
    let ranges = chunk_ranges(audio);
    if ranges.len() > 1 {
        info!(
            "Long recording ({:.1}s): transcribing it in {} pieces",
            audio.len() as f64 / SAMPLE_RATE as f64,
            ranges.len()
        );
    }
    let mut ready = ready.into_iter().peekable();
    let mut pieces: Vec<Option<PieceText>> = ranges
        .iter()
        .map(|range| {
            ready
                .next_if(|(done, _)| done == range)
                .map(|(_, piece)| piece)
        })
        .collect();
    let missing: Vec<usize> = (0..ranges.len()).filter(|&i| pieces[i].is_none()).collect();
    if missing.len() > 1 {
        info!("Transcribing {} pieces in one batch", missing.len());
        let slices: Vec<&[f32]> = missing.iter().map(|&i| &audio[ranges[i].clone()]).collect();
        let results = engine.run_many(&slices)?;
        for (&i, result) in missing.iter().zip(results) {
            pieces[i] = Some(resolve(&audio[ranges[i].clone()], result, engine)?);
        }
    } else if let Some(&i) = missing.first() {
        pieces[i] = Some(piece_text(&audio[ranges[i].clone()], engine)?);
    }
    let mut texts = Vec::new();
    let mut language = None;
    for piece in pieces.into_iter().flatten() {
        language = language.or(piece.language);
        texts.push(piece.text);
    }
    Ok(ChunkedText {
        text: join_texts(texts),
        language,
    })
}

fn piece_text(piece: &[f32], engine: &mut impl PieceEngine) -> Result<PieceText, EngineError> {
    let result = engine.run(piece);
    resolve(piece, result, engine)
}

/// The text of `piece` from the result of its run: a run that stopped early is
/// cut in two and run again, or keeps its partial text when too short to cut.
fn resolve(
    piece: &[f32],
    result: Result<PieceText, EngineError>,
    engine: &mut impl PieceEngine,
) -> Result<PieceText, EngineError> {
    let (message, partial) = match result {
        Err(EngineError::Incomplete { message, partial }) => (message, partial),
        other => return other,
    };
    let min = MIN_PIECE_SECS * SAMPLE_RATE;
    let secs = piece.len() as f64 / SAMPLE_RATE as f64;
    if piece.len() < 2 * min {
        let kept = partial.as_deref().map_or(0, |text| text.chars().count());
        warn!("A {secs:.1}s piece stopped early ({message}); keeping the {kept} characters decoded before it stopped");
        return Ok(PieceText {
            text: partial.unwrap_or_default(),
            language: None,
        });
    }
    warn!("A {secs:.1}s piece stopped early ({message}); cutting it in two");
    let cut = quietest_point(piece, min..piece.len() - min);
    let first = piece_text(&piece[..cut], engine)?;
    let second = piece_text(&piece[cut..], engine)?;
    Ok(PieceText {
        text: join_texts([first.text, second.text]),
        language: first.language.or(second.language),
    })
}

fn join_texts(texts: impl IntoIterator<Item = String>) -> String {
    texts
        .into_iter()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tone of `secs` seconds with silent gaps starting at `gaps` (seconds).
    fn speech_with_gaps(secs: usize, gaps: &[usize]) -> Vec<f32> {
        let mut audio: Vec<f32> = (0..secs * SAMPLE_RATE)
            .map(|i| (i as f32 * 0.05).sin() * 0.5)
            .collect();
        for &gap in gaps {
            let start = gap * SAMPLE_RATE;
            audio[start..start + SAMPLE_RATE / 2].fill(0.0);
        }
        audio
    }

    fn secs(samples: usize) -> f64 {
        samples as f64 / SAMPLE_RATE as f64
    }

    #[test]
    fn short_recording_is_one_piece() {
        let audio = speech_with_gaps(MAX_CHUNK_SECS, &[]);
        assert_eq!(chunk_ranges(&audio), vec![0..audio.len()]);
    }

    #[test]
    fn long_recording_is_cut_in_its_silences_and_keeps_every_sample() {
        let gap = MAX_CHUNK_SECS - CUT_WINDOW_SECS / 2;
        let audio = speech_with_gaps(3 * MAX_CHUNK_SECS, &[gap, 2 * gap]);
        let ranges = chunk_ranges(&audio);

        assert!(ranges.len() > 1);
        assert_eq!(ranges.first().map(|r| r.start), Some(0));
        assert_eq!(ranges.last().map(|r| r.end), Some(audio.len()));
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
        for range in &ranges {
            assert!(secs(range.len()) <= MAX_CHUNK_SECS as f64);
        }
        let first_cut = ranges[0].end;
        assert!(audio[first_cut..first_cut + QUIET_WINDOW]
            .iter()
            .all(|sample| *sample == 0.0));
    }

    fn piece(text: &str) -> Result<PieceText, EngineError> {
        Ok(PieceText {
            text: text.to_string(),
            language: Some("fr".to_string()),
        })
    }

    fn looping(partial: &str) -> Result<PieceText, EngineError> {
        Err(EngineError::Incomplete {
            message: "output repetition".to_string(),
            partial: Some(partial.to_string()),
        })
    }

    #[test]
    fn pieces_are_joined_in_order() -> Result<(), EngineError> {
        let audio = speech_with_gaps(2 * MAX_CHUNK_SECS + 5, &[]);
        let expected = chunk_ranges(&audio).len();
        let mut calls = 0;
        let result = transcribe_in_chunks(&audio, Vec::new(), &mut |_: &[f32]| {
            calls += 1;
            piece(&format!(" part{calls} "))
        })?;

        assert_eq!(calls, expected);
        let words: Vec<String> = (1..=expected).map(|n| format!("part{n}")).collect();
        assert_eq!(result.text, words.join(" "));
        assert_eq!(result.language.as_deref(), Some("fr"));
        Ok(())
    }

    #[test]
    fn a_looping_piece_is_cut_in_two() -> Result<(), EngineError> {
        let audio = speech_with_gaps(MAX_CHUNK_SECS, &[]);
        let mut lengths = Vec::new();
        let result = transcribe_in_chunks(&audio, Vec::new(), &mut |piece: &[f32]| {
            lengths.push(piece.len());
            if piece.len() == audio.len() {
                looping("lost")
            } else {
                piece_text_for(piece.len())
            }
        })?;

        assert_eq!(lengths.len(), 3);
        assert_eq!(lengths[1] + lengths[2], audio.len());
        assert!(lengths[1..]
            .iter()
            .all(|&n| n >= MIN_PIECE_SECS * SAMPLE_RATE));
        assert_eq!(result.text, format!("{} {}", lengths[1], lengths[2]));
        Ok(())
    }

    fn piece_text_for(len: usize) -> Result<PieceText, EngineError> {
        piece(&len.to_string())
    }

    #[test]
    fn a_short_looping_piece_keeps_its_partial_text() -> Result<(), EngineError> {
        let audio = speech_with_gaps(2 * MIN_PIECE_SECS - 1, &[]);
        let mut calls = 0;
        let result = transcribe_in_chunks(&audio, Vec::new(), &mut |_: &[f32]| {
            calls += 1;
            looping(" before the loop ")
        })?;

        assert_eq!(calls, 1);
        assert_eq!(result.text, "before the loop");
        Ok(())
    }

    #[test]
    fn cancel_and_failure_stop_the_transcription() {
        let audio = speech_with_gaps(2 * MAX_CHUNK_SECS + 5, &[]);
        let mut calls = 0;
        let cancelled = transcribe_in_chunks(&audio, Vec::new(), &mut |_: &[f32]| {
            calls += 1;
            Err(EngineError::Cancelled)
        });
        assert!(matches!(cancelled, Err(EngineError::Cancelled)));
        assert!(calls >= 1);

        let failed = transcribe_in_chunks(&audio, Vec::new(), &mut |_: &[f32]| {
            Err(EngineError::Failed("engine stopped".to_string()))
        });
        assert!(matches!(failed, Err(EngineError::Failed(_))));
    }

    /// Fake engine: every batch is recorded; a piece of `loop_len` samples loops.
    struct Batching {
        batches: Vec<usize>,
        single_runs: usize,
        loop_len: usize,
    }

    impl PieceEngine for Batching {
        fn run(&mut self, piece: &[f32]) -> Result<PieceText, EngineError> {
            self.single_runs += 1;
            piece_text_for(piece.len())
        }

        fn run_many(
            &mut self,
            pieces: &[&[f32]],
        ) -> Result<Vec<Result<PieceText, EngineError>>, EngineError> {
            self.batches.push(pieces.len());
            Ok(pieces
                .iter()
                .map(|piece| {
                    if piece.len() == self.loop_len {
                        looping("lost")
                    } else {
                        piece_text_for(piece.len())
                    }
                })
                .collect())
        }
    }

    #[test]
    fn missing_pieces_go_in_one_batch_and_a_looping_one_is_cut() -> Result<(), EngineError> {
        let audio = speech_with_gaps(3 * MAX_CHUNK_SECS + 5, &[]);
        let ranges = chunk_ranges(&audio);
        let ready = vec![(
            ranges[0].clone(),
            PieceText {
                text: "ready".to_string(),
                language: None,
            },
        )];
        let mut engine = Batching {
            batches: Vec::new(),
            single_runs: 0,
            loop_len: ranges[1].len(),
        };
        let result = transcribe_in_chunks(&audio, ready, &mut engine)?;

        assert_eq!(engine.batches, vec![ranges.len() - 1]);
        assert_eq!(engine.single_runs, 2);
        assert!(result.text.starts_with("ready "));
        assert!(!result.text.contains("lost"));
        Ok(())
    }
}
