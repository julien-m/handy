---
feature: default-input-per-app
title: Default Input per App Implementation
status: In Progress
created: 2026-10-29
updated: 2026-10-29
---

# Implementation — 013-default-input-per-app

Status: In Progress (automated evidence complete; native macOS proof pending with Julien). Branch `feat/default-input-per-app` from `dev` `ab08ca1b`, behind 006's `destination-return` rollout gate for every visible and behavioral part (badge entry, insertion route, CLI domain actions); the setting, its descriptor and its lenient read exist in every build. Read [the functional source](spec.md), [the plan](plan.md) and [the progress](progress.md).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: One Remember entry per destination type](spec.md#fr-001) | `src-tauri/src/destination/default_input/capture.rs`, `src-tauri/src/destination/default_input/services.rs`, `src-tauri/src/destination/default_input/instance.rs`, `src-tauri/src/commands/default_input.rs`, `src/overlay/default-input-entry.tsx`, `src/overlay/use-default-input.ts`, `src/overlay/default-input-model.ts`, `src/overlay/destination-menu.tsx` | `@spec FR-001: One Remember entry per destination type`, `@spec AC-001: Addressing tool found inside the bundle`, `@spec AC-001: Remember entry in the badge menu` | ✅ Implemented | 2026-10-29 |
| [FR-002: Field fingerprint without content](spec.md#fr-002) | `src-tauri/src/destination/default_input/fingerprint.rs`, `src-tauri/src/destination/default_input/capture.rs`, `src-tauri/src/destination/default_input/tree.rs`, `src-tauri/src/destination/default_input/macos_tree.rs` | `@spec FR-002: Fingerprint without content`, `@spec AC-002: Generated DOM ids are never kept` | ✅ Implemented | 2026-10-29 |
| [FR-003: Terminal captured through the bundle's tool](spec.md#fr-003) | `src-tauri/src/destination/default_input/terminal.rs`, `src-tauri/src/destination/default_input/runner.rs`, `src-tauri/src/destination/default_input/macos_bundle.rs` | `@spec FR-003: Terminal captured through the bundle's tool` | ✅ Implemented | 2026-10-29 |
| [FR-004: Persisted per application, one validation](spec.md#fr-004) | `src-tauri/src/destination/default_input/model.rs`, `src-tauri/src/settings.rs`, `src-tauri/src/settings_service/descriptor_data.rs`, `src-tauri/src/settings_service/public.rs`, `src-tauri/src/settings_service/protocol.rs`, `src-tauri/src/settings_service/errors.rs`, `src-tauri/src/settings_service/errors_pin_enter.rs`, 008 contract | `@spec FR-004: One shared validation for every writer` | ✅ Implemented | 2026-10-29 |
| [FR-005: Route decided after activation, without 011](spec.md#fr-005) | `src-tauri/src/destination/default_input/route.rs`, `src-tauri/src/destination/insertion.rs`, `src-tauri/src/destination/guard.rs`, `src-tauri/src/destination/manager.rs`, `src-tauri/src/destination/app.rs` | `@spec FR-005: Route decided after activation, without 011`, `@spec AC-005: Editable focus keeps today's insertion` | ✅ Implemented | 2026-10-29 |
| [FR-006: Unique field found, focused, verified](spec.md#fr-006) | `src-tauri/src/destination/default_input/search.rs`, `src-tauri/src/destination/default_input/route.rs`, `src-tauri/src/destination/default_input/fingerprint.rs`, `src-tauri/src/destination/context.rs`, `src-tauri/src/destination/outcome.rs`, `src-tauri/src/destination/insertion.rs`, `src-tauri/src/recovery/types.rs`, `src-tauri/src/recovery/decision.rs`, `src/overlay/recovery-preview.ts` | `@spec FR-006: Bounded breadth-first search, unique match`, `@spec AC-006: Unique field focused and verified`, `@spec AC-006: Every attribute equal, never a password`, `@spec AC-006: Remembered target not found, nothing sent` | ✅ Implemented | 2026-10-29 |
| [FR-007: Terminal typed through the tool, no Enter](spec.md#fr-007) | `src-tauri/src/destination/default_input/dispatch.rs`, `src-tauri/src/destination/default_input/terminal.rs`, `src-tauri/src/destination/default_input/runner.rs`, `src-tauri/src/destination/delivery.rs`, `src-tauri/src/destination/guard.rs`, `src-tauri/src/automatic_enter/policy.rs`, `src-tauri/src/automatic_enter/transaction.rs` | `@spec FR-007: Typed through the tool, without Enter`, `@spec AC-007: Escape-safe chunks, no Enter`, `@spec AC-007: Terminal skips 010; unavailable never pastes`, `@spec AC-007: No automatic Enter after a terminal delivery`, `@spec AC-010: No shell, closed stdin, bounded then killed` | ✅ Implemented | 2026-10-29 |
| [FR-008: Settings list and CLI to forget](spec.md#fr-008) | `src/components/settings/default-inputs-setting.tsx`, `src/components/settings/default-inputs-model.ts`, `src/components/settings/advanced/AdvancedSettings.tsx`, `src-tauri/src/commands/default_input.rs`, `src-tauri/src/settings_service/domains/destination.rs`, `src-tauri/src/settings_service/domains/destination_default_inputs.rs`, `src-tauri/src/settings_service/cli/destination.rs`, `src-tauri/src/cli_destination.rs`, 26 locales | `@spec AC-008: Remembered inputs listed and forgotten`, `@spec AC-009: default-input-remove and default-input-clear` | ✅ Implemented | 2026-10-29 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| AC-001 | `src-tauri/src/destination/default_input/services.rs`, `src-tauri/src/destination/default_input/capture_tests.rs`, `src/overlay/default-input-model.test.ts`, `src/overlay/default-input-render.test.tsx` | ✅ (automated; native pending) |
| AC-002 | `src-tauri/src/destination/default_input/fingerprint_tests.rs`, `src-tauri/src/destination/default_input/capture_tests.rs` | ✅ (automated; native pending) |
| AC-003 | `src-tauri/src/destination/default_input/terminal_tests.rs`, `src-tauri/src/destination/default_input/capture_tests.rs` | ✅ (fake tool; native pending) |
| AC-004 | `src-tauri/src/destination/default_input/model_tests.rs`, `src-tauri/src/settings_service/domains/destination_default_inputs_tests.rs` | ✅ (also covered by the settings CLI and validation integration suites) |
| AC-005 | `src-tauri/src/destination/default_input/route_tests.rs`, `src-tauri/src/destination/default_input/orchestrator_tests.rs` | ✅ (automated; native pending) |
| AC-006 | `src-tauri/src/destination/default_input/search_tests.rs`, `src-tauri/src/destination/default_input/route_tests.rs`, `src-tauri/src/destination/default_input/orchestrator_tests.rs`, `src-tauri/src/recovery/decision_tests.rs`, `src/overlay/recovery-preview.test.ts` | ✅ (fake tree; native pending) |
| AC-007 | `src-tauri/src/destination/default_input/terminal_tests.rs`, `src-tauri/src/destination/default_input/route_tests.rs`, `src-tauri/src/destination/default_input/orchestrator_tests.rs`, `src-tauri/src/automatic_enter/transaction_flow_tests.rs` | ✅ (fake tool; native pending) |
| AC-008 | `src/components/settings/default-inputs-model.test.ts`, `src/components/settings/default-inputs-render.test.tsx` | ✅ (headless render; native pending) |
| AC-009 | `src-tauri/src/settings_service/domains/destination_default_inputs_tests.rs` | ✅ (also covered by the settings CLI integration suite) |
| AC-010 | `src-tauri/src/destination/default_input/runner.rs`, `src-tauri/src/destination/default_input/terminal_tests.rs` | ✅ (no shell, bounded, text never in errors) |

## Files Created/Modified

- Created: `src-tauri/src/destination/default_input/` (`mod.rs`, `model.rs`, `fingerprint.rs`, `tree.rs`, `search.rs`, `route.rs`, `terminal.rs`, `runner.rs`, `services.rs`, `capture.rs`, `dispatch.rs`, `instance.rs`, `macos_tree.rs`, `macos_bundle.rs`, test files and `fake_tree.rs`), `src-tauri/src/commands/default_input.rs`, `src-tauri/src/settings_service/domains/destination_default_inputs{,_tests}.rs`, `src/overlay/{default-input-entry.tsx,use-default-input.ts,default-input-model.ts}` and tests, `src/components/settings/{default-inputs-setting.tsx,default-inputs-model.ts}` and tests.
- Modified: guard (`with_default`, `terminal_route`, `mark_addressed`, verdict per route), orchestrator (route prepared after 011's `begin`), `InsertionReport.addressed`, `RecoveryReason`/`DestinationFailure`/`BubbleReason` gain `default_field_not_found` and `default_terminal_gone`, 009 `Completion::AddressedTerminal` → `Suppression::AddressedTerminal`, delivery wraps the configured method in `AddressedMethod`, manager holds `DefaultInputServices` (installed on macOS by `destination::app::install`), settings field/descriptor/public/schema/errors, destination domain and clap actions, commands registered in `lib.rs`, `src/bindings.ts` regenerated, menu footer entry, Advanced settings section, bubble reasons, 26 locales, `README.md`, 008 contract (`settings-cli.md`, checked schema), integration test counts.

## Decisions taken in this run

See `spec.md` Clarifications: `terminal_identity_workspace`, `terminal_text_escapes`, `terminal_no_enter`, `activation_kept`, `secure_focus_unchanged`, `search_bounds`, `generated_dom_id`, `capability_by_bundle`, `cap_twenty`, `macos_only`.

## Differences from the plan

1. The search, route and capture read through one `TreeNode::read` (facts and children in one `AXUIElementCopyMultipleAttributeValues`), the call measured on Codex.
2. The settings section lists inputs only when the backend answers a support state (hidden while loading or when destination return is not offered), like `ReturnToAppSetting` except that a failed support query hides it.
3. `default_field_not_found` advises "Click in a text field" (010's focus advice without the no-field setting line); `default_terminal_gone` uses the open-the-app advice.

## Gaps

- **Penflow mockups and contract not updated.** `penflow/ui.pen` and `penflow/flow-ui-contract/contract.json` have no "Remember" entry, no "Remembered fields" section and no new bubble reasons. New test ids: `<screen>-menu-remember-input`, `-remember-feedback`, `<output screen>-default-inputs` (`-row`, `-forget`, `-forget-all`, `-empty`, `-list`, `-error`, `-title`).
- **Windows:** no tree search (decision `macos_only`); the list is stored and shown but nothing is offered or applied.
- **Hook paths of the front:** the offer fetch and click-to-feedback path of the hooked components run only in the native overlay; their pure models and hook-free views are tested.

## Native proofs pending with Julien

1. **Codex field.** Click in the Codex composer, start a dictation, choose "Remember this field for Codex" in the badge menu (menu says "Field remembered for Codex"; Settings, Advanced lists "Field: Work with Codex"). Then click in the Codex conversation (focus on the "Conversation" group), pin Codex (or keep it as destination), dictate from another application: the text lands in the composer, the conversation is unchanged. Then open two Codex windows with the focused one lacking a composer, or rename nothing and type in a password field: no paste, bubble "The remembered field of Codex was not found".
2. **cmux terminal.** Start a dictation with cmux in front, choose "Remember this terminal" (Settings lists "Terminal: <workspace>"). Pin cmux, put another application in front, dictate: the text appears in that workspace's focused terminal, not submitted, the clipboard unchanged, no Enter even with Automatic Enter "Always". Dictate a text containing "backslash n" literally: no Enter. Close the workspace and dictate: bubble "The remembered terminal of cmux no longer exists". Restart cmux (`restore-session`) and dictate: record whether the workspace UUID survived.
3. **Unchanged path.** With a field remembered, click in another text field of the same app and dictate: pasted there as before; with 011's remembered input at the start, 011 wins.
4. **CLI.** `handy settings get destination_default_inputs --json`, `handy settings destination default-input-remove com.openai.codex`, `handy settings destination default-input-clear`.
