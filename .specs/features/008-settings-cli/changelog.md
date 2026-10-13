# Changelog - 008-settings-cli

## 2026-10-12 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** independent spec review complete and ready (reviewer model claude-opus-5-5); Progression: READY for plan
- **Author:** Claude Code (spec-specify --auto)

<!-- finalize:spec-specify:2026-10-12:d83a0c90 -->

### 2026-10-12 — Plan: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle only (Draft to Planned via finalize); normative requirements unchanged
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Artifacts:** 7 implementation steps; 2 sequence diagrams, 1 state diagram, 1 logical ER diagram; Rust/native tests and QE gates
- **Decisions:** Shared atomic commit service before CLI enablement; private OS IPC, legacy-start fences; same-executable Windows console spike; output-only initial domain
- **Author:** Codex (gpt-6.1-sol, spec-plan --auto)

<!-- finalize:spec-plan:2026-10-12:4dabd6e8 -->

### 2026-10-12 — Plan Update: Align 007 integration with smart paste

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md updated)
- **AC impacted:** None (pre-implementation; AC-006 integration clarified)
- **Author:** Codex (gpt-6)
- Replace retired clipboard-window/cancellation dependencies with the smart_paste output extension, six honest outcomes and the changeCount restoration guard.
- Preserve SettingsRepository, commit_mutation, Domain trait, all seven implementation steps, Windows reliable paste and Linux methods.
- Refresh independent spec/plan reviews and implementation progression evidence; no runtime success claimed.

<!-- finalize:spec-plan:2026-10-12:7b940570 -->

### 2026-10-13 — Plan: Resume existing technical plan and refresh independent semantic review

- **Type:** Feature
- **Spec modified:** No normative changes; lifecycle metadata through official finalization only
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 planned, no runtime evidence claimed
- **Author:** Codex

<!-- finalize:spec-plan:2026-10-13:e428ba3d -->

### 2026-10-13 — Feature: Partial isolated startup spike; Step 1 blocked

- **Type:** Feature
- **Spec modified:** Lifecycle status only, through official finalization; requirements and plan unchanged.
- **Code modified:** Four isolated settings_service modules and two private test files; no production wiring or settings writer.
- **AC impacted:** AC-001 and AC-005 partial primitives only; no complete AC certified.
- **Author:** Codex
- Standalone native diagnostic 19 passed, 0 failed; independent local reviews resolved five defects. Required Cargo remains blocked before tests by Swift macro subprocess permission error; Windows/Linux/legacy/two-user/worker handoff native proofs remain pending.
- Conventions proof blocked: missing .specs/conventions-gates.yaml, no receipt. Steps 2–7 remain Todo; no commit, push or merge.

<!-- finalize:spec-implement:2026-10-13:61bf0cb4 -->

### 2026-10-13 — Refactor: Resume partial local validation

- **Type:** Refactor
- **Spec modified:** No
- **Code modified:** src-tauri/tests/settings_platform.rs, src-tauri/tests/settings_platform/handoff.rs
- **Change:** Partial local validation resumed with the supported Apple Intelligence build stub; test fixtures return `Result` and propagate failures with `?`.
- **Local proof:** Full Cargo suite 305 passed, 16 ignored, 0 failed; formatting, frontend regressions and feature/repository conventions PASS with unchanged baseline and 0 new violations.
- **Remaining gate:** Step 1 Blocked; Windows/Linux mechanisms and production startup/worker/deferred historical invocation handoff absent. Steps 2–7 Todo; native platform evidence pending; no AC/SC certified.
- **AC impacted:** AC-001, AC-005 (partial diagnostics only)
- **Author:** Codex

Inspect [the current checkpoint](progress.md), [partial mappings](implementation.md) and [final validation summary](stub-result-summary.json) before finalizing In Progress.

<!-- finalize:spec-implement:2026-10-13:9b1995bc -->

