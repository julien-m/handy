# Implementation: Long Dictations Transcribed While Recorded

| Requirement | Location |
|---|---|
| FR-001, AC-001 | `src-tauri/src/managers/transcription_chunks.rs` (`next_cut`), `src-tauri/src/managers/transcription_prefetch.rs` (`spawn`), `src-tauri/src/managers/transcription.rs` (`start_prefetch`, `StreamRouter::feed`), `src-tauri/src/actions.rs` |
| FR-002, AC-002, AC-004 | `src-tauri/src/managers/transcription.rs` (`transcribe_dictation`, `transcribe_with`, `transcribe_cpp`), `src-tauri/src/managers/transcription_chunks.rs` (`transcribe_in_chunks`) |
| FR-003, AC-002, AC-005 | `src-tauri/src/managers/transcription_prefetch.rs` (`Prefetched::pieces_for`), `src-tauri/src/managers/transcription.rs` (`cpp_run`) |
| AC-003 | `src-tauri/src/managers/transcription.rs` (`cancel_stream`), `src-tauri/src/managers/transcription_prefetch.rs` |
| FR-004 | `src-tauri/src/cli.rs` (`live`), `src-tauri/src/lib.rs` (`replay_at_recording_pace`) |
| FR-005, AC-006 | `src-tauri/src/engine_supervisor/batch.rs`, `protocol.rs` (`Request::RunBatch`, `Response::Batch`), `worker.rs`, `supervisor.rs` (`transcribe_batch`, `run_work`), `src-tauri/src/managers/transcription_chunks.rs` (`PieceEngine`), `src-tauri/src/managers/transcription.rs` (`CppPieces`) |
| FR-006, AC-007 | `src-tauri/Info.plist` (`NSAppSleepDisabled`) |

Tests: `transcription_chunks.rs`, `transcription_prefetch.rs`, `engine_supervisor/batch.rs` (tests).
