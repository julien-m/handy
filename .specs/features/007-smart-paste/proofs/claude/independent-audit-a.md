# Independent audit A — 007 Smart Paste core (steps 1–3, step 5 states/timers)

- **Auditor:** Claude (independent, read-only; did not write this code). No Codex, no `audit` skill, no commit.
- **Date:** 2026-10-13
- **Worktree / branch:** `/Users/julienm/projects/handy-wt/007-smart-paste`, `feature/007-smart-paste`, HEAD `a65181b`; diff base `git merge-base HEAD dev` = `967eb38`.
- **Scope:** `82e0330` (step 1), `a2f926a` (step 2), `3c0168c` (step 3), `a65181b` (step 1 test fix); `4c5b365` (step 5) only for its use of states and timers. Steps 4–6 and settings are covered by audit B and were read only at their interfaces (`macos_pasteboard.rs`, `main_thread.rs`, `key_events.rs`, `keyboard.rs`). Items marked "Waiting for 006/008" are deferred.
- **Read first:** `AGENTS.md`, `.specs/constitution.md`, `.conventions/index.md` (+ `ai-ressources/code-conventions/general.md`, `rust.md`), `spec.md`, `plan.md` (steps 1–3, 5), `progress.md`, `implementation.md`.

## Verdict: **PASS** (0 blocker, 0 major, 6 minor, 5 nit)

The restoration rule is implemented as specified: the only paths that call `restore_snapshot` are `SettlementDue` (reachable only from `ReadPhase::ReadObserved`) and `Exit { is_restoration_due: true }`. "Not read" drops the snapshot in `PendingClipboard::settle`, so it can never come back. Every restore and materialization is guarded by the publication `changeCount` inside the same main-thread closure. Direct insertion is verified only on an exact UTF-16 read-back. It never falls back after a change it did not expect, and two guards check for late writes: one before publication, one before the keystroke. Each `insert` call returns exactly one of the 8 outcomes, and late reads go to a separate idempotent diagnostic. I found no `unwrap`/`expect`/`panic!` in production code. No transcript, field value or clipboard content reaches a log, and the CF/AX FFI follows the Create/Copy rule with null checks.

The minor findings below do not let old content be pasted and do not cause a duplicate insertion. Each one narrows a guarantee or misreports an outcome. Some residual risks are inherent to the spec's unauthenticated read receipts and to AppKit caching. They are not defects of these commits, but they need a decision before release (section "Residual risks").

## Findings

### MINOR-1 — An in-window read can be declared "not read" by the actor (timer/read ordering not enforced)

- **Where:** `src-tauri/src/smart_paste/actor.rs:130-141` (`ActorQueue::next_read` returns `None` on the first `recv_timeout` expiry) → `coordinator_chord.rs:126` (`expire_no_read`) → `lifecycle.rs` `poll_deadline`, whose contract says "callers record every read stamped at or before `now` first".
- **Scenario:** The provider stamps a read at key-down + 799 ms on the main thread. The thread is preempted before `sink` sends it, and the actor's `recv_timeout` expires at 800 ms. The actor declares `not_read` with the read still queued. The outcome is `not_read`: the snapshot is discarded and the previous clipboard is never restored, recovery is offered and Enter is suppressed. Then a "may have been pasted after all" diagnostic follows. The window is small in production (stamp then send). It is reproduced deterministically with a delivery delay: `repro-007a/crate/tests/repro.rs::r1_*` → `R1 outcome=NotRead late_read_diagnostics=1`.
- **Fix:** On timeout, drain the channel without blocking (`try_recv`) and process every `Read` stamped `<= deadline` before calling `expire_no_read`. Do the same in the idle loop before `handle_deadline`. Add an actor test with a backdated read.

### MINOR-2 — One read-back timeout right after the write yields "uncertain" even when the field becomes exact

