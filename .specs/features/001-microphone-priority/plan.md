---
feature: microphone-priority
spec_ref: spec.md
title: Microphone Priority Technical Plan
status: Planned
created: 2026-10-12
updated: 2026-10-14
penflow_verification_policy:
  version: 1
  mode: livespec
  generated_docs: required
  native_geometry: required
  homologous_references: required
  native_export: required
---

# Implementation Plan: Microphone Priority

## Summary

Extend 008's settings transaction/domain service with persistent microphone priority; resolve audio input with bounded workers and session generations, keep intent separate from availability/effective input, and replace the single selector with an accessible draft editor.

Inspect [functional source](spec.md) for FR-001–FR-008, AC-001–AC-008 and SC-001–SC-003. Inspect [008's implementation record](../008-settings-cli/implementation.md) and the merged `src-tauri/src/settings_service/` for the exact shared names; [008's plan](../008-settings-cli/plan.md) is historical context, and names it announced but never shipped (`commit_mutation`, `LiveSettingsEffects`) are not used. Scope **L**: eight FR, shared audio/settings lifecycle, migration and cross-platform native failure handling. 008 is merged in `dev` (`e9f96df`). `microphones list` also depends on **step 0 of 006**, which adds live domain operations (D2) and typed error details (D3) to the 008 framework; the exact names are recorded in `.specs/ship.md` (« Contrat D2/D3 »). This command produces documentation only.

## Technical Context

| Aspect | Choice | Reason / observed source |
|---|---|---|
| Backend | Rust edition 2021, Tauri 2.11.5, existing Tokio | Existing managers, commands, Cargo manifest |
| Audio | Existing cpal, AudioRecorder, VAD, resampling | Preserve sample-processing pipeline, channel policy and readiness handshake |
| Settings | Merged 008: `SettingsRepository::read_snapshot`, `SettingsSnapshot`, `Revision`, `SettingsError`, `domains::{DomainChange, ChangeKind::Apply}`, `SettingsBackend::commit_change` (`sequence::write_change` live, `commit_edit_with_probe` offline), `tauri_adapter` for UI writes, `effects.rs` live effects | One locked fresh-read commit path and the existing durable revision/lock |
| CLI | 008 `Domain` trait and `domains::production()`, `crate::cli` clap types, `render.rs`; 006 step 0 `DomainPlan::Live`/`LiveOperation` (D2) and `ErrorReason`/`ErrorItem` (D3) | Add `microphones` registration; reuse local IPC/offline routing |
| UI | React 18, TypeScript, Zustand, i18next | Existing general audio settings; typed Tauri commands/events |
| Storage | Existing JSON settings payload; `microphone_priority: Vec<String>` | No new database, HTTP API, cloud resources or credentials |
| Tests | Cargo unit/integration, Bun, Playwright, native hardware procedure | Pure policy versus real capture/file/process boundaries |

Inspect [manager](../../../src-tauri/src/managers/audio.rs), [recorder](../../../src-tauri/src/audio_toolkit/audio/recorder.rs), [enumeration](../../../src-tauri/src/audio_toolkit/audio/device.rs), [audio commands](../../../src-tauri/src/commands/audio.rs), [actions](../../../src-tauri/src/actions.rs) and [selector](../../../src/components/settings/MicrophoneSelector.tsx) for implementation context. Preserve existing filenames; new Rust module filenames use Rust snake_case, new Markdown/TS files kebab-case. Repository Bun/ESLint/Prettier overrides generic Commander/Biome/Vitest guidance.

## Constitution Check

| Principle | Gate / decision |
|---|---|
| Existing native/frontend boundary | PASS: native validates/persists/resolves; UI owns draft, focus and translated presentation |
| Observed stack/manager choices | PASS: reuse audio manager/recorder and 008 service; no parallel settings repository or IPC protocol |
| Tool availability versus runtime evidence | PASS: tools checked below; no recording/build/test success inferred |
| Specify before implementation and meaningful tests | PASS: every FR/AC mapped; real capture-loss and store fault boundaries required |
| Simple responsibilities and injectable I/O | PASS: pure policy, catalog, opener, session owner, thin UI/CLI adapters |
| Compatibility and privacy | PASS: retain clamshell and legacy raw key; no new log payload containing audio, names or secrets |

## Requirement Inventory

| Requirement | Acceptance / measure | Steps |
|---|---|---|
| FR-001 | AC-001/007; ordered unique draft, pointer/keyboard, restart | 2, 6, 7 |
| FR-002 | AC-002/003; four availability states, effective reason, timed refresh | 1, 3, 5, 6, 7 |
| FR-003 | AC-003/004; order, default, 2 s/6 s, no preference erasure | 1, 3, 4, 7 |
| FR-004 | AC-004/007; pending revision, reconnect, preserved audio prefix | 1, 4, 7 |
| FR-005 | AC-005; override plus closed lid, then default | 3, 4, 7 |
| FR-006 | AC-006; absent-key migration, explicit empty wins, recoverability | 2, 7 |
| FR-007 | AC-001/007/008; atomic writes, CAS, shared events/cache | 2, 4, 5, 6, 7 |
| FR-008 | AC-008; CLI list/set-priority and exact 008 envelope/exits | 5, 7 |

SC-001 requires all eight AC plus native/UI gates; Story 4's P2 label does not relax the release gate. SC-002 asserts exact configured order and unrelated settings across each failure fixture. SC-003 requires keyboard focus/error recovery and separate design/runtime assertions.

## Architecture Decisions

### Settings and migration