### 2026-10-13 — Plan Update: Minimal CLI scope and native evidence policy

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None (plan.md refined only)
- **AC impacted:** AC-001, AC-005 (planning and evidence policy only; requirement IDs unchanged)
- **Author:** Codex (gpt-6.1-sol)
- Reuse current partial Step 1 passive/private-resource primitives; plan removal of obsolete legacy decoder/spool/reservation and handoff tests during implementation.
- Preserve the shared transaction service, domain framework, DTO names, envelope and exits; detected older owner returns unavailable without store writes, with no worker/replay.
- Require macOS native evidence before Implemented; list exact Windows/Linux/older-binary/second-account checks as pending release evidence, never blocking steps, integration completion or dev merge.
- Keep the simple console-subsystem/FreeConsole Windows choice compilation-testable, with native shell/GUI observations pending.
- Refresh independent spec/plan reviews, structural validation, Analyze and implement progression using gpt-6.1-sol and the explicitly declared 160000-character review budget.
- Existing spec status In Progress and partial progress/implementation checkpoints are preserved; implementation will refresh their superseded gates. No runtime acceptance or commit/push is claimed.

<!-- finalize:spec-plan:2026-10-13:ff08720c -->

### 2026-10-13 — Documentation: Normalize existing partial implementation map for plan goal validation

- **Type:** Spec Update
- **Spec modified:** No
- **Code modified:** None
- **AC impacted:** None; mappings, historical evidence and declared partial status unchanged
- **Author:** Codex (gpt-6.1-sol)
- Add required feature/title frontmatter and use the recognized Acceptance Criteria Mapping heading in implementation.md. These are documentary structural repairs required by spec-plan outcome validation, not implementation or acceptance work.

<!-- finalize:spec-plan:2026-10-13:1953f656 -->

## 2026-10-13 — Feature: Narrow retained native settings primitives

- **Type:** Feature
- **Spec modified:** No
- **Code modified:** Unix/private and Windows/private primitives, passive instance probe, console dispatch preparation, Cargo target dependency/features, platform tests; obsolete spool/decoder/handoff removed.
- **AC impacted:** AC-001, AC-005 (partial preparation, not certified)
- **Author:** Codex
- **Outcome:** Local 10/10 primitive tests, clippy and fmt pass; official typed capture blocked by LiveSpec tool defect; Steps 2–7 not run. Read [the checkpoint](progress.md).

## 2026-10-13 — Bugfix: socket creation avoids substituted-file chmod

- **Type:** Bugfix
- **Spec modified:** No
- **Code modified:** src-tauri/src/settings_service/platform_unix.rs
- **AC impacted:** AC-005 (primitive subset only; no complete acceptance certification)
- **Author:** Codex
- Native targeted tests: 13 passed, 0 failed; source formatting passed. Official LiveSpec Cargo capture invalid (unsupported runner, missing fresh report/assertion trace); implement phase blocked before Step 2.
- Read [the official tool defect](../../tool-defects/2026-10-13-cargo-runner-lacks-native-execution-certification.md) and [the current mappings](implementation.md).

## 2026-10-13 — Feature: Step 1 parser and console predicate verified

- **Type:** Feature
- **Spec modified:** No; plan and registry untouched by this checkpoint.
- **Code modified:** Actual CLI parser, Windows startup dispatch and platform integration tests.
- **AC impacted:** AC-001, AC-005 partial only; every AC/SC remains uncertified.
- **Author:** Codex
- Actual historical parser coverage includes all 13 flags, `-f`, defaults, value/combinational paths and the production console predicate. RED exit 101/E0599; GREEN exit 0, 17 passed, 0 failed, 0 ignored; scoped formatting/diff exit 0; independent Step 1 re-review PASS reported by supervisor.
- Current Step 1 Blocked at mandatory audit; Steps 2–7 Todo. Historical blockers and receipts preserved, with one current resume table. Foreign/old-binary/second-account native checks remain pending release evidence; no native PASS inferred.
- Inspect [the current checkpoint](progress.md) and [GREEN transcript](proofs/implement-resume-step1/parser-green.out).

## 2026-10-13 — Documentation: Distinguish local Step 1 PASS from blocked mandatory audit