- **Where:** `direct.rs:248` (`Err(_) => return DirectAttempt::Uncertain(ReadbackUnavailable)` inside `read_back`, `direct.rs:240`).
- **Scenario:** A Chromium or Electron field, or any app busy applying the `AXSelectedText` write, does not answer the first `AXValue` read within its 5–20 ms messaging timeout (`kAXErrorCannotComplete`). The final native AX run already shows a cold first contact exceeding the 20 ms budget. The next poll would show the exact expected value, but the attempt is already `uncertain`: recovery bubble, no automatic Enter, although the text is correctly inserted. Repro: `r2_*` → `R2 result=Uncertain(ReadbackUnavailable)` for a field that returns `Timeout` once and then the exact value.
- **Fix:** Retrying a *read* is not the "blind retry" the plan forbids, which concerns the write or the paste. Keep polling within the window after a read error, and return `Uncertain(ReadbackUnavailable)` only if no read succeeded by `window_end`. An exact value is still `Verified`, an unchanged value at the end still means fallback with the guard, and any other value is `Uncertain`. The fake in `tests/smart_paste/fakes.rs` does not model timeouts (`call_cost` never fails a call), which is why the test suite cannot see this.

### MINOR-3 — Copy mode loses the AC-012 late-read diagnostic

- **Where:** `coordinator_chord.rs:196-208` (`declare_not_read`: in copy mode `settle` materializes the transcript and `pending` becomes `None`), `pending.rs` `settle`.
- **Scenario:** With clipboard handling "copy to clipboard", the target reads at 900 ms. The transcript was materialized as eager plain text at 800 ms, so the read is served by the pasteboard server without a provider callback and no transaction remains to correlate it. AC-012 requires the bubble to say "the text may have been pasted after all", which can never happen in this mode. Repro: `r3_*` → `DontModify: pending=Some(..) diagnostics=1` / `CopyToClipboard: pending=None diagnostics=0`.
- **Fix:** Either keep the lazy item serving until the next transaction or exit in copy mode too, with the plain-text materialization deferred to settlement or supersession, or record this limitation explicitly in spec/implementation and in the 010 contract. Add a test either way.

### MINOR-4 — Handy's own transcript loses its privacy markers when it is "restored" or materialized

- **Where:** `coordinator.rs:302-308` (`SnapshotSource::PendingTranscript` → `PasteboardSnapshot::from_plain_text`, `snapshot.rs:123`), `macos_pasteboard.rs:219` (`write_plain`, used by `materialize_transcript` on exit and keystroke cancellation).
- **Scenario:** Dictation 1 ends `not_read`, so its lazy item (Transient/Concealed/AutoGenerated) stays on the clipboard. Dictation 2 snapshots it as plain text only and restores it after its read. Transcript 1 then lands on the clipboard without markers and is recorded by clipboard-history managers. AC-009 asks to restore "the saved items and types exactly", and the item carried 3 marker types. Exit and cancellation materializations drop the markers in the same way: the plan's marking table only defines "no markers" for copy mode. Repro: `r4_*` → `restored types = [["public.utf8-plain-text"]]`.
- **Fix:** Build the `PendingTranscript` snapshot with the same eager marker types (empty data). Outside copy mode, materialize with the markers. Assert the restored types in `after_not_read_the_pre_first_snapshot_never_returns`.

### MINOR-5 — Destination lost after an AX write is reported `insertion_failed`, not `uncertain`

- **Where:** `coordinator_chord.rs:67-85` (`cancel_reason` returns `InsertionFailed` as soon as `is_current` is false, before consulting the `FallbackGuard`).
- **Scenario:** The AX write was issued and read back as unchanged. The guard is still pending and the user switches app during `paste_delay`, so the chord is cancelled. The write may still apply late, so the text may be in the field. The plan says: "loss of destination gives insertion failed before effects, uncertain after a possibly applied AX write". The outcome claims a clean failure, and the 010 bubble may then invite the user to paste text that is already there.
- **Fix:** When `guard.is_some()`, map the loss of destination to `(DirectAccessibility, Uncertain)`. Add a chain test that combines destination loss with an AX write that was issued.

### MINOR-6 — `SmartPasteActor::shutdown` cannot settle when called from the main thread (integration hazard for step 7)