Use the merged 008 service. New stored key `microphone_priority: Vec<String>` gets a reviewed descriptor row in `descriptor_data.rs` (`Array`, owner `microphones`, `RuntimeEffect::AudioInput`) and a projection in `PublicSettings` (`public.rs`). Every write commits a `DomainChange(ChangeKind::Apply(..))` whose edit sets the complete validated list on the locked latest settings: the CLI through `SettingsBackend::commit_change` (`sequence::write_change` in the running instance, `SettingsRepository::commit_edit_with_probe` offline), the settings UI with its base revision through `tauri_adapter::write_change(app, &DomainChange, expected)`, a thin wrapper over the existing `sequence::write_change` added by whichever of 001 and 003 ships first (003 plans the same wrapper). Do not create another lock, revision counter, file writer, settings channel or envelope. The whole ordered list is one edit of the latest snapshot; unrelated simultaneous writes survive. `effects.rs` maps `microphone_priority` to the microphone live effect (`LiveEffect::MicrophoneDevice` or a dedicated variant), which invalidates the next-open resolution cache and queues the always-on reopen of [Recording boundary](#recording-boundary-always-on-and-loss); the effect runs after the durable commit and before the `settings-changed` event, as for every 008 effect.

Generic `set` refusals: `validation::editable_descriptor` already refuses list keys with a typed reason per owner domain; add the `"microphones" => ValidationReason::MicrophonesDomain` arm (message naming `handy settings microphones set-priority`, translated by the UI) and attach D3 `details.reason = "microphones_domain"` (new `ErrorReason::MicrophonesDomain`, through `SettingsError::with_reason`). After migration `selected_microphone` stays stored for downgrade but no surface writes it: its descriptor row becomes `read_only: true`, so `handy settings keys` reports it read-only and `get` still shows the stored value, and the read-only branch of `editable_descriptor` answers a read-only key owned by `microphones` with the same `MicrophonesDomain` refusal (`validation_failed`, exit 2, `details.field = "selected_microphone"`, `details.reason = "microphones_domain"`) instead of the generic "changed in the settings UI" message, which would be false because the UI no longer writes that key either. This is the solution that fits the merged 008 code: removing the descriptor row would break 008's rule that every stored key is classified (the descriptor list also drives the raw-merge preservation of stored fields), and keeping it writable would accept a value nothing reads. The legacy UI command `set_selected_microphone` and its `tauri_adapter::write` call are removed in Step 5; `RuntimeEffect::AudioInput` of `selected_microphone` moves to `microphone_priority` (the `effects.rs` test that every declared runtime effect is applied or read at use stays green).

Detect key presence in raw JSON before serde defaults. Absent `microphone_priority`: a **null** legacy `selected_microphone` (008 stores the system default as null) becomes `[]`; any non-null value becomes `[exact_name]`, including the string `"default"`, which 008 treats as an ordinary device name (a real ALSA PCM on Linux), never as a sentinel (spec Clarification of AC-006, 2026-10-14). Present new list, including `[]`, wins. Preserve the raw legacy key unchanged and ignore it for normal resolution once the new key exists. Keep clamshell/channel/unknown fields. The rule lives in `settings::apply_settings_migrations(&mut settings, payload)`, which 008's `normalization::normalize` calls with the raw payload, so key presence is detected before serde defaults. The running instance persists it once through 008's startup normalization commit (`SettingsRepository::initialize`: startup lease, settings lock, atomic replacement that keeps the original bytes on failure); failure leaves the original readable and reports an actionable persistence error. Headless CLI reads normalize in memory and never write the migration. Test second startup no-op, unavailable/ambiguous legacy names, the literal `"default"` name and failed flush/replace.

Shared validation accepts exact, non-empty, non-whitespace-only unique names regardless of current presence; preserve case and meaningful bytes. Presence is runtime status, never a persistence validator. UI Add lists the real catalog (including duplicate-name groups labelled ambiguous); a grouped ambiguous name may be saved once but never opens an arbitrary device. The synthetic system-default option is not a priority entry: empty list represents fallback. A real device literally named `default` or `Default` is an exact name, in migration and in new-list validation alike; there is no sentinel. Removed unavailable entries remain removable. Duplicate/empty input returns `validation_failed` without a partial commit; with D3, `details.items` lists each refused name as `ErrorItem::new(index, code)` (0-based position in the submitted order, new `ErrorReason` variants `microphone_name_empty` and `microphone_name_duplicate`, fixed messages that never echo the name). No invented list-count cap; 008's bounded request size remains authoritative.

### Resolution and deadline allocation

Inject `AudioDeviceCatalog`, `StreamOpener`, monotonic `Clock/Deadline`, `LidStateProbe` and `CaptureSessionPort`. A catalog snapshot records observation/generation, name groups and transient handles; indices never persist. Successful complete enumeration may mark missing names unavailable; failure or incomplete name discovery marks affected status unknown; duplicates are ambiguous. Failed probes cannot masquerade as absence. UI refresh is non-invasive enumeration only, not opening candidate devices. One in-flight refresh per view; open/5-second cadence/Retry trigger fresh observations, late replies are fenced and unmount cancels publication.

Normal resolution uses the committed revision captured at session start and a fresh catalog at every recording start, including always-on. Device-change notifications may invalidate cache but cannot replace the next-start fresh check needed for reconnection. Cache transient handles/config only within a catalog generation. Closed lid **and configured override** selects override then default, excluding normal list. Open/unknown lid, or no override, uses normal priority. Channel mapping uses saved channel only if valid for actual device; otherwise existing default mapping without changing settings.

Budget rule (orchestrator decision 2026-10-14, AC-003): **every open attempt gets its full 2-second timeout; the 6-second total limits how many attempts start and never shortens one.** A single monotonic 6-second resolution deadline covers the lid probe, enumeration, the one fresh re-enumeration on stale-cache/enumeration failure and the open/readiness attempts. A configured attempt (or the closed-lid override attempt) starts only while at least 4 seconds remain: its own 2 seconds plus 2 seconds reserved for the system-default attempt. Enumeration, the lid probe and the re-enumeration are bounded so that they end by t = 4 s at the latest; a probe still running then counts as failed (status unknown). When fewer than 4 seconds remain, the configured choices not yet attempted are recorded as failed attempts with reason `deadline` (their open is never called, preferences unchanged), and the system-default attempt starts no later than t = 4 s with its full 2 seconds, so it always runs after configured attempts fail, as AC-003 requires. Presence/ambiguity failures consume no open call. Consequence: at most two configured attempts can run to their full timeout before the default; faster failures leave room for more choices. Re-enumerate at most once, never reopen an already attempted name. Four-plus slow devices and slow enumeration must prove termination, full 2-second attempts and the default slot rather than silently extend the deadline. If the native spike shows that the default cannot be invoked and cancelled inside this budget on macOS, AC-003 is a release blocker, not a claimed guarantee. Start latency samples are collected separately for on-demand/always-on; no unsupported additional user-facing latency target is invented.

### Worker ownership and readiness

Existing `AudioRecorder::open` waits on unbounded `init_rx.recv()` and can join a stalled worker. Refactor attempt initialization into an owned attempt handle with generation, deadline/cancellation flag and stop/drain channel. Coordinator never joins a potentially blocked opener on the main loop or while holding settings/manager state locks. A timed-out generation cannot publish a device, cache config, capture-ready event or samples; a late native stream is immediately dropped by its owner. Track/reap abandoned workers; cap outstanding native opens so repeated starts cannot create unlimited detached threads. Reaching the cap reports a recording error, never successful capture. Step 1 must prove cleanup and device release on macOS (CoreAudio); the same proof on the Windows (WASAPI) and Linux (ALSA/PulseAudio) backends is pending release evidence under the constitution, while their code builds and passes the automated tests. A `spawn_blocking` timeout alone proves only caller latency. If cpal cannot bound native ownership cleanup, stop the spike and propose an isolated cancellable native worker design for review before production; never assume Rust can kill a blocked thread.

`Stream::play()` is not recording readiness. Preserve existing first-sample handshake and capture generation fencing; indicator stays arming until the actual stream/capture is ready. Do not buffer/replay audio spoken before readiness. Effective input is published only from a successful owned stream; resolve actual default name from that opened handle rather than infer it from a synthetic label.

### Recording boundary, always-on and loss

A session serializes start/stop/error/priority-effect intents in the audio coordinator. Snapshot committed revision/order/lid at start; no settings lock spans enumeration/open/capture. A priority commit invalidates next-open cache and queues a revision-tagged effect. Idle always-on re-resolves/reopens; active recording continues on its chosen stream, then applies latest pending revision after completion. A successful transaction means settings are committed and this boundary effect is accepted; post-commit native failure uses 008's truthful `internal_error` with committed outcome/revision and reconciliation, not rollback or false validation failure. No-op commit does not increment revision or needlessly reopen. Always-on recording start re-resolves; same device retains healthy stream, changed device reopens before capture. Idle stream-loss recovery is one bounded resolution per event, no self-triggered unbounded loop; later user start/real device event may retry.

On stream error during capture, seal the current generation's captured prefix, end it exactly once, report translated cut-short notification and send that prefix through the existing normal stop/transcription path. Do not use cancel-generation or splice samples from fallback device. Add a loss notification/output from the recorder's error path and a recoverable capture-buffer drain even if the consumer dies; `rec.stop()` returning error→empty today is insufficient. Preserve VAD/channel/resampling decisions. Ordinary stop and loss racing share one completion token; trailing-buffer delay is skipped on lost input. Push-to-talk release after automatic end consumes its ended binding and is a no-op; a later independent toggle starts a new session. Cancellation explicitly requested by the user retains existing cancel semantics.

Keep current effective stream while always-on is open; otherwise display last **recording** device since launch, marked last-used, or not-used-yet after restart. Idle stream observations never overwrite last-recording state. Reasons are first-choice, fallback, clamshell, system-default; availability is independent. Samples retained/transcribed are not a durable-history promise: existing actions saves history only after verified WAV save. Test WAV failure separately: transcription still runs, missing history is surfaced honestly. F1 does not redesign history, clipboard, destination, AX, activation, LLM or insertion. Native loss completion must not require the future 010 bubble.

### UI and CLI operations

Keep the public names in AC-008: `handy settings microphones list` and `handy settings microphones set-priority <name>...`. New `settings_service/domains/microphones.rs` implements the 008 `Domain` trait with canonical actions `list` and `set-priority`, registered in `domains::production()`. Zero names sets the valid empty list; quote names with spaces, use clap's `--` separator for hyphen-leading names. Reuse 008 expected-revision argument/JSON placements, help and renderer.

- **`set-priority`** is decided by the pure `Domain::plan`: validate the names (D3 `details.items` on refusal), then `DomainPlan::Change { change: DomainChange(ChangeKind::Apply(..)), keys: vec!["microphone_priority"] }`. It works offline (`commit_edit_with_probe`) and live (`sequence::write_change`, then the microphone live effect), with `--dry-run` previewing through the same edit.
- **`list`** is a **live operation of 006 step 0 (D2)**: `plan` returns `DomainPlan::Live(Box::new(MicrophonesList))`, a `domains::live_operation::LiveOperation` whose `run(context, call)` reads the configured order and revision with `LiveContext::read`, reaches the running instance's audio manager with `managed_state::<Arc<AudioRecordingManager>>(context)`, takes a fresh bounded catalog observation on a worker (never the UI thread, well inside 008's 5-second answer deadline) and the current or last effective input, and returns `LiveAnswer::Read(data)`. Without a running Handy, `domains::execute` returns the 008 live-only refusal unchanged (`unavailable`, exit 5, empty details): the CLI process never enumerates devices itself and never fabricates availability or an effective device. The configured order stays readable offline with `handy settings get microphone_priority` (spec Clarification, 2026-10-14). The list reply pairs a revision-consistent configured snapshot with a separately timestamped audio observation; these are explicitly distinct clocks, not an atomic promise about changing hardware. A failed enumeration inside the live operation yields unknown statuses with a typed warning in data, never a destructive write.

