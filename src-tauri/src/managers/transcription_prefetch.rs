//! Pieces of a long dictation transcribed while it is still being recorded
//! (feature 022).
//!
//! The recorder's frames reach a worker thread that keeps its own copy of the
//! recording. Each time a full piece exists (see [`chunks::next_cut`]), the
//! worker transcribes it, so that at the stop only the last piece is left. The
//! stop reuses the pieces only when the recording and the run options it ends
//! with are exactly the ones the worker saw.

use super::chunks::{self, PieceText};
use crate::engine_supervisor::EngineError;
use log::{debug, info};
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use transcribe_cpp::RunOptions;

/// PCM sample rate of every dictation (16 kHz mono).
const SAMPLE_RATE: usize = 16_000;

/// What a piece's text depends on besides its audio.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RunKey {
    /// Id of the model the pieces were transcribed with.
    pub(super) model: String,
    /// Options of the runs.
    pub(super) options: RunOptions,
}

/// What the worker transcribed during one recording.
pub(super) struct Prefetched {
    /// Every frame the worker received, in order.
    audio: Vec<f32>,
    /// Model and options of its runs; `None` when no transcribe-cpp model was loaded.
    key: Option<RunKey>,
    /// Successful pieces with their exact range in `audio`.
    pieces: Vec<(Range<usize>, PieceText)>,
    /// The dictation was cancelled while the worker ran a piece.
    cancelled: bool,
}

impl Prefetched {
    /// Whether the user cancelled the dictation while a piece was running:
    /// the stop must then submit no further work.
    pub(super) fn cancelled(&self) -> bool {
        self.cancelled
    }

    // @spec FR-003: Reuse only for the same audio and options — .specs/features/022-transcribe-while-recording/spec.md#fr-003
    /// The pieces to reuse for `audio` transcribed as `key` says: all of them
    /// when the worker saw a prefix of exactly this audio with the same model
    /// and options, none otherwise.
    pub(super) fn pieces_for(self, audio: &[f32], key: &RunKey) -> Vec<(Range<usize>, PieceText)> {
        let same_audio = audio.get(..self.audio.len()) == Some(self.audio.as_slice());
        if self.key.as_ref() != Some(key) || !same_audio {
            if !self.pieces.is_empty() {
                info!(
                    "Discarding {} pieces transcribed during the recording: the audio, the model or the options changed",
                    self.pieces.len()
                );
            }
            return Vec::new();
        }
        if !self.pieces.is_empty() {
            info!(
                "Reusing {} pieces transcribed during the recording",
                self.pieces.len()
            );
        }
        self.pieces
    }
}

/// A running worker: feed it frames, then [`PrefetchHandle::finish`] it.
pub(super) struct PrefetchHandle {
    frames: mpsc::Sender<Vec<f32>>,
    worker: JoinHandle<Prefetched>,
    /// Set when the dictation is abandoned: the worker runs no further piece.
    cancelled: Arc<AtomicBool>,
}

impl PrefetchHandle {
    /// Hand one recorded frame to the worker.
    pub(super) fn feed(&self, frame: &[f32]) {
        let _ = self.frames.send(frame.to_vec());
    }

    /// Close the recording and wait for the piece in progress, if any.
    pub(super) fn finish(self) -> Option<Prefetched> {
        let started = Instant::now();
        drop(self.frames);
        let prefetched = self.worker.join().ok();
        debug!(
            "Waited {:?} for the pieces transcribed during the recording",
            started.elapsed()
        );
        prefetched
    }