- **Where:** `actor.rs:79` (`shutdown` blocks on `recv_timeout`), `macos_pasteboard.rs:50,159-169` (each settlement write is a main-queue dispatch with a 1 s timeout), `main_thread.rs:75`.
- **Scenario:** Tauri exit hooks run on the main thread. If step 7 calls `actor.shutdown(t)` there, the actor's `restore_snapshot` or `materialize_transcript` waits for the blocked main queue and is abandoned after 1 s (`MainThreadUnavailable` becomes `LeaveNewerContent`). The lazy item then dies with the process and the clipboard is left neither restored nor materialized. This is the "core prevents clean integration" case only if the hazard stays undocumented: the API needs a stated contract.
- **Fix:** Document on `shutdown` that it must run off the main thread while the main run loop keeps pumping (for example `prevent_exit`, then a worker calls `shutdown`, then `exit`). Add a `debug_assert!(!is_main_thread())`, or return an error when called on main.

### NIT-1 — `Next::Disconnected` is unreachable in production
`actor.rs:48` clones a sender into the read sink that the ports keep, so the channel never disconnects while the actor runs (`actor.rs:168`). Dropping the handle without `shutdown` leaks the thread and never settles. Implement `Drop` that sends `Shutdown`, or fix the comment.

### NIT-2 — A transaction id is reported for a transaction that never started
`coordinator.rs:218` takes `TransactionId` before the snapshot and publication. Failures before publication return `transaction_id: Some(..)` although `PasteOutcome` documents "when one was started". 010 could wait for diagnostics of an item that never existed.

### NIT-3 — The preparation budget can overrun by up to one per-call timeout
`direct.rs:196` computes one timeout for `is_secure`, and `macos_ax.rs:151-157` uses it for two calls (`AXRole` and `AXSubrole`). Preparation can therefore reach about 120 ms. It still skips before writing, so AC-018 is respected, but the 100 ms split is not exact.

### NIT-4 — The effective read-back window ends about 5 ms early
The last poll starts at `window_end - MIN_READ_TIMEOUT` (`direct.rs:250-254`), so a write applied at 146–150 ms is caught by the guard rather than by read-back. The test `late_application_boundary_is_the_last_read_of_the_window` pins this behavior (145 verified, 146 unchanged). The behavior is safe, but the spec's "150 ms" is not met to the letter.

### NIT-5 — The AX evidence receipt is stale
`evidence/native-ax-adapter.json` records the `macos_ax.rs` hash from `a2f926a` (`21f93c…`). HEAD is `3be71c…` after the step-6 CF refactor. The final raw rerun (`proofs/claude/final-smart_paste_native_ax.out`) passes at HEAD and shows a cold first contact `Skipped(FieldUnavailable)`, but the JSON receipt was not regenerated.

## Residual risks (not counted, release-gated by AC-016 / SC-007 / SC-008; need Julien's decision)

1. **AppKit caching makes AC-009's "quiet after the *last* read" unobservable.** In `evidence/native-spike.json`, `repeated_postkey_callback_count = 1` and `prekey_then_postkey_callback_count = 1`. Quiet is therefore measured from the first observed read: a target that reads again more than 300 ms after that first callback receives the restored previous clipboard. The fake tests `probe_then_read_restores_after_the_second_read` (lifecycle and coordinator) prove the reducer, not AC-009 natively, because `FakePasteboard`/`ScriptedReads` report every read. A clipboard manager that ignores the markers and reads before the keystroke makes every dictation `not_read`, so the clipboard is never restored and no Enter is sent. This is documented as a limitation, but the spec Gherkin "restoration happens 300 ms after the second read" cannot be met with this mechanism.
2. **Unauthenticated receipts.** A third-party read *after* the key-down (a manager polling changeCount and ignoring markers) becomes the first qualifying read. A slow target whose first read comes more than 300 ms later then receives the previous clipboard, which is exactly the outcome decision (1) forbids. No attribution is possible with the AppKit API.
3. **AX late apply after the pre-key guard, or a stale `AXValue` that never reflects the write**, produces a duplicate insertion. This is acknowledged in the plan and must be measured per app (SC-007).
4. **Overlapping dictations:** a delayed Cmd+V from dictation N can read dictation N+1's item (OS race, acknowledged in the plan).

