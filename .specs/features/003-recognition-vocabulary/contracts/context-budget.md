# Context budget per mechanism — 003-recognition-vocabulary

Step 1 of the plan. Each mechanism has a value, a unit, how it is computed and the source it comes
from. Handy enables a mechanism only when its budget is documented here. A model that declares a
mechanism without a documented budget gets `context_budget_unverified`: it is never sent
unchecked text, and its dictations use approximate correction.

Pinned engine: `transcribe-cpp` / `transcribe-cpp-sys` 0.3.1 (Cargo.lock checksum of
`transcribe-cpp-sys`: `fedfadb42719d92e58a2c340ccfb6ef327ac1b846fc1d4716b65384a330513a6`). Paths
below are relative to the crate sources under `~/.cargo/registry/src/*/transcribe-cpp-sys-0.3.1/`.

## Mechanism table

| Mechanism | State in this build | Budget | Unit |
| --- | --- | --- | --- |
| Whisper initial prompt | **Enabled** | 223 | tokens of the loaded model's tokenizer |
| Generic vocabulary (`RunOptions::vocabulary`: Qwen3-ASR, Fun-ASR-Nano, Granite Speech with the GGUF capability, MOSS) | Disabled: `context_budget_unverified` | Depends on each request's audio (formulas below) | tokens |
| Context prompt (`RunOptions::prompt`) | Disabled: `context_budget_unverified` | Shares the generic budget | tokens |
| ONNX engines (`transcribe-rs` 0.3.8) | No mechanism | — | — |
| Streaming | No mechanism | — | — |

## Whisper initial prompt — 223 tokens

- **Rule.** `max_prev_cap = n_text_ctx / 2 − 1` (`whisper_prev_cap`, `src/arch/whisper/model.cpp`
  line 954, sha256 `b656935f0ecdd20231b3c6ff846e4fb09659a5ef53a4199ab6866fba02d9a137`), with
  `dec_max_target_positions = n_text_ctx` (`src/arch/whisper/bin_load.cpp` line 256). Every Whisper
  model has `n_text_ctx = 448`, so 448 / 2 − 1 = **223**. The OpenAI reference decoder keeps the same
  `n_ctx // 2 − 1` prompt tokens.
- **What the engine tokenizes.** `whisper_run` strips the prompt and prepends one space
  (`" " + strip(initial_prompt)`), then encodes it with the model's tokenizer (`model.cpp` lines
  1621–1683). If the result exceeds the cap, it keeps only the **most recent** 223 tokens
  (`capped_prompt`, lines 1696–1722). The earliest terms would be dropped without notice: that is
  the trimming Handy must avoid.
- **How Handy fits it.** The worker renders the terms with `", "` and counts the tokens of
  `" " + strip(rendered)` with `Model::tokenize`, the same `Tokenizer::encode` the run uses
  (`transcribe_tokenize_impl`, `src/transcribe.cpp` line 2908; Rust `Model::tokenize`,
  `transcribe-cpp-0.3.1/src/model.rs` sha256
  `02979d9c10975b5c853ddc3981228f629c28d3d541b52a9bd778fa0a376d1fca`). It sends the longest whole
  prefix that fits, stopping at the first term that does not fit
  (`recognition_vocabulary::budget::fit_whisper_prefix`). The count includes the separators and
  the leading space, so the engine never trims the prompt and every term reported as sent reaches
  it whole.
- **Task.** The prompt is attached for transcription and translation alike, as before this feature.
- **Mechanism probe.** `Feature::InitialPrompt` **and** acceptance of the Whisper run extension on
  the run slot (`TRANSCRIBE_EXT_KIND_WHISPER_RUN`). A model that declares `InitialPrompt` and rejects
  the extension (Voxtral) gets no mechanism.
- **Measurement.** Pending: no GGUF model is available on the implementation machine (the models
  directory is empty and nothing is downloaded during implementation). The measurement to record
  is `bun scripts/vocabulary-native-proof.ts` (plan Step 6) on a Whisper model: the token count of
  the rendered prompt reported by the worker for a 500-term list, and the request that reaches the
  engine.

## Generic vocabulary and context prompt — not documented in this build

No document publishes a per-mechanism number. The pinned sources compute the room **per request**,
from the audio length and from internal constants:

| Family | Room formula (tokens) | Source (sha256) |
| --- | --- | --- |
| Fun-ASR-Nano | `ceiling − 256 (k_gen_reserve) − n_audio − 48 (k_prompt_overhead_tokens)`, terms rendered as the hotword preamble + `", "`-joined terms + `"]\n"` | `src/arch/funasr_nano/model.cpp` lines 87–89, 160–175 (`98da0a12…8000c`) |
| Qwen3-ASR | `ceiling − 256 (k_gen_reserve) − (15 + T_enc + language prefix)`; terms joined with `" "`, the context prompt trimmed first | `src/arch/qwen3_asr/model.cpp` lines 81, 415–445, 769 (`beb294c9…45617`) |
| Granite Speech | `ceiling − 256 − n_audio − 64`; `" Keywords: "` + `", "`-joined terms; plain transcription only, translation only on `granite-speech-4.1-2b` | `src/arch/granite/model.cpp` lines 87–89, 550–552, 620–640 (`ee3ed628…2f4b`) |
| MOSS | `ceiling − 256 (k_max_new) − base prompt length` | `src/arch/moss/model.cpp` lines 264–270 (`4f13e785…f2ba1b`) |

All four call `fit_terms_and_context` (`src/transcribe-prompting.cpp` lines 97–166, sha256
`6fc2de9caa055344505867115c63dad59296ca9046f899a3d007af7e5a8eb59f`), which drops the tail terms that
do not fit and only logs a warning. The Rust API of 0.3.1 exposes neither `n_audio`, `T_enc`, the
context ceiling nor the number of terms kept (`Transcript` has no such field).

Consequence: Handy cannot compute these budgets before a request, nor learn afterwards which terms
reached the engine. Enabling them would let the engine drop terms that the status reports as sent,
which AC-003 forbids. They stay disabled (`MechanismSupport::Unverified`), as Step 1 requires:

- the status of such a model is the technical error `context_budget_unverified` (CLI:
  `internal_error`, exit 1, `details.reason = "context_budget_unverified"`), never one of the five
  statuses;
- its dictations run without context, and approximate correction applies.

To enable a family later: obtain from the engine either the accepted-term count of a run or the
room before a run (an API or a pinned, version-checked formula over `Capabilities`), measure it on
the catalog models with their tokenizer, and add the row here before changing
`capability::resolve_mechanism`.

Catalog families affected in this build: `qwen3_asr` (2 models), `funasr_nano` (2),
`granite_speech` and `granite_speech_nar` (4, capability depending on the GGUF), `moss` (1). Whether
each declares `Feature::Vocabulary` or `Feature::ContextPrompt` must be read from the loaded model.
This is pending native evidence.