List data: `configured_order`, `availability` entries `{name,status}`, `availability_observed_at`, `effective_input` `{name,reason,scope,capture_generation}` or null, and optional typed enumeration warning. Mutation data returns committed `microphone_priority` (the configured order); 008 envelope carries the authoritative revision. `scope` is current/last-recording/not-used-yet (no offline scope: offline, `list` is refused). Exact wire enums and nullable fields are locked in domain DTO tests, not localized. Reuse success 0; persistence_failed/internal_error 1; usage_error/validation_failed 2; revision_conflict 3 with current_revision; not_found 4 where applicable; unavailable/unknown-outcome 5, including the offline live-only refusal of `list`. A failed catalog projection may be successful data with unknown statuses; a failed transport/service read uses 008 errors. No credentials or raw AppSettings serialization. No second JSON line or diagnostics on stdout, no MCP server, no OpenAPI because no HTTP endpoint is introduced.

UI keeps `draft_order`, base Revision and dirty/error/busy/conflict state. Save submits the complete draft as the same `DomainChange` through `tauri_adapter::write_change` with the base revision (same validator and transaction as the CLI); busy prevents double dispatch. The settings UI reads availability and the effective input through a Tauri command over the same audio-manager projection that `MicrophonesList` uses, so both surfaces report the same data. Stale revision retains draft and offers Reload to adopt current committed order/discard edits. A settings-changed event advances committed data but never overwrites a dirty draft; older async reads cannot overwrite newer revisions. Reset empties draft only (translated explanation that Save is required), not clamshell; retain spec label rather than silently invent a renamed action. Pointer dragging and Move up/down have identical draft transitions; stable per-name row identity preserves focus after reorder/remove/retry. Availability text adds information to color; loading/error/success/dragging/drop-target/empty/invalid/valid/enabled/disabled/focus are testable states.

