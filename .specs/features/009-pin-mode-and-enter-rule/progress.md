# Progress — 009-pin-mode-and-enter-rule

Run `impl-20261015` (Claude Code, `/spec-implement`, model claude-opus-5-5), branch `feature/009-pin-mode` from `dev` `09cc1ea2`.
Gates before code: `livespec validate --pre-impl` exit 0 (semantic readiness covered, no finding) and `livespec validate --progression implement` exit 0 (`Progression: READY`), both with the review budget of the receipts (`--review-max-chars 180000`); proofs under `proofs/impl-20261015/`. Delivery lane: full (High criticality); the end-of-feature delivery review is run by the orchestrator, not by this run.

| Step | Status | Files | Tests run | Result | Updated at |
|---|---|---|---|---|---|
| 0a | Done | `src/overlay/pin-state.test.ts`, `src/components/settings/automatic-enter-state.test.ts` | `bun test src/overlay/pin-state.test.ts src/components/settings/automatic-enter-state.test.ts` | RED as required before code (modules absent, exit 1, `proofs/impl-20261015/step0a-red.*`), GREEN after Step 6 (17 pass) | 2026-10-15 |
| 1 | In Progress (native part pending with Julien) | `src-tauri/tests/destination_platform.rs` (pin lookup on the real workspace), `src-tauri/tests/enter_platform.rs` (final focus check timing with a key spy), `scripts/pin-enter-native-proof.ts` | `cargo test --features destination-return,smart-paste-dispatch --test destination_platform --test enter_platform` (compiled, ignored by default) | Automated part done; the native run is not executed (no app activation, key or clipboard on Julien's Mac); see "Native proofs pending" | 2026-10-15 |
| 2 | Done (automated) | `src-tauri/src/destination/{pin.rs,pin_tests.rs,manager.rs,session.rs,session_tests.rs,snapshot.rs,menu.rs,insertion.rs,context.rs,outcome.rs,delivery.rs,mod.rs}`, `src-tauri/src/commands/destination.rs`, `src-tauri/tests/pin_destination.rs` | `cargo test --lib destination` (4 combinations), `cargo test --test pin_destination` | Pass | 2026-10-15 |
| 3 | Done (automated) | `src-tauri/src/settings.rs`, `src-tauri/src/automatic_enter/{migration.rs,migration_tests.rs,mod.rs}`, `src-tauri/src/settings_service/{descriptor_data.rs,public.rs,errors.rs,cli.rs,cli/destination.rs,domains/{mod.rs,output.rs,destination.rs,destination_tests.rs}}`, `src-tauri/src/cli.rs`, `src-tauri/src/shortcut/mod.rs`, `src-tauri/src/lib.rs`, `src/bindings.ts`, 008 tests made gate-aware, `src-tauri/tests/{automatic_enter_migration.rs,pin_enter_cli.rs}`, `README.md`, `.specs/features/008-settings-cli/contracts/settings-cli.md` | `cargo test --test settings_validation --test settings_cli --test settings_domains --test settings_live --test smart_paste_settings --test automatic_enter_migration --test pin_enter_cli` (both gate states) | Pass | 2026-10-15 |
| 4 | Done (automated) | `src-tauri/src/automatic_enter/{policy.rs,policy_tests.rs,transaction.rs,transaction_tests.rs,rendezvous.rs}`, `src-tauri/tests/enter_gate.rs` | `cargo test --lib automatic_enter`, `cargo test --test enter_gate` | Pass (180 + 144 matrix cases) | 2026-10-15 |
| 5 | Done (automated) | `src-tauri/src/clipboard.rs`, `src-tauri/src/paste_tx/{mod.rs,macos.rs,windows.rs}`, `src-tauri/src/destination/delivery.rs`, `src-tauri/src/smart_paste/app.rs`, `src-tauri/src/automatic_enter/{app.rs,scan_tests.rs}`, `src-tauri/tests/enter_insertion.rs` | `cargo test --test enter_insertion`, emitter and `claim_enter` scan (`automatic_enter::scan_tests`) | Pass; Windows code is compiled on Windows only (pending release evidence) | 2026-10-15 |
| 6 | Done (automated) | `src/overlay/{pin-state.ts,destination-model.ts,destination-badge.tsx,destination-menu.tsx,use-destination.ts,indicator-stack.tsx,notice-model.ts,indicator.css}`, `src/components/settings/{automatic-enter.tsx,automatic-enter-state.ts,advanced/AdvancedSettings.tsx}`, `src/stores/settingsStore.ts`, 26 locales, `tests/pin-enter-ui.spec.ts`, `tests/harness/automatic-enter*` | `bun run test:indicator`, `bun run test:output-settings`, `bun run test:pin-enter-ui`, `bun run check:translations` | Pass; native window, VoiceOver and Penflow visual certification pending | 2026-10-15 |
| 7 | In Progress (native acceptance pending with Julien) | `proofs/impl-20261015/final/`, conventions fixes `69afc00b` | Final verification (see `proofs/impl-20261015/summary.txt`: tsc, lint, format, translations, build, 13 bun suites, all Playwright specs, cargo fmt, clippy lib/bins and all-targets, cargo test --lib and the full cargo test in the four feature combinations, conventions repo, progression, feature validation, doctor) | 40/40 exit 0; macOS native acceptance not run | 2026-10-15 |

## Delivery review round 1 — corrections

Review BLOCKED (one blocking finding), audit PASS; reports in `.reviews/delivery.json`. Fixed in the worktree, verification in `proofs/impl-20261015-fix1/summary.txt`:

| Finding | Correction | Tests |
|---|---|---|
| Badge choice with return off while pinned went to the frontmost app (blocking) | `insertion::activation_route` activates any badge choice and the pin, whatever the setting | `pin_route_tests::another_app_chosen_in_the_badge_with_return_off_while_pinned_is_activated`, `pin_destination::a_badge_choice_with_return_off_while_pinned_gets_its_enter_only_in_front` |
| Focus checked on the worker, key up to 1 s later on the main thread | `automatic_enter::key_press::GuardedKeyPress`: PID and identity checked again on the main thread right before the key, refused past 100 ms, one shared state so the result is exact | `key_press_tests` (6), `transaction_tests::the_emitter_outcome_is_reported_as_is_and_gets_the_expected_destination` |
| `output get` help names `auto_submit` with the gate open | gated help text, unchanged closed | build in both states |
| Unpin above the list, no "This dictation only" | Unpin at the bottom, hint on the other apps (26 locales), `MenuEntry.pinned` | `indicator-render.test.tsx`, `pin_tests::the_menu_marks_the_pinned_app_so_the_others_read_this_dictation_only` |
| Files over 400 lines, long functions | `pin_route_tests.rs`, `transaction_fixture.rs`/`transaction_flow_tests.rs`, `errors_pin_enter.rs`, `cli_destination.rs`; `commit_start` 25, `from_settings` 30, `AutomaticEnter` 27 lines | unchanged suites |

## Step 1 — feasibility notes

- The native read-only tests are written, compiled and ignored by default; they need a logged-in session and Accessibility for the runner, and they press no key: the Enter authority's key emitter is a spy there.
- The plan's macOS gate of Step 1 (no menu focus theft, ≤ 1 s activation without freeze, eligible Enter ≤ 100 ms, full transcript in ChatGPT and Comet when Enter lands) is not evidenced: Steps 2 to 6 were implemented on the automated evidence by instruction of the orchestrator (native proofs with Julien), and stay "Done (automated)" only.

## Native proofs pending with Julien

None is declared passed. Exact procedure: `.specs/native-proofs-checklist.md`, section "009 — Mode PIN et règle Entrée", with `scripts/pin-enter-native-proof.ts --binary <absolute release binary> --platform macos --out .specs/features/009-pin-mode-and-enter-rule/proofs/native-macos/ --interactive` on `bun run tauri build --bundles app --features destination-return,smart-paste-dispatch`.

1. Closed gate identical to `dev` (`handy-gates-closed.app`): `destination` usage error, `automatic_enter` unknown, Auto Submit only, Enter 50 ms after paste.
2. Real migration of a store (`auto_submit` true + `ctrl_enter` → `always`), then back to the closed build with its behavior kept.
3. SC-003: pin ChatGPT from the badge, five dictations from Safari, Notes, TextEdit, Safari, Notes; badge pin mark, VoiceOver "Destination: ChatGPT, pinned", menu without focus theft.
4. Other app, cross and Unpin while pinned (current dictation only; Unpin restores the captured app).
5. Return off: no badge without a pin; with a CLI pin, ChatGPT activated and receives the text.
6. Pinned Notes quit (no insertion, "pinned but not open" ≥ 4 s, History), reopened (inserted), not brought forward (`activation_failed`), pin kept.
7. Enter counts for Never/Always/Only in PIN mode, pinned and unpinned, in a counting test field; t0 → key 50–100 ms (QE target).
8. No Enter: method none, smart paste "not read", app quit before insertion, focus switch after insertion, next dictation before the Enter.
9. Chromium double read (ChatGPT, Comet): the full transcript in the field when Enter lands (r8 review risk: 007 publishes "read observed" at the first read).
10. macOS beta reliable paste on the session (`handy-destination-only.app`, `reliable_paste` true): Enter only after a read.
11. Settings window: one Automatic Enter control, keyboard and VoiceOver.
12. CLI with Handy running: pin/status/unpin answers and badge parity; absent identifier `not_found`.

Pending release evidence (never a gate): Windows native run, Linux observation, older Handy binary (checklist §5).
