---
feature: dictation-field-return
title: Dictation Field Return Implementation
status: In Progress
created: 2026-10-15
updated: 2026-10-15
---

# Implementation — 011-dictation-field-return

Status: In Progress. Steps 1 to 5 of the plan are implemented on `feature/011-dictation-field-return` (from `dev` `4e3e5143`, 009 and 010 merged), behind 006's closed `destination-return` rollout gate; Step 1 (native spike) and Step 6 (acceptance with Julien) have their automated part only, and every native macOS proof is pending with Julien (`.specs/native-proofs-checklist.md`, section 011). With the gate closed, which is the default build, Handy is `dev`: no session, so no focused-element read, no attempt and no verification. No AC or SC is certified complete. Read [the functional source](spec.md), [the plan](plan.md) and the step checkpoints in [progress](progress.md).

Commits: `ffc9ebdb` (Steps 1–5: model, capture, attempt, guard verification, 010 hand-off, macOS and Windows adapters, tests, native harness), `9809ff61` (spent-budget fault test), `256cf4f9` (documentation), then the fix commit of delivery review round 1 (see "Delivery review round 1 — corrections"). Final verification: `proofs/impl-20261015-fix1/summary.txt` (earlier: `proofs/impl-20261015/summary.txt`, whose Windows harness proof predates `ffc9ebdb` and is superseded).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Remember the focused input and its range](spec.md#fr-001) | `src-tauri/src/destination/{field.rs,field_capture.rs,field_lifetime.rs,app.rs,macos_field.rs}`, `src-tauri/src/recovery/classifier.rs` | `@spec FR-001: Remember the focused input and its range`; `@spec AC-001: Editable input of the same process only`; `@spec AC-001: 010's editable rule decides what 011 remembers` | ⚠️ Partial: bounded start read off the start path, 010's rule, password, process, permission, errors, deadline, range read or not, privacy; native read of Comet, Notes and ChatGPT pending | 2026-10-15 |
| [FR-002: Note, focus, raise, restore, verify, insert](spec.md#fr-002) | `src-tauri/src/destination/{field_attempt.rs,field_check.rs,guard.rs,insertion.rs}` | `@spec FR-002: Note then focus then raise then restore`; `@spec AC-002: One attempt within 50 ms without retry`; `@spec AC-002: Verify input and range at the guard within 10 ms`; `@spec AC-005: A verified input counts as editable for 010` | ⚠️ Partial: attempt after activation, verification inside the guard before 010, paste only after a verified range; native matrix pending | 2026-10-15 |
| [FR-003: Pinned instance, kept for the same instance](spec.md#fr-003) | `src-tauri/src/destination/{field_capture.rs,field_lifetime.rs,menu.rs,pin.rs,manager.rs}` | `@spec FR-003: Read the pinned instance's input at the start`; `@spec AC-003: Kept only for the same running instance` | ⚠️ Partial: pinned instance read whatever is in front, dropped by a badge choice, the cross, a pin of another application or an unpin to another instance; native pin run pending | 2026-10-15 |
| [FR-004: Unchanged path or bubble, never elsewhere](spec.md#fr-004) | `src-tauri/src/destination/{field_check.rs,guard.rs,insertion.rs,outcome.rs,context.rs}`, `src-tauri/src/recovery/{decision.rs,service.rs}` | `@spec FR-004: Paste only into the verified input or unchanged`; `@spec FR-004: Verify the remembered input before any paste`; `@spec AC-004: Branches (a) and (b) decided at the guard`; `@spec AC-004: Field return failed and nothing sent` | ⚠️ Partial: guard table, unchanged attempt on 006/010, every other failure `field_return_failed` with nothing pasted, 010's bubble or one `paste-error`, no Enter; native fallback cases pending | 2026-10-15 |
| [FR-005: Gate, platforms, no setting or UI](spec.md#fr-005) | `src-tauri/src/destination/{mod.rs,field_capture.rs,windows_field.rs,windows_uia.rs,bounded_worker.rs,uia_mapping.rs}`, `src-tauri/Cargo.toml` | `@spec FR-005: macOS and Windows only without a setting`; `@spec AC-006: No read without a destination`; `@spec AC-007: Windows text input rules`; `@spec AC-007: Bounded UI Automation calls` | ⚠️ Partial: zero spawn with the gate closed, Linux unsupported port, Windows adapter compiled and clippy-clean for MSVC with its mapping and bounded calls tested everywhere; no setting, CLI key, event or UI added; Windows native run pending release evidence | 2026-10-15 |

## Acceptance Criteria Mapping

Derived from the `@spec AC-xxx` anchors of the Rust tests: `proofs/impl-20261015/acceptance-mapping.json` (`livespec test --feature 011-dictation-field-return --derive-mapping`, exit 0, 13 bindings, AC-001 to AC-008, no issue).

| AC | Test File | Status |
| --- | --- | --- |
| [AC-001](spec.md#ac-001) | `src-tauri/tests/destination_field_capture.rs`, `src-tauri/tests/destination_field_native.rs` (ignored, native) | ⚠️ Partial: editable remembered with and without range; password by role and subrole, other process, not editable, undetermined, permission (zero query), errors and deadline remember nothing; every query timeout is the remaining budget, never below 5 ms; native pending |
| [AC-002](spec.md#ac-002) | `src-tauri/tests/destination_field_return.rs`, `src-tauri/tests/destination_field_native.rs` (ignored, native) | ⚠️ Partial: note, focus, raise when needed, range restored, verified at the guard, pasted at the caret; native pending |
| [AC-003](spec.md#ac-003) | `src-tauri/tests/destination_field_capture.rs`, `src-tauri/tests/destination_field_lifetime.rs` | ⚠️ Partial: pinned instance read in the background, pin not running reads nothing, drop and keep rules; native pending |
| [AC-004](spec.md#ac-004) | `src-tauri/tests/destination_field_return.rs`, `src-tauri/tests/destination_field_recovery.rs`, `src-tauri/src/destination/field_check_tests.rs` | ⚠️ Partial: branch (a) inserts as without 011; branch (b) for nine injected faults, the range missing, changes during the delay and a spent budget pastes nothing; bubble and `paste-error`; native pending |
| [AC-005](spec.md#ac-005) | `src-tauri/tests/destination_field_recovery.rs` | ⚠️ Partial: verified input inserts with zero 010 query, Enter follows 009 (none with another app in front, none after a recovery); native pending |
| [AC-006](spec.md#ac-006) | `src-tauri/tests/destination_field_lifetime.rs`, `src-tauri/tests/destination_field_capture.rs` | ⚠️ Partial: cancellation, new dictation, end, method none release the reference unused; return off, Handy in front and closed gate read nothing; native pending |
| [AC-007](spec.md#ac-007) | `src-tauri/src/destination/uia_mapping.rs`, `src-tauri/src/destination/bounded_worker.rs` (unit tests), `proofs/impl-20261015/windows-check/` | ⚠️ Partial: Windows mapping and bounded calls tested on every platform, adapter compile-checked; Windows native run pending release evidence |
| [AC-008](spec.md#ac-008) | `src-tauri/tests/destination_field_privacy.rs`, `src-tauri/tests/destination_field_native.rs` (source checks), `src-tauri/src/destination/field_check_tests.rs` | ⚠️ Partial: sentinel range absent from every log level and the serialized context; the port cannot read content; native sentinel check pending |

| SC | Evidence | Status |
| --- | --- | --- |
| SC-001 | Native matrix (checklist section 011) | ❌ Pending with Julien |
| SC-002 | `destination_field_capture.rs`, `destination_field_return.rs`, `destination_field_recovery.rs` | ⚠️ Automated cases pass; native fallback cases pending |
| SC-003 | `destination_field_capture.rs::with_the_gate_closed_no_start_read_is_ever_spawned` and `::with_the_gate_closed_a_whole_dictation_reads_nothing_and_pastes_as_without_011` (closed builds: a whole dictation with the scripted port installed makes zero call, `FieldCheck::Absent`, same report and target as without 011's port); the closed gate's absence of any session (`dev`'s paste) is 006's closed-gate rollout tests; final cargo tests in the four combinations | ⚠️ Automated pass; native closed-gate run pending |
| SC-004 | `destination_field_privacy.rs`, harness `--log --sentinel` | ⚠️ Automated pass; native sentinel pending |
| SC-005 | `destination_field_capture.rs::the_start_path_returns_while_the_port_never_answers_and_a_late_answer_is_dropped` (open builds) | ⚠️ Automated pass |

## Rollout gate

011 rides 006's `destination-return` Cargo feature and adds none. Closed (default build): `capture_at_start` and `ticket_at_stop` return `None`, `spawn_start_read` refuses before any read, so no focused element is ever read and the insertion is `dev`'s; every 011 module stays compiled and tested in the four feature combinations. Open: the field return applies on macOS (Accessibility) and Windows (UI Automation) to every dictation whose 006/009 route names an application; Linux has no destination and an unsupported port.

## Differences from the plan

The code is the reference where it differs from the plan; none changes a requirement.

1. The platform port is held by `DestinationManager` (`with_field_access`, `field_access`) instead of a new `Orchestrator.field`, so the eight existing orchestrator constructions stay unchanged; the orchestrator reads it from its manager.
2. The plan's `FieldPort { capture, attempt, verify }` is split in two layers: platform primitives (`FieldAccess`, `FieldNode`: focused element and window, facts, range read and compare, focus and range writes, raise, identity) and shared bounded sequences (`field_capture::capture`, `field_attempt::attempt`, `field_check::verify`) with the pure table `decide_field_check`. Budgets, per-call timeouts and the 5 ms floor are thus implemented and tested once for both platforms.
3. `freeze_with_field` returns the field beside 006's `FrozenDestination`, which keeps its fields; `freeze` releases the field.
4. The guard turns a `Diverted` check into `Recover(FieldReturnFailed)` itself, even with 006's `NoRecoveryConsumer`; 010's port answers the same without any query, and answers `Insert` for `Verified`.
5. Tests are integration tests over one scripted accessibility world (`src-tauri/tests/destination_field/`) and in-crate unit tests (`field_check_tests.rs`, `bounded_worker`, `uia_mapping`, `macos_field`) instead of `field_tests.rs`, `manager_tests.rs` and `fake_field.rs`. The fake port, like the real one, has no accessor for a value, selected text or title (stronger than a panicking accessor).
6. The capture budget runs from the read request (the session commit), and `attach_field` refuses an answer later than 50 ms (plan review warning).
7. Windows: element identity is the runtime id and process, compared locally, instead of `CompareElements`; the client lives in `windows_uia.rs`; the bounded executor (`bounded_worker.rs`) and the text-input mapping (`uia_mapping.rs`, expressed in 010's rule vocabulary) are cross-platform and tested on every CI job.
8. `DestinationContext.field` is `#[serde(skip)]`: the decision never reaches a serialized value.
9. `RememberedField::serves` keeps the field when the route names the instance read or the process the route named (an unpin back to the same instance keeps it).

## Review warnings (r3 synthesis)

| Source | Warning | Answer |
| --- | --- | --- |
| Spec, AC-002 | The start range is restored even if the user typed in the same input during the recording | Implemented as specified; native item "Frappe dans le même champ" and a decision line for Julien (checklist §2 and §3) |
| Spec, AC-004 | Whether "no focused element" is a noted value or an unreadable read | Conservative: no focused element when noting gives `NotNoted`, at the guard `FocusUnreadable`: branch (b), never a paste; native reload item records it; decision line for Julien |
| Spec, info | Focus before raise; `AXRaise` may not make the window key | A failed verification gives the bubble; native item "Deuxième fenêtre" records focus after `AXRaise` |
| Spec, info | SC-004 does not check history | Nothing of the field reaches history: 011 adds no history write |
| Plan | 70 ms bound measured only in the Step 1 spike | Native item "Temps et thread principal" measures the real `check()` after Step 4 with the classic method |
| Plan | `attach_field` without deadline | Fixed: refused after 50 ms from the request (test with a port that never answers, then answers late) |
| Plan | Windows pinned application in the background remembers nothing | Implemented as the plan says; listed in checklist §3 for Julien's decision |
| Plan | CI test matrix | `.github/workflows/test.yml` already runs the four combinations (010 Step 7); the 011 tests run there |
| Plan | SC-002 focus refused and attempt budget spent | Explicit tests: focus refused (branch a), raise refused, spent budget (`an_attempt_out_of_budget_stops_without_sending_and_pastes_nothing`); every diverted case asserts a failed report (transcript retained by 006) |
| Plan | SC-004 sentinel in native checks | Harness `--log <file> --sentinel <text>` |
| Plan, info | `field_return_failed` reaches the `insertion-outcome` event | Internal wire value: no frontend source parses the outcome reason any more (010 removed the 006 notice; `src/overlay/use-overlay-notices.ts`), and 010's bubble takes the reason |

## Deltas for other features' owners

Not edited by 011: to be realigned through their own LiveSpec flow (`/spec-refine 006-destination-application plan`, `/spec-refine 010-recovery-bubble plan`).

| Owner | Delta in code |
| --- | --- |
| 006 (`.specs/features/006-destination-application/plan.md`, `contracts/destination-contract.md`) | `DestinationContext.field: FieldCheck` (not serialized); `RecoveryReason::FieldReturnFailed`; `DestinationFailure::FieldReturnFailed` (`field_return_failed`); `DispatchGuard::with_field` and the verification in the first pass before the 010 verdict; the attempt after activation in `Orchestrator::deliver`; `ManagerState.field`, `attach_field`, `field_target`, `freeze_with_field`, `with_field_access` and the drop rules in `commit_start`, `cancel`, `finish`, `fail_insertion`, badge choices and 009's pin and unpin |
| 010 (`.specs/features/010-recovery-bubble/plan.md`, spec reason use) | `before_insertion` answers `Insert` for `Verified` and `Recover(FieldReturnFailed)` for `Diverted` without query; `field_return_failed` maps to "Insertion failed" with "nothing was sent"; the editable rule is shared as `classify_focused` with a `Deadline` that takes a floor (0 for 010); the AX helpers of `macos_ax.rs` are crate-visible; the composed main-thread bound is 70 ms when the check is `Unchanged` |
| 009, 007 | None: pin resolution, Enter authority and 007's direct insertion are used unchanged |

## Files Created/Modified

- Created: `src-tauri/src/destination/{field.rs,field_capture.rs,field_attempt.rs,field_check.rs,field_check_tests.rs,field_lifetime.rs,macos_field.rs,windows_field.rs,windows_uia.rs,bounded_worker.rs,uia_mapping.rs}` — model, port, sequences, decision table, lifetime, adapters.
- Modified: `src-tauri/src/destination/{app.rs,context.rs,guard.rs,insertion.rs,manager.rs,menu.rs,mod.rs,outcome.rs,pin.rs}` — 006 integration; `src-tauri/src/recovery/{classifier.rs,decision.rs,service.rs,platform/macos_ax.rs}` — 010 integration; `src-tauri/Cargo.toml` — Windows features.
- Tests: `src-tauri/tests/destination_field_{capture,return,lifetime,recovery,privacy,native}.rs`, `src-tauri/tests/destination_field/{fake.rs,rig.rs}`; context literals in `src-tauri/src/automatic_enter/transaction_fixture.rs`, `src-tauri/src/smart_paste/destination_bridge_tests.rs`, `src-tauri/tests/{enter_platform.rs,recovery_privacy.rs}`; 010 table in `src-tauri/src/recovery/decision_tests.rs`.
- Native: `scripts/field-return-native-proof.ts`, `.specs/native-proofs-checklist.md` (section 011, decisions, gate, release).

## Release Evidence

| Step | Name | Proof or receipt |
| --- | --- | --- |
| 5 | Windows adapter | `proofs/impl-20261015-fix1/final/39.*` (harness `cargo clippy --target x86_64-pc-windows-msvc -- -D warnings`, exit 0) and `final/40.*` (harness `cargo check --target x86_64-pc-windows-msvc`, exit 0), both from the fix commit's tree: they compile the real `windows_field.rs`, `windows_uia.rs`, `bounded_worker.rs`, `uia_mapping.rs` and `field.rs` with stubs of the 006/010 types they import. The whole crate cannot be cross-compiled locally: `cargo check --target x86_64-pc-windows-msvc` in `src-tauri` stops at the `ring` build script (`'assert.h' file not found`, no Windows SDK headers; `proofs/impl-20261015-fix1/windows-crate-check.log`, exit 101). The CI `windows-check` job and the Windows build workflows compile the whole crate. The round-0 proof `proofs/impl-20261015/windows-check-clippy.*` predates `ffc9ebdb` and is superseded. Native Windows run pending (checklist §5) |

## Delivery review round 1 — corrections

Review BLOCKED and audit BLOCKED (`.reviews/delivery.json`, round 1) for one blocking point: the Windows build did not compile. Corrected on the coordinator's instruction; verification in `proofs/impl-20261015-fix1/summary.txt`.

| Finding | Correction | Tests |
| --- | --- | --- |
| Blocking: `TextPatternRangeEndpoint_Start`/`_End` not imported (`windows_uia.rs`); the Windows proof predated the code | Imported; Windows harness clippy and check rerun from the fix tree (exit 0); stale proof marked superseded; whole-crate cross-check impossible locally, stated in Release Evidence | `final/39`, `final/40` |
| Closed gate on Windows not identical to `dev`: the UI Automation thread and COM client started at launch | The worker is created on the first field call only (`OnceLock`); with the gate closed no field call happens; module documentation aligned | Windows harness compile; closed-gate test below |
| COM leak after an expired call | `BoundedWorker::call_or_undo`: whoever takes the answer slot first decides; a late job whose caller gave up hands its result to an undo run on the thread, which removes the element or range it kept (`forget_seen`, `forget_range`) | `bounded_worker::tests::an_object_kept_by_an_expired_call_is_released_when_the_late_job_ends` |
| A stale `freeze` took the next dictation's field | `freeze_with_field` acts only after a successful freeze and only on the field of that dictation and generation | `destination_field_lifetime::a_stale_freeze_never_takes_the_input_of_the_next_dictation` |
| `std::thread::spawn` on the start path | `thread::Builder` named `handy-field-read`; on failure nothing is remembered, a warning is logged, no panic | build, clippy |
| Widened visibility | `short_string` private again; `bounded_worker` and `uia_mapping` private modules (dead-code allowed off Windows); `capture_since` private; `attach_field` and `decide_field_check` crate-visible | clippy in the four combinations |
| SC-003 needed a whole dictation | Closed-gate test over a whole dictation with the scripted port installed (see SC-003 row) | `destination_field_capture::with_the_gate_closed_a_whole_dictation_reads_nothing_and_pastes_as_without_011` |
| Info: a cancellation between activation and attempt moved the focus | `field_attempt::begin` reads the cancellation first and drops the field | `destination_field_return::a_cancellation_after_the_activation_never_moves_the_focus` |
| Info: `expected()` over 30 lines | Split into `expected` and `row_of` | `decision_tests` |
| Same-field typing, Windows pinned background, Chromium late focus | Behavior unchanged; the two review proposals (character count with an AC-008 amendment, a Windows exception of AC-003) and the Chromium case are in `.specs/native-proofs-checklist.md` (§2 item "Onglet changé", §3) | documentation |