    /// Abandon the dictation: the worker stops after the piece it is running
    /// and never submits another one to the engine.
    pub(super) fn cancel(self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// [`Self::cancel`], keeping the thread so a test can wait for it.
    #[cfg(test)]
    fn cancel_for_test(self) -> JoinHandle<Prefetched> {
        self.cancelled.store(true, Ordering::Release);
        self.worker
    }
}

// @spec FR-001: Pieces are transcribed while the user speaks — .specs/features/022-transcribe-while-recording/spec.md#fr-001
/// Start a worker. `setup` runs on the worker thread (it may wait for the
/// model to load) and gives the model and run options with the function that
/// runs one piece, or `None` when nothing can be transcribed ahead.
pub(super) fn spawn<S, R>(setup: S) -> PrefetchHandle
where
    S: FnOnce() -> Option<(RunKey, R)> + Send + 'static,
    R: FnMut(&[f32], &RunOptions) -> Result<PieceText, EngineError>,
{
    let (frames, received) = mpsc::channel::<Vec<f32>>();
    let cancelled = Arc::new(AtomicBool::new(false));
    let stop = Arc::clone(&cancelled);
    let worker = thread::spawn(move || {
        let setup = setup();
        let mut prefetched = Prefetched {
            audio: Vec::new(),
            key: None,
            pieces: Vec::new(),
            cancelled: false,
        };
        let Some((key, mut run)) = setup else {
            while let Ok(frame) = received.recv() {
                prefetched.audio.extend_from_slice(&frame);
            }
            return prefetched;
        };
        let mut start = 0;
        while let Ok(frame) = received.recv() {
            prefetched.audio.extend_from_slice(&frame);
            while let Some(end) = chunks::next_cut(&prefetched.audio, start) {
                if stop.load(Ordering::Acquire)
                    || !run_piece(&mut prefetched, start..end, &key, &mut run)
                {
                    prefetched.cancelled = true;
                    return prefetched;
                }
                start = end;
            }
        }
        prefetched.key = Some(key);
        prefetched
    });
    PrefetchHandle {
        frames,
        worker,
        cancelled,
    }
}

/// Transcribe `range` of the recording into `prefetched`. Returns `false`
/// when the engine call was cancelled, which ends the work for this dictation.
fn run_piece<R>(prefetched: &mut Prefetched, range: Range<usize>, key: &RunKey, run: &mut R) -> bool
where
    R: FnMut(&[f32], &RunOptions) -> Result<PieceText, EngineError>,
{
    let started = Instant::now();
    let secs = range.len() as f64 / SAMPLE_RATE as f64;
    match run(&prefetched.audio[range.clone()], &key.options) {
        Ok(piece) => {
            debug!(
                "Piece of {secs:.1}s transcribed during the recording in {:?}",
                started.elapsed()
            );
            prefetched.pieces.push((range, piece));
            true
        }
        Err(EngineError::Cancelled) => false,
        Err(e) => {
            debug!("A {secs:.1}s piece transcribed during the recording failed: {e}");
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: usize = 16_000;

    type NoRun = fn(&[f32], &RunOptions) -> Result<PieceText, EngineError>;

    fn tone(secs: usize) -> Vec<f32> {
        (0..secs * SECOND)
            .map(|i| (i as f32 * 0.05).sin() * 0.5)
            .collect()
    }

    fn key() -> RunKey {
        RunKey {
            model: "canary".to_string(),
            options: RunOptions {
                language: Some("fr".to_string()),
                ..Default::default()
            },
        }
    }

    /// Feed `audio` in 20 ms frames to a worker whose runs name the piece range.
    fn record(audio: &[f32]) -> Option<Prefetched> {
        let handle = spawn(|| {
            let mut offset = 0;
            Some((key(), move |piece: &[f32], _: &RunOptions| {
                let text = format!("{offset}+{}", piece.len());
                offset += piece.len();
                Ok(PieceText {
                    text,
                    language: None,
                })
            }))
        });
        for frame in audio.chunks(SECOND / 50) {
            handle.feed(frame);
        }
        handle.finish()
    }

    #[test]
    fn full_pieces_are_transcribed_with_the_final_cuts() -> Result<(), EngineError> {
        let audio = tone(3 * chunks::MAX_CHUNK_SECS + 5);
        let pieces = record(&audio)
            .map(|done| done.pieces_for(&audio, &key()))
            .unwrap_or_default();
        let ranges = chunks::chunk_ranges(&audio);

        assert_eq!(pieces.len(), ranges.len() - 1);
        for ((done, _), expected) in pieces.iter().zip(&ranges) {
            assert_eq!(done, expected);
        }

        let mut runs_at_stop = 0;
        let result = chunks::transcribe_in_chunks(&audio, pieces, &mut |_: &[f32]| {
            runs_at_stop += 1;
            Ok(PieceText {
                text: "last".to_string(),
                language: None,
            })
        })?;
        assert_eq!(runs_at_stop, 1);
        assert!(result.text.ends_with("last"));
        Ok(())
    }

    #[test]
    fn other_audio_or_options_reuse_nothing() {
        let audio = tone(2 * chunks::MAX_CHUNK_SECS + 5);
        let mut other = audio.clone();
        other[0] = 1.0;
        let mut changed = key();
        changed.options.language = Some("en".to_string());
        let mut other_model = key();
        other_model.model = "whisper".to_string();

        let for_other = record(&audio).map(|done| done.pieces_for(&other, &key()));
        let for_changed = record(&audio).map(|done| done.pieces_for(&audio, &changed));
        let for_model = record(&audio).map(|done| done.pieces_for(&audio, &other_model));

        assert_eq!(for_other.map(|p| p.len()), Some(0));
        assert_eq!(for_changed.map(|p| p.len()), Some(0));
        assert_eq!(for_model.map(|p| p.len()), Some(0));
    }

    #[test]
    fn a_cancelled_worker_runs_no_further_piece() {
        let runs = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = Arc::clone(&runs);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let handle = spawn(move || {
            Some((key(), move |_: &[f32], _: &RunOptions| {
                counted.fetch_add(1, Ordering::SeqCst);
                let _ = started_tx.send(());
                let _ = release_rx.recv();
                Ok(PieceText {
                    text: String::new(),
                    language: None,
                })
            }))
        });
        let audio = tone(3 * chunks::MAX_CHUNK_SECS + 5);
        for frame in audio.chunks(SECOND / 50) {
            handle.feed(frame);
        }
        let _ = started_rx.recv();
        let worker = handle.cancel_for_test();
        drop(release_tx);
        let _ = worker.join();
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_cancelled_engine_call_ends_the_worker() {
        let runs = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = Arc::clone(&runs);
        let handle = spawn(move || {
            Some((key(), move |_: &[f32], _: &RunOptions| {
                counted.fetch_add(1, Ordering::SeqCst);
                Err(EngineError::Cancelled)
            }))
        });
        let audio = tone(3 * chunks::MAX_CHUNK_SECS + 5);
        for frame in audio.chunks(SECOND / 50) {
            handle.feed(frame);
        }
        let cancelled = handle.finish().map(|done| done.cancelled());
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        assert_eq!(cancelled, Some(true));
    }

    #[test]
    fn without_a_model_nothing_is_transcribed_ahead() {
        let audio = tone(2 * chunks::MAX_CHUNK_SECS + 5);
        let handle = spawn(|| -> Option<(RunKey, NoRun)> { None });
        for frame in audio.chunks(SECOND / 50) {
            handle.feed(frame);
        }
        let pieces = handle.finish().map(|done| done.pieces_for(&audio, &key()));
        assert_eq!(pieces.map(|p| p.len()), Some(0));
    }
}