- **Type:** Spec Update
- **Spec modified:** No; plan and registry untouched.
- **Code modified:** None.
- **AC impacted:** None certified; existing Partial/Missing mappings retained.
- **Author:** Codex
- Mandatory audit exited 1 without a verdict: Codex failed exit 4; `WARNING could not create PATH aliases: Operation not permitted (os error 1)`; `Error: failed to initialize in-process app-server client: Operation not permitted (os error 1)`.
- Correct the current table to Step 1 Blocked, Steps 2–7 Todo. Local 17/17 tests and independent quality review PASS remain separate from the absent audit verdict; no retry, alternative method or Git/history action. Read [the checkpoint](progress.md).

## 2026-10-13 — Superseded checkpoint: Step 1 whole-AC rule misinterpreted as step blocker

Historical interpretation only; root correction below governs current work. No runtime defect was established.

- **Type:** Spec Update
- **Spec modified:** No; approved spec/plan unchanged.
- **Code modified:** Parser/entry, fallible Windows console wrapper, private Windows FFI/probes and platform tests preserved by this documentary checkpoint.
- **AC impacted:** AC-001 and AC-005 partial only; no complete AC/SC certified.
- **Author:** Codex
- **Prerequisite:** Approved minimal Step 1 scope and native evidence policy; independent current source review.
- **Required inputs:** Actual-bridge transcripts, whole-AC mapping/raw/schema/review, current goal and runtime verifier.
- **Action:** Record authoritative Step 1 Blocked by tool granularity, Steps 2–7 Todo; preserve earlier audit failure as history. Reject console-flash rollback against approved plan; retain Windows native check as pending release evidence.
- **Execution evidence:** GREEN 20 passed, 0 failed, 0 ignored; strict clippy, types, lint and format exit 0; independent partial-scope source review PASS. Mapping review complete=true, ready=false, errors=[]; AC-005 missing and official ingest exit 1. No execution capture was attempted with the rejected mapping.
- **Success criteria:** Local primitive/source checks met; typed per-step proof unavailable because complete AC-005 needs later production routing/effects. After-step audit NOT RUN because Step 1 is not Done. No staging, commit, push or merge; no delivery/certification claim.
- **Failure handling:** Preserve the exact tool defect and honest Partial/Missing mappings; stop before Step 2. Refresh stale plan-dependent reviews officially on authorized resume; never weaken whole-AC acceptance or fabricate step receipts.

Inspect [GREEN](proofs/current-run-step1/green2.stdout), [real Swift bridge](proofs/current-run-step1/bridge.json), [summary](proofs/current-run-step1/summary.json), [source review](proofs/current-run-step1/code-quality-review.json), [mapping](proofs/current-run-step1/acceptance-mapping.json), [raw reviewer JSON](proofs/current-run-step1/mapping-review-response.json), [schema](proofs/current-run-step1/mapping-response-schema.json), [not-ready review](proofs/current-run-step1/acceptance-review.json), [ingest command result](proofs/current-run-step1/mapping-command-results.json) and [official defect](../../tool-defects/2026-10-13-step-execution-proof-requires-whole-acceptance-criterion-bef.md).

## 2026-10-13 — Documentation: Root correction restores Step 1 review/audit gate

- **Type:** Spec Update
- **Spec modified:** No; spec/plan metadata owned by root.
- **Code modified:** None by this correction; retain actual parser/entry, console wrapper, session probes and bounded Windows FFI.
- **AC impacted:** AC-001/AC-005 partial only; no certified AC/SC.
- **Author:** Codex
- **Prerequisite:** Root re-read the Step Gate (spec-implement lines 303–318) and corrected the previous whole-AC step-block interpretation.
- **Required inputs:** Current source including untracked files, targeted/transverse checks, independent reviews/checkpoint and renewed audit; final whole-AC evidence remains a separate goal obligation after Step 7.
- **Action:** Set the authoritative checkpoint to Step 1 In Progress pending renewed audit; Steps 2–7 Todo. Root independently restores pipeline state through official CLI. Mark the earlier report superseded-supervisor-gate-interpretation, retaining its observed command/review and historical analysis.
- **Execution evidence:** Existing real-bridge suite 20 passed, 0 failed, 0 ignored; strict clippy/types/lint/format exit 0; independent source review PASS within Step 1 partial scope. Mapping remains complete=true/ready=false/errors=[] with AC-005 missing; ingest exit 1 and no execution capture with the rejected mapping. Renewed audit has not yet passed.
- **Success criteria:** No runtime defect is established by correct refusal to certify a partial AC. Step 1 Done waits for renewed audit/reviews including untracked source; no Step 2 start or certification. The generic execution-receipt task is proved after all plan steps, not used as a whole-AC prerequisite for Step 1.
- **Failure handling:** Keep approved criteria and final trusted whole-AC Cargo certification mandatory. Preserve native pending evidence and the known branch-metadata audit minor for root resolution. No bypass, fake evidence, archive success or Git history action.

