# Implementation — 008-settings-cli

Status: In Progress. Step 1 is Done within its planned primitive scope; Steps 2–7 remain Todo. Read [the functional source](spec.md) and [the technical plan](plan.md); inspect [current progress](progress.md) and [execution history](logs/2026-10-13.md).

## Requirement Mapping

| Requirement | File(s) | @spec Anchor | Status | Last Verified |
| --- | --- | --- | --- | --- |
| [FR-001: Headless CLI](spec.md#fr-001) | Inspect [entry](../../../src-tauri/src/main.rs), [actual parser](../../../src-tauri/src/cli.rs), [console wrapper](../../../src-tauri/src/settings_service/console_windows.rs) | `@spec FR-001: Keep CLI handles, detach GUI console`; `@spec FR-001: Fallible GUI console detachment` | ⚠️ Partial: historical parser/console path tested; settings subgroup/early path absent | 2026-10-13 |
| [FR-002: Generic settings](spec.md#fr-002) | None | None | ❌ Missing | 2026-10-13 |
| [FR-003: Shared validation and revision](spec.md#fr-003) | None | None | ❌ Missing: repository/all-writer migration unstarted | 2026-10-13 |
| [FR-004: Private routing](spec.md#fr-004) | Inspect [passive probe](../../../src-tauri/src/settings_service/instance_probe.rs), [private resources](../../../src-tauri/src/settings_service/platform_unix/resources.rs), [socket](../../../src-tauri/src/settings_service/platform_unix/socket.rs), [lease primitive](../../../src-tauri/src/settings_service/platform_unix/startup_fence.rs), [Windows pipe](../../../src-tauri/src/settings_service/platform_windows.rs) | `@spec FR-004: Owner-only filesystem resources`; `@spec FR-004: Private settings endpoint`; `@spec FR-004: Fence new startup against offline writes`; `@spec FR-004: Current-logon private settings pipe` | ⚠️ Partial: pinned primitives, accepted-peer authentication and pipe ownership boundary; complete routing/client/lease lifecycle absent | 2026-10-13 |
| [FR-005: Domain framework](spec.md#fr-005) | None | None | ❌ Missing | 2026-10-13 |
| [FR-006: Output and errors](spec.md#fr-006) | None | None | ❌ Missing | 2026-10-13 |

## Acceptance Criteria Mapping

| AC | Test File | Status |
| --- | --- | --- |
| [AC-001](spec.md#ac-001) | Inspect [parser/console tests](../../../src-tauri/tests/settings_platform/cli_console.rs) | ⚠️ Partial: historical flags/defaults/values/combinations and dispatch; settings subgroup/M1 native path unimplemented |
| [AC-002](spec.md#ac-002) | None | ❌ Missing |
| [AC-003](spec.md#ac-003) | None | ❌ Missing |
| [AC-004](spec.md#ac-004) | None | ❌ Missing |
| [AC-005](spec.md#ac-005) | Inspect [Unix tests](../../../src-tauri/tests/settings_platform/unix.rs), [Unix regressions](../../../src-tauri/src/settings_service/platform_unix/tests.rs), [pipe boundary tests](../../../src-tauri/tests/settings_platform/windows.rs) | ⚠️ Partial: 22-test primitive suite; full live/offline/deadline/client/effects/reclamation absent, no complete criterion certified |
| [AC-006](spec.md#ac-006) | None | ❌ Missing |
| [AC-007](spec.md#ac-007) | None | ❌ Missing |
| [AC-008](spec.md#ac-008) | None | ❌ Missing |

No complete AC/SC is certified. The original partial mapping was correctly not ready and is historical; its source paths/hashes predate current corrections and are not current acceptance evidence. New whole-AC mappings/reviews and trusted policy-078.2 Cargo captures are required when full behavior exists.

## Current Step 1 proof and audit disposition

Inspect [final summary](proofs/current-run-step1/minimal-final2-summary.json) and [independent scoped review](proofs/current-run-step1/minimal-final2-review.json): current 14-file hashes match, 22 passed/0 failed/0 ignored, real Swift bridge, clippy/Cargo fmt exit 0. Prior frontend types/lint/global format exit 0 applies to unchanged frontend. Hard caps pass (294/300 file lines; 47/50 function lines). Inspect [actual bridge](proofs/current-run-step1/bridge.json) and [audit output](proofs/current-run-step1/audit3.stdout): exit 0, 0 blocker/2 major/7 minor/2 nit, with dispositions recorded in the final review; no zero-finding claim.

Unix accept verifies kernel UID before returning a stream; canonical directory0700/identity guards remain without process umask/pathname chmod. Windows accept consumes its owned handle, verifies client SID before File transfer and drops errors/refusal; the unauthenticated getter was removed. Effective pipe major was fixed with assertion RED101/GREEN22; FreeConsole major was a false positive against [Microsoft API remarks](https://learn.microsoft.com/en-us/windows/console/freeconsole), with console source unchanged. Safety/name/sample fixes were verified; soft FFI size observations remain explicit, bounded allocation/cleanup tradeoffs. Root corrected the official Plan Review timestamp; plan Next Action is continuation guidance and oldest-supported-Windows floor remains a release-policy question.

Failed/closed socket paths remain intentionally retained, avoiding same-UID check/unlink deletion races. Step 5 still owns client authentication before bytes and lease-proven reclamation. Step 7 owns full mandatory macOS settings/new-app/M1/live-effects evidence. Injected wrong UID/verifier tests with actual macOS resources do not prove Windows SID or a real second account. Windows/Linux/older-binary/second-account checks remain pending release evidence; no complete AC or feature PASS.

## Files Created/Modified

The final summary binds 14 existing/current source and test paths. Inspect [entry](../../../src-tauri/src/main.rs), [parser](../../../src-tauri/src/cli.rs), [console wrapper](../../../src-tauri/src/settings_service/console_windows.rs), [passive probe](../../../src-tauri/src/settings_service/instance_probe.rs), [Unix facade](../../../src-tauri/src/settings_service/platform_unix.rs), [resources](../../../src-tauri/src/settings_service/platform_unix/resources.rs), [socket](../../../src-tauri/src/settings_service/platform_unix/socket.rs), [startup fence](../../../src-tauri/src/settings_service/platform_unix/startup_fence.rs), [Unix tests](../../../src-tauri/src/settings_service/platform_unix/tests.rs), [Windows primitives](../../../src-tauri/src/settings_service/platform_windows.rs), [test facade](../../../src-tauri/tests/settings_platform.rs), [parser tests](../../../src-tauri/tests/settings_platform/cli_console.rs), [Unix boundary tests](../../../src-tauri/tests/settings_platform/unix.rs) and [pipe boundary tests](../../../src-tauri/tests/settings_platform/windows.rs).

## Historical observations

Inspect [execution history](logs/2026-10-13.md) for old sandbox/audit initialization failures, unsupported-runner captures, rejected partial mapping and superseded whole-AC step-block interpretation. Inspect [owned bootstrap restoration](proofs/current-run-step1/bootstrap-surgical-restore.json) and native permission experiments as historical mechanism investigation only. No successful build/RED/PASS is attributed to the abandoned bootstrap construction. Historical decoder/spool/replay artifacts were removed under the minimal plan; no replay daemon or legacy compatibility listener is part of 008.
