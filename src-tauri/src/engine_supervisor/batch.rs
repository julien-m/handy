//! Several pieces in one engine run (feature 022): the answer the worker sends
//! for a [`RunBatch`](super::protocol::Request::RunBatch) and how the supervisor reads each piece.

use super::protocol::Response;
use super::EngineError;
use serde::{Deserialize, Serialize};
use transcribe_cpp::Transcript;

/// The result of one piece of a batch.
#[derive(Debug, Serialize, Deserialize)]
pub enum BatchItem {
    Transcript(Transcript),
    /// The piece stopped early; see [`Response::Incomplete`].
    Incomplete {
        message: String,
        partial: Option<String>,
    },
    Error(String),
}

impl BatchItem {
    /// The piece's result as the supervisor reports a single run.
    pub fn into_result(self) -> Result<Transcript, EngineError> {
        match self {
            BatchItem::Transcript(transcript) => Ok(transcript),
            BatchItem::Incomplete { message, partial } => {
                Err(EngineError::Incomplete { message, partial })
            }
            BatchItem::Error(message) => Err(EngineError::Failed(message)),
        }
    }
}

// @spec FR-005: Remaining pieces in one batch — .specs/features/022-transcribe-while-recording/spec.md#fr-005
/// The answer to a [`RunBatch`](super::protocol::Request::RunBatch): `run` transcribes the pieces cut
/// from `pcm` by `lens`; a length mismatch is a plain error.
pub fn batch_response(
    pcm: &[f32],
    lens: &[usize],
    run: impl FnOnce(&[&[f32]]) -> transcribe_cpp::Result<Vec<transcribe_cpp::Result<Transcript>>>,
) -> Response {
    let mut pieces = Vec::with_capacity(lens.len());
    let mut rest = pcm;
    for &len in lens {
        let Some((piece, tail)) = rest.split_at_checked(len) else {
            return Response::Error("batch lengths exceed the audio".to_string());
        };
        pieces.push(piece);
        rest = tail;
    }
    if !rest.is_empty() {
        return Response::Error("batch lengths do not cover the audio".to_string());
    }
    match run(&pieces) {
        Ok(results) => Response::Batch(
            results
                .into_iter()
                .map(|result| match result {
                    Ok(transcript) => BatchItem::Transcript(transcript),
                    Err(e) => match Response::from_run_error(e) {
                        Response::Incomplete { message, partial } => {
                            BatchItem::Incomplete { message, partial }
                        }
                        Response::Error(message) => BatchItem::Error(message),
                        other => BatchItem::Error(format!("unexpected answer: {other:?}")),
                    },
                })
                .collect(),
        ),
        Err(e) => Response::from_run_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_pieces_follow_their_lengths() {
        let pcm: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let lens = [3, 7];
        let mut seen = Vec::new();
        let response = batch_response(&pcm, &lens, |pieces| {
            seen = pieces.iter().map(|piece| piece.to_vec()).collect();
            Ok(Vec::new())
        });
        assert!(matches!(response, Response::Batch(items) if items.is_empty()));
        assert_eq!(seen, vec![pcm[..3].to_vec(), pcm[3..].to_vec()]);

        for wrong in [[3, 8], [3, 6]] {
            let response = batch_response(&pcm, &wrong, |_| Ok(Vec::new()));
            assert!(matches!(response, Response::Error(_)));
        }
    }
}
