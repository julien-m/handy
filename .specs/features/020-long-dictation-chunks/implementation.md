# Implementation: Long Dictations Transcribed in Pieces

| Requirement | Location |
|---|---|
| FR-001, AC-001, AC-004, AC-005 | `src-tauri/src/managers/transcription_chunks.rs` (`chunk_ranges`, `transcribe_in_chunks`), `src-tauri/src/managers/transcription.rs` (`transcribe_cpp`) |
| FR-002, AC-002, AC-003, AC-005 | `src-tauri/src/managers/transcription_chunks.rs` (`piece_text`), `src-tauri/src/engine_supervisor/worker.rs` (`run_failure`), `src-tauri/src/engine_supervisor/protocol.rs` (`Response::Incomplete`), `src-tauri/src/engine_supervisor/supervisor.rs` (`EngineError::Incomplete`) |

Tests: `src-tauri/src/managers/transcription_chunks.rs` (tests).