Read [the authoritative checkpoint](progress.md), [partial mappings](implementation.md) and [the corrected report](../../tool-defects/2026-10-13-step-execution-proof-requires-whole-acceptance-criterion-bef.md). Inspect [GREEN](proofs/current-run-step1/green2.stdout), [independent source review](proofs/current-run-step1/code-quality-review.json) and [not-ready whole-AC review](proofs/current-run-step1/acceptance-review.json) for the separate observations.

## 2026-10-13 — Documentation: Refresh Recent Activity and retain socket failure paths safely

- **Type:** Spec Update
- **Spec modified:** No; no feature status or other feature metadata changed.
- **Code modified:** None by the documenter; ongoing source corrections owned by worker.
- **AC impacted:** AC-001/AC-005 partial only; no certified AC/SC.
- **Author:** Codex
- **Prerequisite:** Renewed audit found stale registry Recent Activity; Step 1 remains In Progress.
- **Required inputs:** Current global changelog, registry activity markers and actual socket failure/close ownership behavior.
- **Action:** Refresh the exact latest ten activity entries newest first; retain all other feature metadata. Document intentional fail-closed pathname retention because check-then-unlink cannot exclude same-UID replacement; leave lease-proven reclamation to Step 5.
- **Execution evidence:** Source inspection only for this correction. Earlier 20/20/source-quality PASS remains bound to its old snapshot; new worker source changes await fresh tests/review. Inspect [replacement tests](../../../src-tauri/src/settings_service/platform_unix/tests.rs) and [owned-path retention tests](../../../src-tauri/tests/settings_platform/unix.rs); no new execution result claimed.
- **Success criteria:** Registry activity reflects the current ordered changelog, while Step 1 is not Done and Steps 2–7 remain Todo. Fresh post-correction checks and audit remain pending.
- **Failure handling:** Preserve history and partial mappings; never delete a substituted endpoint, imply full stale-socket reclamation or advance without accepted Step 1 evidence.