## Design Reference

| Screen | Component breakdown | Reference / stage |
|---|---|---|
| microphone-settings | Existing settings container, priority rows, Add catalog control, Move up/down/remove, pointer target, Save/Reset/Reload, status/retry, effective-input panel, clamshell explanation | Approved spec IDs; PNG and theme reference generated in the common pass before spec-ship |

No theme.css/theme.md is currently present. Retain current tokens/components; if the common pass creates a theme, install/import it once with existing-output guard before UI implementation. No mockups or shared Penflow edits in this command.

## Penflow Contract Inputs

- Planned `flow_id: microphone-priority`, `screen_id: microphone-settings` are authoritative from the approved decision.
- Inspect [current semantic tree](../../../penflow/semantic-ui-tree.json) as incomplete context; its existing IDs are not newly approved by this plan.
- Primary future code input: `penflow/code-ir.json`; absent today, as is `expected-ui-tree.json`; ui.pen desktop geometry is incomplete. Inspection reported BLOCKED/certified false. No code IR or design PASS is claimed.
- User decision overrides immediate forward-chain/mockup generation: the program owner produces all flow/screen contracts, semantic/test IDs and Pencil mockups once before spec-ship. This plan retains required policy procedures above; required does not mean performed. Bind all interactive/observable states to semantic/test IDs then; don't freeze placeholder selectors or regenerate approved IDs as a repair.
- Before UI code: forward validation/export/code IR, generated documents, native geometry/export and homologous visual references; cumulative FR/AC predicates, bound review-snapshot/raw review-result and plan-review approval; design profile C51 PASS/certified true plus registry/mockup gates. Runtime code/build evidence is a separate later gate. Missing common-pass artifacts block UI implementation/release, not permission to fabricate this preparation.

## Gherkin Scenarios and Sequence Diagrams

```gherkin
Feature: One shared microphone settings transaction
  Scenario: Conflict retains user draft
    Given UI base revision is 8 and CLI has committed revision 9
    When UI saves its changed list with expected revision 8
    Then revision_conflict returns current revision 9 without mutation
    And the dirty draft remains until Reload is activated
  Scenario: Commit during capture
    Given a recording owns device A at revision 9
    When CLI commits device B first through the shared service
    Then the whole order is durable at revision 10
    And A continues until this session ends
    And the next opening resolves revision 10
```

```mermaid
sequenceDiagram
  participant UI as Draft editor
  participant CLI as microphones Domain (set-priority)
  participant Service as 008 sequence::write_change / commit_edit_with_probe
  participant Disk as Locked fresh repository
  participant Audio as Audio coordinator (effects.rs)
  UI->>Service: DomainChange via tauri_adapter::write_change, expected Revision
  CLI->>Service: same DomainChange via SettingsBackend::commit_change
  Service->>Disk: lock and reread current settings
  alt invalid or stale revision
    Service-->>UI: typed error, keep draft
    Service-->>CLI: 008 error and exit
  else valid changed order
    Service->>Disk: durable complete commit, revision plus 1
    Service->>Audio: accept next-boundary effect
    Audio->>Audio: invalidate next resolution, queue if recording
    Service-->>UI: committed revision and settings-changed
    Service-->>CLI: single success envelope
  end
```

```gherkin
Feature: Microphone list through the running instance
  Scenario: Live list
    Given Handy is running
    When the user runs "handy settings microphones list --json"
    Then the running instance answers with the configured order, each choice's availability and the effective device with its reason
  Scenario: No running instance
    Given Handy is not running
    When the user runs "handy settings microphones list --json"
    Then the exit code is 5 with code "unavailable" and empty details
    And "handy settings get microphone_priority --json" still returns the configured order
  Scenario: Legacy key refused
    Given the settings were migrated to a priority list
    When the user runs "handy settings set selected_microphone USB --json"
    Then the exit code is 2 with code "validation_failed" and details.reason "microphones_domain"
    And nothing is written
```

```mermaid
sequenceDiagram
  participant CLI as handy settings microphones list
  participant Route as 008 route
  participant Instance as Running Handy (RoutedBackend)
  participant Op as MicrophonesList LiveOperation
  participant Audio as AudioRecordingManager and catalog worker
  CLI->>Route: Domain operation
  alt instance endpoint answers
    Route->>Instance: private channel request
    Instance->>Op: run(LiveContext, LiveCall)
    Op->>Op: LiveContext::read (configured order, revision)
    Op->>Audio: bounded fresh catalog and effective input
    Audio-->>Op: statuses (unknown on failure) and effective input
    Op-->>Instance: LiveAnswer::Read(list data)
    Instance-->>CLI: success envelope, exit 0
  else no running instance
    Route-->>CLI: unavailable, exit 5 (008 live-only refusal)
  end
```

```gherkin
Feature: Bounded input ownership
  Scenario: Fallback and late opener
    Given A then B is saved and A's opener times out
    When B becomes ready inside the global deadline
    Then B supplies samples and A remains saved
    And a late completion for A is disposed without publishing readiness
  Scenario: Lost input completes once
    Given a session has captured a prefix from B
    When B disconnects while a user stop also arrives
    Then the prefix is transcribed exactly once
    And no fallback microphone is spliced into this recording
```

```mermaid
sequenceDiagram
  participant Start as Recording intent
  participant Policy as Resolution coordinator
  participant Catalog as AudioDeviceCatalog and lid probe
  participant Opener as Owned StreamOpener
  participant Capture as Capture generation
  participant Action as Normal transcription action
  Start->>Policy: committed snapshot, new generation, deadline
  Policy->>Catalog: fresh enumeration and lid state
  loop ordered candidates once within normal budget
    Policy->>Opener: bounded open attempt
    alt missing, ambiguous, failed or expired
      Opener-->>Policy: typed failure; fence late result
    else ready stream
      Opener-->>Policy: actual device handle
      Policy->>Capture: start, await first-sample readiness
    end
  end
  alt configured attempts fail
    Policy->>Opener: one default attempt inside 6 seconds
    Opener-->>Policy: ready or recording error
  end
  Capture-->>Start: current effective input and capture-ready
  Capture->>Action: sealed prefix on stream loss, one completion token
  Action->>Action: existing WAV and transcription pipeline
```

## State Model

