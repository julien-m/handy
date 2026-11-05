# Changelog — 022 Long Dictations Transcribed While Recorded

## 2026-11-05

- Pieces still missing at the stop go to the model in one batch (98 s: 1.79 s instead of 2.1–2.8 s).
- App Nap disabled (`NSAppSleepDisabled`): it slowed the stop to 2.08 s; 0.67 s without it.
- `handy -f <wav> --live` measures the wait after a replayed dictation.
- A cancelled dictation, or one replaced by a new recording, stops its background worker: no further piece is sent to the engine, and a cancelled engine call ends it.
- Pieces are reused only with the same model as well; a cancel during the stop's wait for the worker abandons the dictation instead of transcribing it again.

## 2026-11-03

- Created: full pieces of a long dictation are transcribed while the user keeps speaking; the stop reuses them when the audio and options are unchanged.