Read [the current implementation disposition](implementation.md#ongoing-step-1-corrections-and-socket-failure-path-disposition).

Inspect [the registry activity](../../README.md#recent-activity) and [the global changelog](../../changelog.md) as historical context.

## 2026-10-13 — Plan Update: Bounded Unix channel mechanism replaces socket-mode prescription

- **Type:** Spec Update
- **Spec modified:** No; FR/AC IDs, seven steps, business scope and release evidence policy unchanged.
- **Code modified:** None by documenter; worker owns pending correction.
- **AC impacted:** AC-005 mechanism only; no certified criterion.
- **Author:** Codex
- **Prerequisite:** Root authorized the finding-required refinement from actual process-umask audit finding and independent minimal-channel review.
- **Required inputs:** Current plan, exact mechanism decision, primary informative OS references, native experiments and surgical restoration evidence.
- **Action:** Replace socket-mode security prescription with canonical current-UID directory0700/identity guards and kernel peer-UID enforcement on both endpoints before application bytes. Prohibit process-mask/pathname-permission mutation or bootstrap permission framework; retain fail-closed paths and Step5 lease-proven reclamation.
- **Execution evidence:** Documentary mechanism edit only. Darwin listener-fd fchmod returned EINVAL; pinned-path open EOPNOTSUPP. Exact owned bootstrap hunks restored with matching prior hashes; no successful compile, RED/PASS or current mechanism tests claimed. Earlier local tests/source reviews remain historical snapshots.
- **Success criteria:** Minimal scope and IDs preserved; macOS ACL caveat/mandatory peer authorization explicit. Worker correction, fresh actual-bridge/local/native proofs, independent source review, official plan review/progression and audit pending; Step1 In Progress, Steps2–7 Todo.
- **Failure handling:** Root refreshes official complete plan review after this plan/docs freeze; progression implement must return READY before worker implements the new production mechanism and before Step2 can follow accepted Step1. Keep full whole-AC certification mandatory; no bootstrap overbuild, fake current PASS or history action.

Read [refined plan](plan.md#routing-startup-fence-and-legacy-compatibility), [actual mechanism review](proofs/current-run-step1/minimal-channel-review.json), [listener-fd experiment](proofs/current-run-step1/native-socket-fchmod.stdout), [pinned-path experiment](proofs/current-run-step1/native-socket-pinned-fchmod.stdout) and [owned-hunk restoration](proofs/current-run-step1/bootstrap-surgical-restore.json).

## 2026-10-13 — Feature: Step 1 primitive gate completed; feature acceptance remains partial

- **Type:** Feature
- **Spec modified:** No; approved scope unchanged.
- **Code modified:** 14 source/test paths bound by final summary/review.
- **AC impacted:** AC-001/AC-005 partial only; no complete AC/SC certified.
- **Author:** Codex
- **Prerequisite:** Official refined plan review/progression READY; root accepted final scoped source review/audit dispositions.
- **Required inputs:** Current14 hashes, actual Swift bridge, final targeted/transverse checks and independent review.
- **Action:** Record Step1 Done, Step2 Todo. Preserve passive probes/console behavior, authenticate Unix accepted peer and owned Windows pipe before resource transfer; no process permission mutation.
- **Execution evidence:** Assertion RED101; GREEN22/0/0; clippy/Cargo fmt0; prior unchanged-frontend types/lint/format0. Final scoped PASS; audit exit0 reported0 blocker/2 major/7 minor/2 nit with dispositions, not zero findings.
- **Success criteria:** Planned primitive step accepted; complete ACs remain Partial/Missing. Client authentication/reclamation remainStep5; full macOS settings/new-app/native proofStep7.
- **Failure handling:** Keep foreign/old-binary/second-account proof pending release evidence; whole-AC mapping/certification mandatory later. Supervisor owns history and next-step progression.

Inspect [final summary](proofs/current-run-step1/minimal-final2-summary.json), [scoped review](proofs/current-run-step1/minimal-final2-review.json), [audit](proofs/current-run-step1/audit3.stdout) and [current checkpoint](progress.md).

## 2026-10-13 — Feature: Step 2 pure settings foundation gate completed

- **Type:** Feature
- **Spec modified:** No; minimal scope/IDs unchanged.
- **Code modified:**16 source/contract paths bound by corrective summary/review.
- **AC impacted:** Foundation subsets only; every complete AC/SC remains unproved.
- **Author:** Codex
- **Prerequisite:** Root accepted current corrective scoped review and local gates.
- **Required inputs:** Inventory63/ports51/enums17, explicit public projection, closed DTOs and generated checked schema;16 current hashes.
- **Action:** Record Step2 Done/Step3 Todo. Correct version1/request allowlists/public required keys/enums and genuine validation-failure assertions; keep static domain metadata unregistered.
- **Execution evidence:** Corrective RED101; GREEN21/0/0 with real bridge, strict clippy/full format0; unchanged platform22/frontend gates retained. Repository retry PASS/new0 with unchanged baseline after removing3 new expects. Raw audit0 blocker/7 major/5 minor has explicit correction/false-positive/nonblocking dispositions, not zero findings.
- **Success criteria:** Pure foundation accepted; actual Tauri writersStep4, renderer/live routingStep5, runtime domainsStep6 and full macOS/AC evidenceStep7 remain. RawDocument preservation isStep3; public numeric reads retain intrinsic wire bounds/legacy compatibility.
- **Failure handling:** Do not turn pure port parity into corresponding UI-adapter parity or claim whole-AC/native certification. Supervisor owns next commit/progression; no documenter history action.

Inspect [current summary](proofs/current-run-step2/corrective-summary.json), [scoped review](proofs/current-run-step2/corrective-review.json), [raw audit](proofs/current-run-step2/audit.stdout), [repository receipt](../../conventions/runs/20261013T113238Z/receipt.json) and [checkpoint](progress.md).

## 2026-10-13 — Plan Update: Keep repository adoption atomic with all-writer migration

- **Type:** Spec Update
- **Spec modified:** No; seven steps, FR/AC IDs, business/native/output scope unchanged.
- **Code modified:** None by documenter; pure Step3 worker remains independent.
- **AC impacted:** Planning/evidence ordering only; no runtime loss or completed criterion claimed.
- **Author:** Codex
- **Prerequisite:** Independent source-backed ordering finding confirmed before plan write; root authorized the bounded correction.
- **Required inputs:** Actual full-snapshot write boundary and log/microphone writers, refined Step3/4 plan, current Tauri resolver and already-locked dirs6.
- **Action:** Keep isolated Repository/Locator/normalization in Step3; move running-app adoption/settings-file plugin ownership/autosave/exit-save closure and startup/cache proof to the single Step4 all-writer cutover. No snapshot-to-patch adapter/global provenance machinery. Record direct dirs6 matching current native Tauri resolution alongside fs2.
- **Execution evidence:** Finding is a feasible source interleaving, not observed runtime loss. Initial pure read subset RED0/3 exit101, GREEN3/0 exit0; transaction/step gates remain pending.
- **Success criteria:** Step3 In Progress, app integration/CLI retained; final architecture and seven-step scope unchanged. Official bounded plan review/progression remains supervisor-owned before app activation.
- **Failure handling:** Block app/service adoption until all-writer closure; preserve existing history and complete AC/native evidence requirements.

Inspect [actual ordering finding](proofs/current-run-step3/ordering-finding.json), [refined plan](plan.md#step-3--build-atomic-repository-revision-and-locking) and [initial read GREEN](proofs/current-run-step3/green-reads.stdout).

## 2026-10-13 — Documentation: Step 3 blocked after platform refusal during audit corrections

- **Type:** Spec Update
- **Spec modified:** No; no plan/source edits.
- **Code modified:** None by the documenter; supervisor verifies final source state separately.
- **AC impacted:** No complete criterion certified; feature remains undelivered.
- **Author:** Codex
- **Prerequisite:** Final Step 3 audit corrections were required despite historical local/scoped review passes.
- **Required inputs:** Supervisor-reported historical21/0/1, final audit1 blocker/3 major/11 minor/1 nit and exact platform refusal prefix.
- **Action:** Mark Step 3 Blocked, Steps 4–7 Todo; preserve all proof/history and stop the affected corrective action.
- **Execution evidence:** Historical scoped PASS does not close the final audit. `input_too_large` mitigation is no longer blocking; supervisor instead received `This content was flagged for possible cybersecurity risk...` and interrupted corrective workers/reviewer.
- **Success criteria:** Not met; no corrected version or full feature certified. Verified prior commits:Step1 `50fdd0446809211a73d970c56df9750ea7272fa8`, Step2 `0a7a433`, documentary refinement `e3bc314`.
- **Failure handling:** No source analysis/correction, retry, alternate agent/tool, Git/history or phase archive by documenter. Supervisor owns final state verification and its BLOCKED archive.

Inspect [the authoritative stop](progress.md#current-step-3-stop--platform-refusal-blocks-required-corrections) and existing [execution history](logs/2026-10-13.md); historical proofs remain preserved.

## 2026-10-13 — Feature: Step 3 audit corrections applied, pending independent audit

- **Type:** Bugfix
- **Spec modified:** No.
- **Code modified:** settings_service atomic.rs, locator.rs, store.rs, transaction.rs, raw_merge.rs (new), ownership.rs, instance_probe.rs, mod.rs; settings.rs (docs); tests/settings_transactions (substitution.rs new, faults.rs, ownership.rs, driver.rs). Committed with the rest of Step 3 as `1ae5d84`.
- **AC impacted:** AC-004 (partial, isolated repository only); no AC certified.
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** Fixed the ABA blocker by detection (parent and read-store identities rechecked before replacement), the symlink-alias, temp-name and array-merge majors, 8 minors and the nit; dispositioned 3 Windows function-size minors.
- **Execution evidence:** RED 4 integration failures plus temp-collision emulation; GREEN full suite 356 passed/0 failed/17 ignored; strict clippy on settings targets, fmt, lint, format check and repository conventions PASS; `--all-targets -D warnings` blocked only by 5 warnings identical at HEAD.

Inspect [the Step 3 checkpoint](progress.md#current-step-3-checkpoint--audit-corrections-applied-pending-independent-audit) and [audit dispositions](implementation.md#step-3-audit-dispositions).

## 2026-10-13 — Feature: Step 5 early settings CLI and private channel to the running instance

- **Type:** Feature
- **Spec modified:** No (contract matrix updated for the device default representation).
- **Code modified:** cli.rs, main.rs, lib.rs, settings_service/{cli,render,operations,route,ipc,ipc_tests,endpoint_unix,endpoint_windows,live,tauri_adapter,transaction,validation,ownership,request,mod}.rs, platform_unix socket/facade/tests, commands/audio.rs, shortcut/mod.rs (`change_binding` 27 lines), settingsStore.ts, 3 device selectors, bindings.ts; tests settings_cli (contract, offline, routed), settings_platform, settings_live, settings_validation.
- **AC impacted:** AC-001, AC-002, AC-003, AC-004, AC-005, AC-007, AC-008 (partial; no AC certified).
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** `handy settings get|keys|set` runs before any GUI setup with one renderer and stable exits; offline routing under the shared lock and validation; authenticated private channel to the running instance with a 5 s deadline and unknown outcome; lease-proven endpoint reclamation and removal at exit. Device selectors store null for the system default on every surface; "default" is a literal device name.
- **Execution evidence:** Full suite 429 passed/0 failed/17 ignored; strict clippy, fmt, lint, format check, tsc, frontend tests, Windows endpoint compile harness exit 0; conventions PASS/new 0. Commits `49b2736`, `b1839f4`, `1b34412`, `592b7e0`, `be94db8`.

Inspect [the Step 5 checkpoint](progress.md#current-step-5-checkpoint--early-cli-and-private-channel) and [the implementation notes](implementation.md#step-5-early-cli-and-private-channel).

## 2026-10-13 — Fix: Step 4 independent audit corrections (A and B) and Linux probe decision

- **Type:** Fix
- **Spec modified:** No.
- **Code modified:** settings_service/{sequence (new), sequence_tests (new), sequence_race_tests (new), ui_value (new), live, live_tests, tauri_adapter, effects, edit, errors, command_error, store, locator, transaction, instance_probe, mod}.rs; shortcut/{mod, binding_change (new), binding_change_tests (new)}.rs; commands/{mod, audio, history, models, transcription}.rs; managers/audio.rs; lib.rs; portable.rs; Cargo.toml/lock and capabilities (store plugin removed); settingsStore.ts, settings-errors.ts, Slider.tsx, AccelerationSelector.tsx, useSettings.ts, bindings.ts, 26 locales; tests settings_live, settings_writers, settings_platform, settings-store.test.ts (new).
- **AC impacted:** AC-003, AC-004, AC-005 (partial; no AC certified).
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** ordered UI and routed CLI writes (preparation, commit, effects, event); service installed before the first startup reader; read-only startup instead of a panic; P0 tests; minors and simple nits; Linux free session bus plus free lease allows the offline write.
- **Execution evidence:** RED 3 suites; GREEN targeted Rust and frontend tests, strict clippy, fmt, lint, format check, tsc, translations; conventions PASS/new 0. Commit `bc76443`.

Inspect [the correction checkpoint](progress.md#current-step-4-correction-checkpoint--independent-audits-a-and-b) and [the dispositions](implementation.md#step-4-audit-corrections).

## 2026-10-13 — Feature: Step 6 domain framework and output domain

- **Type:** Feature
- **Spec modified:** No (contract matrix and schema updated: domain `arguments`, `details.offset`).
- **Code modified:** settings_service/{domains/mod, domains/output, domains/fixture_tests, input}.rs (new), operations, request, protocol, errors, edit, render, route, live, sequence, tauri_adapter, mod; cli.rs; settings.rs; tests settings_domains (new), settings_validation; README.md, AGENTS.md, bindings.ts.
- **AC impacted:** AC-006, AC-007, AC-008 (partial; no AC certified).
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** domain framework (registry, aliases, dry run, atomic commit, live-only refusal, bounded UTF-8 file/stdin input) and the `output` domain; README CLI documentation with recovery rules.
- **Execution evidence:** targeted Rust tests (lib 57, settings targets incl. domains 8) and frontend checks exit 0; strict clippy; conventions PASS/new 0. Commit `462a5a5`.

Inspect [the Step 6 checkpoint](progress.md#current-step-6-checkpoint--domain-framework-and-output-domain).

## 2026-10-13 — Feature: Step 7 macOS native proof (debug build) and acceptance coverage

- **Type:** Feature
- **Spec modified:** No.
- **Code modified:** scripts/settings-native-proof.ts and scripts/macos-window-list.swift (new); no application code.
- **AC impacted:** AC-001, AC-005 (native macOS evidence, partial); no AC certified complete.
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** native harness on macOS 26.5.2 / M4 Pro with a debug proof binary and an isolated HOME: older Handy refusal, offline and live reads/writes, start-hidden activation policy, routed live effect, no app registration by the CLI, endpoint reclamation, read-only startup on a linked store; full `cargo test`.
- **Execution evidence:** harness 18/18 checks, full suite 474 passed/0 failed/17 ignored, all gates exit 0, conventions PASS/new 0. Pending release evidence: M1 release latency, settings window refresh and shortcut re-registration (human), normal-quit endpoint removal, Windows/Linux/second account.

Inspect [the Step 7 checkpoint](progress.md#current-step-7-checkpoint--macos-native-proof-debug-build).

## 2026-10-13 — Fix: final delivery review, correction round 1

- **Type:** Fix
- **Spec modified:** No (the plan's Windows console-subsystem decision is superseded by the GUI subsystem plus `AttachConsole`, recorded in implementation.md and progress.md).
- **Code modified:** settings_service/{instance_probe, platform_windows, endpoint_windows, endpoint_unix, ipc, ipc_tests, route, sequence, sequence_tests, sequence_race_tests, tauri_adapter, console_windows, input, mod, platform_unix, platform_unix/socket, platform_unix/tests, domains/fixture_tests}.rs; new ipc_server.rs, ipc_server_tests.rs, pipe_server_windows.rs, native_changes.rs, sequence_parity_tests.rs, domains/fixture_stdin_tests.rs; main.rs, lib.rs (doc comments), shortcut/mod.rs, commands/audio.rs; tests settings_platform/{windows, unix, cli_console}.rs; README.md, AGENTS.md.
- **AC impacted:** AC-001, AC-003, AC-005, AC-006, AC-008 (partial; no AC certified).
- **Author:** Claude Code (implementation agent, orchestrator-supervised)
- **Action:** Windows offline write under decision 008; endpoint thread without busy loop, with a bounded close and republication of a lost name; Windows pipe server with several instances and I/O deadlines; GUI subsystem restored with terminal attachment for command-line runs; oversized requests refused before sending; routed VAD/channel writes prepared like the UI; stdin tests; stale docs fixed; follow-ups recorded.
- **Execution evidence:** RED parity on the previous routed path; full `cargo test` 490 passed/0 failed/18 ignored; strict clippy, fmt, lint, format check, tsc exit 0; Windows compile harness and clippy exit 0; conventions PASS/new 0. Commit `853bfeb`.

Inspect [the round 1 checkpoint](progress.md#current-delivery-review-checkpoint--correction-round-1) and [the dispositions](implementation.md#delivery-review-correction-round-1).