```gherkin
Feature: Session generation and pending edits
  Scenario: Pending always-on edit
    Given a recording is active and priority changes
    When its current generation finishes
    Then only the latest pending committed revision is resolved
    And reopening does not discard the completed samples
  Scenario: Settings draft states
    Given a dirty valid draft
    When Save fails or conflicts
    Then draft and focus remain recoverable
    When Reload is selected after conflict
    Then current committed order replaces the draft
```

```mermaid
stateDiagram-v2
  [*] --> Idle
  Idle --> Resolving: launch always-on or start or idle committed edit
  Resolving --> Arming: owned stream opened
  Resolving --> Error: deadline or no device
  Arming --> Capturing: actual sample readiness
  Arming --> Idle: cancelled generation
  Capturing --> Capturing: commit queues next revision
  Capturing --> Completing: normal stop or loss seals prefix once
  Completing --> Idle: transcription dispatched
  Idle --> Resolving: pending always-on revision or next start
  Error --> Resolving: later user start or real device change
  note right of Capturing: Never switch input inside one recording
  note right of Resolving: Late generations cannot publish or capture
```

## Data Model

Logical entities in existing JSON plus runtime memory; no SQL table or migration infrastructure.

```mermaid
erDiagram
  SETTINGS_SNAPSHOT ||--o{ PRIORITY_ENTRY : persists_order
  PRIORITY_ENTRY ||--o| AVAILABILITY : observed_by_name
  SETTINGS_SNAPSHOT ||--o{ CAPTURE_SESSION : captured_revision
  CAPTURE_SESSION ||--o| EFFECTIVE_INPUT : owns
  SETTINGS_SNAPSHOT {
    integer revision PK
    string clamshell_microphone
    string legacy_selected_microphone
  }
  PRIORITY_ENTRY {
    string exact_name PK
    integer position
  }
  AVAILABILITY {
    string exact_name PK
    string status
    integer catalog_generation
  }
  CAPTURE_SESSION {
    integer capture_generation PK
    integer settings_revision FK
    string lifecycle
  }
  EFFECTIVE_INPUT {
    integer capture_generation FK
    string actual_name
    string reason
    string scope
  }
```

## Implementation Plan

### Step 1 — Spike bounded native ownership and capture loss

**Prerequisite:** 008 merged (`e9f96df`); current spec/progression plan and independent plan review ready before code.
**Required inputs:** Existing blocking recorder handshake, cpal backends, readiness generation, test hardware and isolated native harness.
**Action:** Plan new `src-tauri/tests/microphone_native.rs` and `scripts/microphone-native-proof.ts`; modify recorder only during implementation to inject attempt/loss ports. Prove timeout fencing, bounded outstanding workers, late stream drop/device release, prefix drain after worker death and enum-while-open behavior on macOS (CoreAudio). Capture fresh-start latency and cleanup observations. The same spike on Windows (WASAPI) and Linux (ALSA/PulseAudio) is recorded as **pending release evidence** with the exact command (`bun scripts/microphone-native-proof.ts --binary <absolute-release-binary> --platform windows|linux --out <proof-directory>`); it never blocks this step or the next ones.
**FR covered:** FR-002.1: Native enumeration evidence, FR-003.1: Deadline and cleanup spike, FR-004.1: Loss prefix and completion spike
**Execution evidence:** Binary/OS/device-class metadata, monotonic timings, handle/worker counters, native cut-short transcript and captured-prefix hash.
**Success criteria:** On macOS: every open attempt bounded by its full 2 s, global ≤6 s with the reserved default slot, zero stale publication, bounded native resource ownership, exactly-once prefix completion; backend unknowns explicit.
**Failure handling:** A missing cleanup/prefix guarantee on macOS blocks production audio integration. A Windows/Linux failure observed later is a release finding for that platform, not a gate on these steps. Review a changed native ownership design before coding it; no caller-timeout-only success.

### Step 2 — Extend 008 schema, migration and transaction validation

**Prerequisite:** 008 merged; Step 1 native feasibility established on macOS.
**Required inputs:** Raw presence-aware settings (`settings::apply_settings_migrations` with the raw payload), `Revision`, `DomainChange`, `SettingsError`, repository fault seam.
**Action:** Modify `src-tauri/src/settings.rs` (field, `apply_settings_migrations` rule) and `settings_service/{descriptor_data.rs,public.rs,validation.rs,errors.rs,effects.rs,tauri_adapter.rs}`: `microphone_priority` row, `selected_microphone` read-only, `ValidationReason::MicrophonesDomain` and `ErrorReason::MicrophonesDomain`/`MicrophoneNameEmpty`/`MicrophoneNameDuplicate` (D3 variants, once 006 step 0 is merged), live effect mapping, `tauri_adapter::write_change` if 003 has not added it. The contract schema (`protocol::contract_schema()` and `features/008-settings-cli/contracts/settings-cli.schema.json`) follows the new key and reasons. New `src-tauri/tests/microphone_settings.rs` uses real temp store plus fault/barrier adapters. Preserve legacy/clamshell/unknown fields; no independent serializer/writer.
**FR covered:** FR-001.1: Persistent ordered list, FR-006.1: Presence-aware recoverable migration, FR-007.1: Shared atomic list validation
**Execution evidence:** Raw before/after generations, idempotence and explicit-empty fixtures, validation parity, failed-write unchanged hashes and revisions.
**Success criteria:** Complete valid order commits once; stale/invalid/failure never partially changes state; migration repeat is no-op and original source survives failures.
**Failure handling:** Any lost intent, changed clamshell, partial persistence or direct writer is P0; keep error/old generation, no blind replay.

### Step 3 — Implement pure ordered resolver and catalog projection

**Prerequisite:** Steps 1–2 pass; input schema stable.
**Required inputs:** AudioDeviceCatalog, Clock/Deadline, LidStateProbe, StreamOpener ports and independent decision cases.
**Action:** New `src-tauri/src/microphones/{mod.rs,policy.rs,catalog.rs,types.rs}` with pure candidate/deadline/status policy and real cpal catalog adapter via `audio_toolkit/audio/device.rs`. Inline unit tests plus `src-tauri/tests/microphone_resolution.rs`; name groups never arbitrary first-match. One re-enumeration, budget allocations/default and normal/clamshell branches explicit.
**FR covered:** FR-002.2: Four-state catalog projection, FR-003.2: Ordered bounded resolution, FR-005.1: Closed-lid override policy
**Execution evidence:** Fake-clock decision traces, call/attempt counts, 0/1/4-plus choice boundaries, unknown/busy/ambiguity/default/lid fixtures.
**Success criteria:** Each eligible name tried once in order, default inside global budget after failures, unknown never unavailable, preferences untouched.
**Failure handling:** Contradiction between deadlines/default/presence stops progression to manager wiring; document failing trace and repair policy without extending spec bounds.

