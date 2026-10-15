---
feature: pin-mode-and-enter-rule
title: PIN Mode and Enter Rule Implementation
status: In Progress
created: 2026-10-15
updated: 2026-10-15
---

# Implementation — 009-pin-mode-and-enter-rule

Status: In Progress. Steps 2 to 6 of the plan are implemented on `feature/009-pin-mode` (from `dev` `09cc1ea2`), behind 006's closed `destination-return` rollout gate; Step 1 and Step 7 have their automated part only, and every native macOS proof is pending with Julien (`.specs/native-proofs-checklist.md`, section 009). With the gate closed, which is the default build, Handy is `dev`: the `auto_submit` switch, key and emitters, no `destination` CLI domain, `automatic_enter` unknown, no pin entry. No AC or SC is certified complete. Read [the functional source](spec.md), [the plan](plan.md) and the step checkpoints in [progress](progress.md).

Commits: `dad3819d` (existing clippy test lints), `a92d931d` (backend, Steps 2–5), `edd02eb6` (UI, Step 6), `40143f4b` (native harness and documentation, Steps 1 and 3), `d8ad2a53` (keyboard check in Chromium), `69afc00b` (repository conventions). Final verification: `proofs/impl-20261015/summary.txt` (40 commands, all exit 0).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Session pin wins over the capture](spec.md#fr-001) | `src-tauri/src/destination/pin.rs`, `src-tauri/src/destination/manager.rs`, `src-tauri/src/destination/menu.rs`, `src-tauri/src/destination/session.rs`, `src-tauri/src/destination/snapshot.rs`, `src-tauri/src/commands/destination.rs`, `src-tauri/src/settings_service/domains/destination.rs`, `src-tauri/src/cli.rs`, `src/overlay/pin-state.ts`, `src/overlay/use-destination.ts`, `src/overlay/destination-menu.tsx`, `src/overlay/destination-badge.tsx`, `src/overlay/destination-model.ts` | `@spec FR-001: Session pin, current and next dictations`; `@spec FR-001: The pin wins over the capture, kept for unpin`; `@spec AC-001: …`, `@spec AC-002: …`, `@spec AC-006: …` | ⚠️ Partial: implemented and tested with fakes (manager, orchestrator, guard, CLI plan and answers, badge markup); native badge, VoiceOver and SC-003 pending with Julien | 2026-10-15 |
| [FR-002: Unavailable pin, no fallback, pin kept](spec.md#fr-002) | `src-tauri/src/destination/insertion.rs`, `src-tauri/src/destination/outcome.rs`, `src/overlay/notice-model.ts` | `@spec FR-002: Unavailable pin, typed reason, pin kept`; `@spec AC-002: A pinned destination is activated even with return off`; `@spec AC-003: Pinned app not running reason shown` | ⚠️ Partial: `pinned_app_not_running` (closed) and `activation_failed` (running, not forward, several instances without the pinned one, unreadable foreground), `Recover`, pin kept (fakes); native quit/reopen pending | 2026-10-15 |
| [FR-003: One three-value Automatic Enter setting](spec.md#fr-003) | `src-tauri/src/settings.rs`, `src-tauri/src/automatic_enter/migration.rs`, `src-tauri/src/automatic_enter/mod.rs`, `src-tauri/src/settings_service/descriptor_data.rs`, `src-tauri/src/settings_service/domains/output.rs`, `src-tauri/src/settings_service/public.rs`, `src-tauri/src/settings_service/errors.rs`, `src-tauri/src/shortcut/mod.rs`, `src/components/settings/automatic-enter.tsx`, `src/components/settings/automatic-enter-state.ts`, `src/components/settings/advanced/AdvancedSettings.tsx` | `@spec FR-003: One three-value Automatic Enter setting`; `@spec FR-003: Migration from auto-submit, key kept`; `@spec AC-004: …`; `@spec AC-006: Invalid Automatic Enter lists the values` | ⚠️ Partial: migration both gate states in one binary, real binary on isolated stores, gated descriptors/schema, UI selector (markup and Chromium keyboard); native migration of a real store and the older-binary check pending | 2026-10-15 |
| [FR-004: At most one Enter, gated by outcome and option](spec.md#fr-004) | `src-tauri/src/automatic_enter/policy.rs`, `src-tauri/src/automatic_enter/transaction.rs`, `src-tauri/src/automatic_enter/rendezvous.rs`, `src-tauri/src/automatic_enter/app.rs`, `src-tauri/src/destination/delivery.rs`, `src-tauri/src/destination/manager.rs`, `src-tauri/src/clipboard.rs`, `src-tauri/src/paste_tx/mod.rs`, `src-tauri/src/paste_tx/macos.rs`, `src-tauri/src/paste_tx/windows.rs`, `src-tauri/src/smart_paste/app.rs` | `@spec FR-004: At most one Enter, gated by outcome and option`; `@spec FR-004: Policy and key frozen at insertion entry`; `@spec FR-004: One Enter authority on the session path`; `@spec FR-004: Smart-paste Enter decided by 009's authority`; `@spec FR-004: Reliable-paste signals for the Enter authority`; `@spec AC-005: …` | ⚠️ Partial: full matrix, claim races, focus, rendezvous orders and the emitter scan pass (fakes, real manager/guard/orchestrator, real smart-paste dispatch with injected outcomes); real key count, timing and Chromium double read pending natively; Windows code compile-checked on Windows only (pending) | 2026-10-15 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| [AC-001](spec.md#ac-001) | `src-tauri/src/destination/pin_tests.rs`, `src-tauri/src/destination/session_tests.rs`, `src-tauri/tests/pin_destination.rs`, `src/overlay/pin-state.test.ts`, `src/overlay/indicator-render.test.tsx` | ⚠️ Partial: pin from the badge or CLI routes this and every following dictation (five from three other apps, automated SC-003), pin mark and "Destination: <name>, pinned", badge shown with return off when pinned, hidden when unsupported or overlay none; native window, VoiceOver and focus pending |
| [AC-002](spec.md#ac-002) | `src-tauri/src/destination/pin_tests.rs`, `src-tauri/tests/pin_destination.rs` | ⚠️ Partial: other app/cross current-only with the pin kept, unpin before freeze restores the capture (badge override dropped), return off → frontmost at the end, after freeze only next dictations, pinned route activated with return off (fakes); native pending |
| [AC-003](spec.md#ac-003) | `src-tauri/src/destination/pin_tests.rs`, `src-tauri/tests/pin_destination.rs`, `src/overlay/notice-model.ts` | ⚠️ Partial: exact reasons per case, no fallback, `Recover`, pin kept, reopened app with a new PID receives the text (fakes); indicator notice text; native quit/reopen, history row and 010 bubble pending (010 not delivered) |
| [AC-004](spec.md#ac-004) | `src-tauri/src/automatic_enter/migration_tests.rs`, `src-tauri/tests/automatic_enter_migration.rs`, `src-tauri/tests/settings_validation/*`, `src/components/settings/automatic-enter-state.test.ts`, `src/components/settings/output-settings-render.test.tsx`, `tests/pin-enter-ui.spec.ts` | ⚠️ Partial: every switch × key, stored value authoritative, invalid → never, closed build never writes or reads it and keeps `auto_submit`; one control per build; keyboard selection announced (Chromium); native and older-binary checks pending |
| [AC-005](spec.md#ac-005) | `src-tauri/src/automatic_enter/policy_tests.rs`, `src-tauri/src/automatic_enter/transaction_tests.rs`, `src-tauri/src/automatic_enter/rendezvous.rs`, `src-tauri/src/automatic_enter/scan_tests.rs`, `src-tauri/tests/enter_gate.rs`, `src-tauri/tests/enter_insertion.rs`, `src-tauri/tests/enter_platform.rs` (ignored, native) | ⚠️ Partial: 0/1 for the full option × route × completion matrix and all eight smart-paste outcomes, none/failure/recovery/stale/cancel/focus suppress, 50–100 ms after t0 with a fake clock, rendezvous both orders and chord failure after an early read, only the authority claims; native key count, timing and Chromium pending |
| [AC-006](spec.md#ac-006) | `src-tauri/src/settings_service/domains/destination_tests.rs`, `src-tauri/tests/pin_enter_cli.rs` | ⚠️ Partial: real binary: closed build `usage_error` and `automatic_enter` unknown; open build: `unavailable`/5 offline, change options `usage_error`/2, `auto_submit` like any unknown key, `automatic_enter` validation lists the three values, unknown outcome on a late instance; reply shapes and lookup `not_found` in the domain; routed pin against a real running Handy pending natively |

| SC | Evidence | Status |
| --- | --- | --- |
| [SC-001](spec.md#sc-001) | `src-tauri/tests/enter_gate.rs` (144 cases), `src-tauri/src/automatic_enter/policy_tests.rs` (180 cases) | ⚠️ Automated matrix at 100%; native counts pending |
| [SC-002](spec.md#sc-002) | `src-tauri/tests/automatic_enter_migration.rs`, `src-tauri/src/automatic_enter/migration_tests.rs` | ⚠️ Every combination in both gate states; real older binary pending (release evidence) |
| [SC-003](spec.md#sc-003) | `src-tauri/tests/pin_destination.rs` (automated form) | ⚠️ Native ChatGPT run pending with Julien |

## Delivered names

`destination::pin::{PinState, PinView, PinnedApp, PinChange, NoRunningApplication}`; `DestinationManager::{pinned, pin_current, pin_identifier, unpin, unpin_from_badge}`; `DestinationSource::Pinned`; `DestinationFailure::PinnedAppNotRunning`; `DestinationSession::{start_selection, pin}`; `DestinationSnapshot::{source, pin}`; `InsertionReport::context` (boxed); `session::badge_visible(return_enabled, pinned, support, overlay_style)`; `automatic_enter::{PRE_ENTER_SETTLE, fresh_install_policy, policy_of, legacy_auto_submit, legacy_view}`; `automatic_enter::policy::{Completion, Suppression, EnterFacts, evaluate_enter, route_has_destination}`; `automatic_enter::transaction::{EnterTransaction, EnterSnapshot, EnterPorts, EnterClaims, KeyEmitter, EnterResult}`; `automatic_enter::rendezvous::{ReliableRendezvous, ReceiptSink}`; `clipboard::{EnterRoute, paste_routed}`; `settings::AutomaticEnterPolicy`, `AppSettings::automatic_enter`; `settings_service::domains::destination::DestinationDomain`; `SettingsError::no_running_application`; commands `pin_destination`, `unpin_destination`, `change_automatic_enter_setting`.

## Differences from the plan

- `claim_enter` is also `Stale` once the manager's generation moved on (a cancellation after publication), so a late Windows reliable claim honors a cancellation (review r8 warning); a live session's cancellation still reports `Disposition(Cancelled)`.
- The `ReceiptSink` signals of the reliable paste are defined by the Enter authority (`automatic_enter::rendezvous`) and re-exported to `paste_tx`; `try_reliable_paste` takes `(auto_submit, key)` as one argument.
- `automatic_enter` is read leniently (an invalid value is absent, then `never`), and a closed build drops it from its in-memory settings without writing, so the UI shows `AutoSubmit` there (review r8 warning on the closed build reading an open store).
- No policy+key batch command: the UI changes them with two separate selectors, each one write.
- Story 5's "listing never, always and pin_only": the refusal keeps `validation_failed` with `canonical_value`, its fixed message lists the three values.
- Salvage does not probe `auto_submit` in the open build: without a descriptor it is never written, so its raw value survives anyway.
- `src/bindings.ts` is generated from one `specta_builder()` with a drift test (`bindings_tests`), identical in both gate states.
- UI versus mockup: Pin and Unpin are listed above the application list (spec Behavioral AC), where the mockup shows Unpin at the bottom; the mockup's "This dictation only" hint is not shown; the Pin entry and the pin mark use test ids derived from the contract (`recording-indicator-pinned-menu-pin`, `recording-indicator-pinned-destination-mark`), which has no node for them.
- The reliable paste's read and injection signalling is one shared helper of `paste_tx` (`record_read`, `inject`), used by the macOS and Windows transactions.
- The conventions baseline (`.specs/conventions/baseline.json`) records the command registry moved out of `run` into `specta_builder` and the shorter `run`; no new violation otherwise.
- Three existing test lints of `dev` (engine supervisor, GGUF metadata, portable marker) were fixed so `cargo clippy --all-targets -D warnings` passes.

## Pending

- **Native macOS, required before Implemented (with Julien):** `.specs/native-proofs-checklist.md`, section 009, with `scripts/pin-enter-native-proof.ts`.
- **Release evidence, never a gate:** Windows native run, Linux observation, older Handy binary (checklist §5).
- **Not delivered by 009:** 010's bubble (the pinned reasons are shown by 006's indicator notice until then), the Penflow certification run and visual baselines (`/spec-test`), the delivery review (orchestrator).