## Interfaces with step 5 (states and timers) — checked, no defect beyond MINOR-1/3/5/6

- No restoration without a qualifying read. A read after a failed chord can authorize a restore, as the plan allows. When a newer dictation starts before restoration, no intermediate restore happens: the pending snapshot is carried only if still owned and not discarded, otherwise the current content is captured. A `not_read` snapshot is never resurrected.
- Reads of superseded items are ignored (filtered by transaction). Reads stamped before key-down never count, reads during the chord hold count, and the 8 s cap is anchored on the first read.
- Exit restores only when the quiet period or the cap is due, and otherwise materializes under the guard.
- Timings come from one source (`timing.rs`), selected per transaction by bundle id: Jump Desktop uses 1.5 s quiet and 3 s no-read, everything else 300 ms and 800 ms.

## Tests — do they prove the ACs?

- **Strong:** the production reducer and coordinator are driven through injected fakes with a virtual clock. Boundaries are covered: 299/300/301, 799/800/801, Jump 1599/1600 and 2999/3000, 8 s cap, read stamped exactly at the deadline. Also covered: changeCount guard (text and image), copy mode, chord failure, permission preflight (zero effects), unsavable snapshot refusal, succession, exit, late write before publication and before the keystroke, partial change, secure field, PID mismatch, oversize field, surrogate-splitting range, slow preparation, per-call timeout ≤ 20 ms and total ≤ 250 ms on a virtual clock. The log-privacy sentinel test covers every coordinator path.
- **Gaps:**
  - The fakes report every read, which hides residual risk 1.
  - `ScriptedField` never times out, which hides MINOR-2.
  - No actor test of read/deadline ordering (MINOR-1).
  - No test of the late-read diagnostic in copy mode (MINOR-3) or of destination loss after an AX write (MINOR-5).
  - No test of a normalizing field (whitespace, newline, NFC/NFD), a rich field with U+FFFC, or a caret in the middle with an astral transcript through the coordinator. The pure tests cover the arithmetic, and the expected outcome there is `uncertain`, never a duplicate.
  - The actor tests depend on real-time sleeps (600 ms); flakiness risk is low.

## Commands run (all with `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`)

| Command | Result |
|---|---|
| `cargo test --test smart_paste` | 86 passed, 0 failed |
| `cargo test --lib smart_paste` | 33 passed, 0 failed |
| `cargo test` (full) | exit 0 — lib 319 passed / 16 ignored; `smart_paste` 86 passed; native targets ignored or harness-free (not executed) |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets` (extra) | exit 0; 5 pre-existing warnings (`lib.rs`, `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs`, `managers/transcription.rs`, `portable.rs`), none in `smart_paste` |
| `livespec conventions verify --feature repo --json` | **PASS**, 0 blockers, 423 violations all baselined, `new_count 0`. Run on a `git archive HEAD` export with the worktree's `node_modules` symlinked, so that the receipt it writes did not modify the repo. Without `node_modules` it fails only on a missing `eslint` binary. `smart_paste` has only `max_function_lines` warnings. |
| Audit reproductions `repro-007a/crate` (`cargo test --test repro`) | 4/4 reproduce MINOR-1..4. The audited sources are included read-only via `#[path]`; nothing copied into the repo. |

Native opt-in targets (`smart_paste_native*`) were **not** executed by this audit. The implementer's raw outputs at HEAD were inspected instead: `final-smart_paste_native_ax.out`, `final-smart_paste_native.out`, `final-smart_paste_native_pasteboard.out`. The general clipboard, focus and keyboard were not touched.

Repo hygiene: this audit's only change to the worktree is this report. Its conventions receipts (`20261013T161104Z`, `20261013T161137Z`) stayed in the scratch export. The untracked `.specs/conventions/runs/20261013T160900Z/` in the worktree, a `repo` PASS created at 16:09Z, was not produced by this audit; the parallel auditor is the likely source.

Scratch artifacts: `/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/repro-007a/` (`repro-output.txt`, `run-full-cargo-test.*`, `run-fmt.out`, `clippy.*`, `conventions.json`, `crate/tests/repro.rs`).