### Step 4 — Integrate stream ownership and recording boundaries

**Prerequisite:** Native spike and pure resolver gates PASS.
**Required inputs:** Recorder worker/error/readiness channels, AudioRecordingManager state, 008 `effects.rs` live effect of `microphone_priority` and actions stop pipeline.
**Action:** Modify `audio_toolkit/audio/recorder.rs`, `managers/audio.rs`, `actions.rs` and necessary shortcut completion dispatch to own attempt generations, drain lost prefix and publish effective state. Remove `persist_default_microphone_after_fallback` erasure and first-match cache path. Adapt `update_selected_device`, start/stop/always-on/lazy-close to queued committed-revision policy, preserve channel/VAD. New `src-tauri/tests/microphone_sessions.rs`; reuse recorder inline tests.
**FR covered:** FR-003.3: Owned opener and ready boundary, FR-004.2: Deferred changes and loss completion, FR-005.2: Clamshell integration, FR-007.2: Commit cache and boundary effects
**Execution evidence:** Race barriers for stop/loss/cancel/commit/late-ready, audio-prefix assertions, always-on startup/edit/reconnect and no-op tests, WAV-failure observation.
**Success criteria:** Exactly one completion per generation; no splicing/discard due to settings change; last recording and current stream distinguished; no main-thread or settings-lock wait on I/O.
**Failure handling:** Duplicate output, lost captured prefix, unbounded reopen or stale readiness is P0. Native effect failure retains truthful committed settings and 008 reconciliation.

### Step 5 — Register microphones domain and typed frontend commands

**Prerequisite:** 008 merged, Steps 2–4 stable and **006 step 0 merged** (D2 `DomainPlan::Live`, `LiveOperation`, `LiveContext`, `managed_state`; D3 `ErrorReason`, `ErrorItem`, `with_items`, `with_reason`), which `microphones list` and the per-name refusals need.
**Required inputs:** `Domain` trait, `SettingsOperation::Domain` wire request, `SettingsBackend::commit_change`, D2/D3 types, catalog/effective projection, shared errors.
**Action:** New `src-tauri/src/settings_service/domains/microphones.rs` (pure `set-priority` plan, `MicrophonesList` live operation); modify `domains/mod.rs` (`production()`), `crate::cli` clap `SettingsCommand`, `settings_service/cli.rs`, `commands/audio.rs` and `lib.rs` command bindings. Retire the legacy `set_selected_microphone` command in favor of priority operations; legacy stored data remains. Generate `src/bindings.ts` through existing specta flow during implementation. New `src-tauri/tests/microphone_cli.rs`; create feature contract matrix/schema from typed DTOs during implementation.
**FR covered:** FR-002.3: Status and effective read contract, FR-007.3: UI and CLI adapter equivalence, FR-008.1: microphones Domain commands
**Execution evidence:** Real CLI processes on isolated stores, live/offline/CAS/no-op/transport-timeout/UTF-8/quoted-name cases; exactly one envelope and exit mapping.
**Success criteria:** Exact AC-008 names; zero-name empty set; `list` live answer and offline `unavailable`/5 with empty details; `set selected_microphone` and `set microphone_priority` refused with `validation_failed`/2 and `details.reason = "microphones_domain"`; per-name `details.items`; no raw settings/secrets/log names; inherited 008 timeout/routing and no-retry semantics; shared validator results.
**Failure handling:** Contract drift or second persistence path blocks release. Uncertain IPC outcome is reread before retry; enumeration failure preserves saved order.

### Step 6 — Build accessible draft editor and status refresh

**Prerequisite:** Shared command contract ready; common UI pass and bound Penflow design approval complete before UI code.
**Required inputs:** Approved microphone-priority/microphone-settings IR and IDs, existing SettingContainer/controls, i18next and Revision.
**Action:** Modify `MicrophoneSelector.tsx`, general settings registration, `src/hooks/useSettings.ts`, `src/stores/settingsStore.ts`, en/fr translations. New `src/components/settings/microphone-priority/{priority-editor.tsx,priority-row.tsx,effective-input.tsx,draft-state.ts,draft-state.test.ts}`; `src/hooks/use-microphone-status.ts` schedules view-lifetime refresh and generation fencing. Pointer reorder uses native drag events plus keyboard Move up/down, no new DnD dependency. New `tests/microphone-priority.spec.ts` for visual/keyboard/async/conflict behavior.
**FR covered:** FR-001.2: Accessible list editor, FR-002.4: Refresh and effective input UI, FR-007.4: Draft CAS and translated errors
**Execution evidence:** State tests, typed bindings/typecheck, English/French labels, focus assertions, drag/drop state screenshots and refresh fake-clock traces.
**Success criteria:** 5-second/open/Retry status, dirty draft protected, Save busy prevents duplicate dispatch, Reset draft-only and Reload explicit, no styling-based test selectors.
**Failure handling:** Missing common-pass IR/approved IDs/design proof blocks UI code; keyboard/focus/error recovery or untranslated strings block delivery.

### Step 7 — Prove acceptance, regression and operational guidance

**Prerequisite:** Steps 1–6 delivered; build/model/native dependencies available; current design authority intact.
**Required inputs:** Eight AC/three SC assertion inventory, actual release binary, isolated store, microphones/lid hardware, native operator.
**Action:** Run commands below; extend test/harness files from prior steps with AC matrix, restart, reconnect, active-edit and legacy-ambiguous cases. Update application README CLI/settings usage during implementation only. Preserve original/transcribed-prefix and WAV outcome evidence without logging speech/device names. Verify all 008 setting-domain regression tests still pass.
**FR covered:** FR-001.3: Restart and editor acceptance, FR-002.5: Availability and native proof, FR-003.4: Deadline fallback evidence, FR-004.3: Session race acceptance, FR-005.3: Clamshell regression proof, FR-006.2: Migration fault evidence, FR-007.5: Transaction parity and event proof, FR-008.2: CLI process contract proof
**Execution evidence:** Versioned binary hash, command stdout/stderr/exits, per-AC assertions, native UI/capture observations and resource/latency samples.
**Success criteria:** All AC/SC and P0/P1 gates have actual scoped proof on macOS; code builds and automated tests pass on macOS, Windows and Linux; no unrelated/global suite failures hidden; every platform claimed is observed.
**Failure handling:** Missing macOS native/UI/resource/WAV evidence keeps the feature from Implemented, with its exact cause; browser Tauri fakes and tool versions cannot certify native acceptance. Per the constitution (decision 2026-10-13), native checks that need Windows, Linux, an older Handy binary or a second user account are recorded as **pending release evidence** with the exact command (the native orchestration command of the Resolved Test Commands with `--platform windows|linux`); they never block a step, feature completion on the integration branch or the merge into `dev`, and are never claimed as passed.

## Resolved Test Commands

Availability checked 2026-10-12: Cargo/rustc 1.94.0, clippy 0.1.94, rustfmt 1.8.0, Bun 1.3.9. Local node_modules binaries absent. No surfaces.yaml; Playwright testDir is `tests`, native integration is `src-tauri/tests`. Commands are resolved, application tests not executed during planning.

| Action | Command | Status |
|---|---|---|
| Rust unit | `cargo test --manifest-path src-tauri/Cargo.toml --lib microphones` | Cargo available; planned module/tests |
| Resolver integration | `cargo test --manifest-path src-tauri/Cargo.toml --test microphone_resolution` | Target planned, compile/dependencies unverified |
| Settings/migration | `cargo test --manifest-path src-tauri/Cargo.toml --test microphone_settings` | Real-file/fault tests planned |
| Session and CLI integration | `cargo test --manifest-path src-tauri/Cargo.toml --test microphone_sessions --test microphone_cli` | Targets planned |
| Native harness | `cargo test --manifest-path src-tauri/Cargo.toml --test microphone_native -- --ignored --nocapture` | Hardware/manual gate, target planned |
| Native orchestration | `bun scripts/microphone-native-proof.ts --binary <absolute-release-binary> --platform <os> --out <proof-directory>` | Script planned; no installed native harness today |
| Frontend unit | `bun test src/components/settings/microphone-priority/draft-state.test.ts` | Bun available; proposed test |
| Existing unit regression | `bun run test:keyboard` | Script observed, not run |
| UI integration/visual | `bun run test:playwright -- tests/microphone-priority.spec.ts` | TOOL NEEDED: `bun install`, then `bunx playwright install chromium` |
| Type check | `bunx --no-install tsc --noEmit` | TOOL NEEDED: `bun install` for pinned TypeScript |
| Frontend lint/translation | `bun run lint`; `bun run check:translations` | ESLint needs local install; scripts observed |
| Rust lint/format | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Tools available, not run |
| Frontend formatting | `bun run format:check` | Prettier needs local install; no application formatting mutation now |
| Build | `bun run build`; `bun run tauri build` | BUILD.md native/model prerequisites to verify before implementation |
| Full suite | `cargo test --manifest-path src-tauri/Cargo.toml` + frontend/unit/UI/type/lint/format + native harness | Composite planned gate, not a present PASS |

## Testing Strategy

| Priority/level | Concrete planned file and assertion | FR / AC |
|---|---|---|
| P0 unit | policy inline tests and microphone_resolution.rs: ordered first usable, ambiguity, failed enumeration, default, lid unknown/no override, every started attempt gets its full 2 s, no configured attempt starts with less than 4 s left, default starts by t = 4 s, deadline-skipped choices recorded as failed, four-plus slow devices, slow enumeration, one refresh/no repeated choice | FR-002/003/005; AC-003/005 |
| P0 real-file integration | microphone_settings.rs: missing/null legacy (→ `[]`), literal `"default"` (→ `["default"]`), unavailable/ambiguous legacy, explicit empty wins, exact order after restart, idempotence, flush/replace fault and unknown-field preservation | FR-001/006/007; AC-001/006/007 |
| P0 concurrency | microphone_sessions.rs: pending edit, always-on launch/idle/active edit, reconnect before next keypress, same stream reuse, late generation, loss/stop/cancel race, no-op reopen, bounded worker cap | FR-003/004/007; AC-003/004/007 |
| P0 capture/action | microphone_sessions.rs and microphone_native.rs: recover sample prefix even on consumer death, once-only transcription, no splice, cut-short notice, post-loss release no-op, WAV failure is not history success | FR-004; AC-004; SC-001/002 |
| P0 process contract | microphone_cli.rs: genuine UI adapter versus CLI valid/invalid parity, CAS, complete-order readback, exact names/envelope/exits, live `list` and offline `unavailable`/5 with empty details, offline `get microphone_priority`, `set selected_microphone`/`set microphone_priority` refused with `details.reason = "microphones_domain"`, per-name `details.items`, IPC unknown reread | FR-007/008; AC-007/008 |
| P1 frontend unit | draft-state.test.ts: validation, busy/error/success, conflict keeps draft, Reset draft only, Reload discards, event revision and late-read fencing | FR-001/007; AC-001/007 |
| P1 UI/a11y/visual | tests/microphone-priority.spec.ts: pointer target/drag, tab/Enter move, focus retention, unavailable removal, all four text statuses, 5 s/open/Retry, current/last/not-used-yet, Save/Reset/Reload, all behavioral trait states | FR-001/002; AC-001/002/007; SC-003 |
| P0/P1 native | microphone_native.rs + native-proof.ts on macOS: unplug while speaking, known reconnect, stale opener cleanup, no main-loop freeze, enum while active, always-on switch, clamshell override/default, permissions denied. Windows/Linux: pending release evidence | FR-002–005; AC-002–005 |
| P1 regression | Existing recorder tests, full Rust plus 008 domain/transaction suite; channel invalid defaults, VAD/resampling, lazy close, push-to-talk/toggle/cancel, clamshell no override | All FR; SC-001/002 |

Inject only I/O boundaries: catalog/open/lid/error/sample drain, monotonic clock, store fault and event/output ports. Test actual policy/validation/transaction, not mocked service-under-test. Clipboard, workspace/activation, AX classification and LLM are outside F1 and remain existing output-stage ports; no unused new seams introduced here. A fake downstream transcription completion port asserts dispatch/prefix once; real native transcription evidence remains mandatory. Use barriers/callbacks/fake clocks, never arbitrary sleeps.

Native manual procedure: build against the required model/VAD resource on an isolated store; record OS/build/binary hash and anonymized device classes. Save A/B order, restart/read UI and CLI; unplug A, record with B, reconnect A and start again. Repeat on-demand/always-on and with edits during capture. Speak a known local fixture, disconnect mid-utterance, inspect retained prefix/transcription and notification exactly once; check WAV/history outcome separately. Exercise real closed lid with override present/absent and unknown-lid injected fixture, channel fallback and enumeration while stream open. Record start/attempt/end monotonic times, readiness, resource counters and UI responsiveness. The same procedure on Windows and Linux is pending release evidence (constitution): macOS proof does not certify them and they are never claimed as passed. Private audio artifacts are kept only in operator-selected isolated proof storage, never printed in debug logs.

## QE Analysis

Criticality High; blast radius Shared settings/audio pipeline; primary risk Regression; confidence Medium for design and Low for unexecuted native behavior. All native QE dimensions apply: functional correctness, regression, contract compatibility, data/migration, security/privacy, performance, accessibility/UX and observability. No remote auth/cloud provisioning is needed.

| Gate | Evidence and owner | Blocking rule / known gap |
|---|---|---|
| Plan quality | Current independent raw review, structural validate, Analyze/progression; planner | Complete review is required; no invented PASS |
| P0 audio ownership | Step 1/4 fake-clock plus macOS native worker/resource/prefix transcripts; implementer/spec-test | Caller timeout without cleanup or captured-prefix loss on macOS blocks release; Windows/Linux observations are pending release evidence, never a step gate |
| P0 data/contract | Real store generations, UI/CLI parity/CAS/no-op and 008 regression; spec-test | Partial write, erasure, lost revision or second writer blocks release |
| P1 UI/source authority | Common pass code IR/IDs, bound review/design certificate, screenshots and focus assertions; program UI producer/spec-test | Penflow incomplete today; user-approved preparation deferral is not certification |
| P1 performance/operability | Full-2 s attempt and 6 s total traces with the default slot, always-on latency samples, off-main-loop observation, actionable retries; native operator (macOS) | Missing deadline/default evidence or resource growth on macOS blocks release |
| Privacy | Sentinel assertions in stdout/stderr/errors/logs, no added name/audio/secret dumps; implementer | Projection only; F1 does not perform a broader vulnerability audit |
| Evidence integrity | AC/SC matrix with binary/OS/hash/transcripts; verifier | Tools/planned tests do not prove native success |

Expected later artifacts: `proofs/001-microphone-priority/<os>/acceptance-matrix.json`, `commands.json`, `resolution-timings.json`, `resource-observations.json`, `native-observations.md` and private captured-prefix hash/WAV outcome. These are planned outputs. QE defines sufficiency; independent review checks plan correctness, Penflow approves UI sources/design, spec-test runs tests and native capture proof. Security audit, unrelated program features and insertion verification remain separate.

## Risks & Considerations

All findings from the original specification review and refreshed model-bound review are disposed here; the original spec is not edited by planning.

| Finding | Decision / test / documented risk |
|---|---|
| F8 interface not pinned | Merged 008 names (`Domain`, `DomainChange`, `SettingsBackend::commit_change`, `tauri_adapter`, `effects.rs`) and 006 step 0 D2/D3 names above |
| AC-008 names versus provisional clarification | Register exact microphones list/set-priority AC names as Domain extension; no rename of 008 |
| UI contract/IDs/mockups deferred | Common pre-spec-ship pass; required policy retained, no certification claim |
| Default versus 6 s and blocking cpal | Every attempt keeps its full 2 s; the 6 s total limits the number of attempts and reserves the default slot (start by t = 4 s); owned attempt fence/reaper/native spike; release blocker if cleanup infeasible on macOS |
| Closed-lid carve-out | Override configured AND confirmed closed; otherwise normal; no-override and unknown-lid tests |
| Always-on per-start latency | Fresh catalog off main loop, same healthy stream reuse; measure mode-specific latency and honor 6 s bound |
| Enumeration side effects | No open/probe during refresh, one in flight; busy/error→unknown; native backend tests |
| Legacy same-name behavior changes | Preserve intent as ambiguous, skip arbitrary first-match, show ambiguity after upgrade; migration fixture |
| Missing Rust/native test strategy | Resolved Cargo commands, injected ports and native procedure in this plan; global strategy untouched |
| Names absent/default/ambiguous Add validation | Exact unique non-empty names accepted independent of presence; synthetic default excluded, real exact name allowed; grouped ambiguous persisted and skipped |
| Awkward clarification record | Preserve accepted original decision/provenance; consume named flow/screen IDs below; no speculative spec cleanup |
| P2 CLI priority versus release | All eight AC gate release despite story priority |
| Reset label confusing | Explicit translated clears-draft explanation; Save required; Reload separate; keyboard/UI tests |
| Post-loss release/toggle | Ended binding release no-op; next independent toggle can start; race/session-token tests |
| Native prefix and history | Once-only recoverable drain/transcription, no replacement splice; WAV-gated history truthfully separate |
| Main-thread/shared lock waits | Worker audio coordinator; no join/open/enum under settings/main-loop lock; observe UI responsiveness |
| 2026-10-14 review: Windows/Linux native proof as step gates | macOS native proof gates Implemented; Windows/Linux recorded as pending release evidence with their exact command (constitution) |
| 2026-10-14 review: `Domain::plan` is pure, `list` needs the audio manager | `list` is a D2 live operation of 006 step 0 (`MicrophonesList`); offline it returns the 008 live-only refusal; configured order readable with `get microphone_priority` |
| 2026-10-14 review: names announced by the 008 plan but not shipped | `commit_mutation`, `LiveSettingsEffects`, `SettingsMutation` for lists replaced by `DomainChange`, `SettingsBackend::commit_change`, `tauri_adapter::write_change`, `effects.rs` |
| 2026-10-14 review: legacy `selected_microphone` still settable | Read-only descriptor, `validation_failed`/2 with `details.reason = "microphones_domain"` (spec Clarification) |
| 2026-10-14 review: `default` sentinel vs 008 | Only null maps to `[]`; `"default"` is an exact device name (spec AC-006 and Clarification) |
| 2026-10-14 review: per-attempt budget below 2 s | Full 2 s per attempt; the 6 s total limits how many start (orchestrator decision) |

## Next Action

After current independent plan review and READY progression, the next implementation command is `/spec-implement 001-microphone-priority` with `--review-max-chars 180000` and the model resolved at implementation time; Step 5 waits for 006 step 0 (D2/D3) and the common UI contract/design pass precedes UI code. It is not executed by this planning task. No application changes, commit or push are authorized here.
