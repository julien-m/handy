---
feature: settings-cli
spec_ref: spec.md
title: Settings CLI Technical Plan
status: Planned
created: 2026-10-12
updated: 2026-10-13
---

# Implementation Plan: Settings CLI

## Summary

Add a Rust-only early `handy settings` path, backed by a single validated patch/commit service, durable revision and cross-process lock; route to the live application through private local IPC, with an offline store adapter using the same service.

Inspect [the approved functional source](spec.md) for FR-001–FR-006, AC-001–AC-008 and SC-001–SC-003. Scope is **L** despite six FR: all existing writers, three OS transports, persistence migration and Windows executable compatibility. Deliver 008 before 001, 002, 003, 004, 005, 006, 007, 009, 010. This refinement reuses the partial Step 1 primitives already on the current branch; it claims no new application implementation or runtime proof.

## Technical Context

| Aspect | Choice | Reason / current evidence |
|---|---|---|
| Native | Rust edition 2021, Cargo, Tauri 2.11.5 | Current stack and Cargo manifest |
| Parsing / serialization | Existing clap 4, serde / serde_json | Typed subcommands; no second CLI executable |
| Async | Existing Tokio; enable required net/io/time/sync production features | Bounded IPC worker; keep native main thread free |
| Frontend | React 18, Zustand, i18next, TypeScript | Existing UI adapters and settings-changed listener |
| Persistence | Existing settings_store.json object, settings payload plus top-level settings_revision | Preserve unknown top-level/payload fields and schema migrations; no SQL table |
| Lock | One persistent settings.lock inode/handle per canonical store; fs2 cross-platform file locking | Same lock for UI, app mutations, startup and CLI; proposed dependency must be reviewed/pinned |
| Installed path resolver | Direct dirs 6, already resolved as 6.0.0 in Cargo.lock | Match current Tauri desktop app_data_dir: dirs::data_dir joined with configured identifier; no guessed HOME or new model library |
| Transport | Unix domain socket (macOS/Linux); Windows named pipe | Local only, explicit owner authentication; never TCP/MCP |
| Test levels | cargo unit/integration/process, Bun adapter tests, Playwright frontend regression, native release proof | Browser mocks cannot prove process/OS behavior |
| Store identity | Canonical absolute store path + user UID/SID, SHA-256 identity | Installed and portable paths resolve identically without AppHandle |
| Infrastructure | Local user directory, lock and endpoint only | No cloud resource, credentials or provisioning |

Inspect [existing parser](../../../src-tauri/src/cli.rs), [entry point](../../../src-tauri/src/main.rs), [startup](../../../src-tauri/src/lib.rs), [settings](../../../src-tauri/src/settings.rs), [portable paths](../../../src-tauri/src/portable.rs), [UI store](../../../src/stores/settingsStore.ts) and [Cargo manifest](../../../src-tauri/Cargo.toml) for implementation context. These source inspections do not replace acceptance requirements.

## Constitution Check

| Principle | Gate / decision |
|---|---|
| Preserve source/native/frontend boundaries | PASS: native service owns validation, I/O and revision; UI translates typed errors and refreshes state |
| Derive stack/managers from manifests | PASS: retain clap/serde/Tokio/Tauri; only locking and target OS APIs are proposed additions |
| Tool availability is not runtime proof | PASS: versions below are availability only; macOS native proof is required before Implemented; Windows/Linux, older-binary and second-account observations remain pending release evidence, never step/completion/dev-merge gates |
| Specify before implementing; meaningful tests | PASS: all six FR mapped, real file/process boundaries and fault injection, no service-under-test mocks |
| Naming and minimal structure | PASS: one settings_service domain; Rust snake_case per language delta, Markdown kebab-case; preserve existing filenames |
| Project tooling overrides generic JS guidance | PASS: Bun, ESLint/Prettier and existing tests retained; no conversion to Commander, Biome or Vitest |
| i18n | PASS: backend stable English errors; UI maps code+field to translation keys and preserves English reason for parity tests |

## Requirement Inventory and Scope

| Requirement | Acceptance / measure | Implementation steps |
|---|---|---|
| FR-001 | AC-001; existing flags, no visible UI/tray/model, M1 reads ≤2 s | 1, 5, 7 |
| FR-002 | AC-002; descriptors, scalar parsing, redaction, non-creating reads | 2, 6, 7 |
| FR-003 | AC-003/004; SC-001/002, all writers, no-op/CAS | 2, 3, 4, 7 |
| FR-004 | AC-005; private IPC, 5 s, unknown outcome, older-instance refusal | 1, 3, 5, 7 |
| FR-005 | AC-006; framework plus existing output domain only | 2, 6, 7 |
| FR-006 | AC-007/008; SC-003, one envelope, channels, stable exits | 2, 5, 6, 7 |

008 does not deliver microphone priority, replacements, new vocabulary/shortcut/prompt operations, destination/PIN, smart paste execution or automatic Enter. Existing lists are readable; generic set refuses them. Their later owners register domains and aliases (`edit`→`update`, `delete`→`remove`) without changing the framework. Only `output` appears as a production domain in the 008 build. A fixture bulk/live domain is compiled only in tests.

## Architecture Decisions

### Entry point and Windows console

Use `CliArgs::try_parse_from` before any Tauri builder, specta export, model/audio manager, portable mutating initialization or single-instance callback. `settings` runs and exits here; all old invocations retain their existing run branch. Accept both JSON placements and deduplicate them. Mixed settings with recording, startup, transcription/model/device/list/repeat flags is usage_error (2); `--debug` and `--json` remain compatible. Parser failures with JSON requested use the same envelope renderer without echoing raw values. JSON help is `{ok:true,revision:N,data:{help:...}}`, human help remains readable; obtain revision using the same non-mutating read path.

Windows decision: retain one `handy.exe` with the **console PE subsystem** in debug and release. During implementation replace the current release-only `windows_subsystem="windows"` attribute with the console selection; retain inherited stdout/stderr handles throughout the `settings` branch. The existing GUI branch calls `FreeConsole` before Tauri; never hide or close the parent's console, allocate another console or introduce a launcher/manifest policy. Existing headless transcription/list operations keep their console handles. Pure dispatch tests distinguish settings, existing headless, GUI and transcription-engine worker branches. Compile the Windows target in its supported native build environment and inspect PE OptionalHeader.Subsystem=3; compilation/header inspection tests this choice without claiming shell/UI runtime evidence. Inspect [Microsoft console creation](https://learn.microsoft.com/en-us/windows/console/creation-of-a-console) and [FreeConsole](https://learn.microsoft.com/en-us/windows/console/freeconsole) as technical context. Native cmd/PowerShell output, waits/exits, redirection and GUI launch/console-flash checks on the oldest supported Windows and 24H2 are **pending release evidence**; they never block Steps 1–7, feature completion on the integration branch or merge into `dev`. A compile failure remains a code defect to fix; missing Windows native observation does not become a step gate. No platform-native success is inferred from compilation.

### Paths, storage and revision

Extract side-effect-free StoreLocator from portable initialization. Read the portable marker without upgrading it or creating Data; resolve installed user app-data using the configured identifier com.pais.handy and OS APIs, not guessed HOME strings. For nonexistent paths, canonicalize the nearest existing ancestor, append validated components and reject symlink/reparse aliases that cross user boundaries. Lock identity survives atomic store replacement: lock the stable sibling lock file, never the replaced JSON inode.

SettingsRepository reads fresh disk bytes under settings.lock; missing store gives defaults/revision 0. Read only salvage/migrate/merge defaults in memory, never save, create settings_store.json, upgrade portable marker or increment revision. Startup takes the same lock and explicitly commits any initialization/migration change once. Older stores with no revision start at 0; first changed commit is 1. Preserve a valid top-level revision through payload salvage; malformed existing revision or unrecoverable revision continuity refuses writes with persistence_failed rather than reset-and-CAS. Entire unreadable JSON reports defaults with warning for read recovery, but cannot be overwritten automatically; explicit supported recovery is outside this feature. Semantic no-op after normalization does not write or increment; migrations persisted by startup are real writes. Reject revision overflow before native effects. Never silently reset the counter.

Commit a typed mutation/patch against the newly loaded settings, not the caller's stale AppSettings snapshot. Validate the whole candidate and dependencies, compare normalized persisted values (including secret changes, excluding runtime overrides), increment once, serialize the entire preserved store object to a private same-directory temporary file, flush/sync, atomically replace and sync the parent directory on Unix (Windows replacement APIs and durable flush). Crash injection must observe old or new complete generation, never partial JSON. Disable plugin-store ownership/autosave/exit-save for this particular settings file; a reload alone cannot prevent a delayed old cache write. Other plugin stores remain unchanged. A compatibility AppHandle adapter reads the service snapshot; it cannot write the settings file independently. Activate app/startup adapters and transfer exclusive settings-file ownership only in Step4, atomically with all-writer migration; Step3 builds isolated repository/locator/normalization primitives while the existing app path and disabled settings CLI remain unchanged.

### All surfaces and live effects

Migrate every writer before CLI set becomes reachable, including get_settings migration/default stores, shortcut fallback/reset commands, UI setters, audio/model selection repairs, model switches and history retention. The inventory is the actual call-site scan, not a hand-selected subset. Replace write_settings(AppSettings) with `commit_mutation` closures/typed patches and propagate Result. Preexisting runtime-only flags and forced-disable-updater remain overrides, never persisted writes.

Inject LiveSettingsEffects for native validation/preparation, shortcut registration, audio/model reconfiguration, autostart, theme/log settings and settings-changed. Worker transaction serializes prepare/validate → durable persist → publish snapshot → apply live effects → notify. Prepare any fallible reversible native change before commit; compensate it on persistence failure. For effects that can fail after durable commit, do not falsely return validation_failed: return internal_error with details `{outcome:"committed", current_revision:N, field:...}`, keep committed data authoritative and reconcile live state before accepting another affected mutation. Timeout still reports outcome unknown to the caller. P0 tests require invalid/refused values and failed preparation leave both disk and revision unchanged, and injected effect failure never yields a success with stale shortcuts. UI native command adapters become async workers where needed; never wait for a worker holding the transaction lock from the main thread. Event contains revision and changed key names only, no secret values; the existing listener refreshes and discards older refresh responses by revision.

### Routing, startup fence and legacy compatibility

New app acquires a per-user/store instance lease before loading settings, under settings.lock, publishes its private endpoint before releasing startup ownership, and keeps the lease until shutdown. A headless writer acquires settings.lock, then checks the new lease/endpoint and passive legacy state again immediately before commit. GUI startup for the new version uses that same lock; it reads a complete commit or establishes ownership before the CLI can mutate. The lease is separate from the transaction lock; a busy or starting owner never means offline permission. A stale new-version endpoint is reclaimable only after OS liveness/ownership proof and protected removal; ambiguous access returns unavailable with no write.

| OS | Private request channel / peer verification | Passive older-instance detection |
|---|---|---|
| macOS | Short AF_UNIX SOCK_STREAM path in a canonical current-UID-owned directory (0700), with owner/type/device/inode guards; both endpoints enforce kernel peer UID via getpeereid before any application bytes | Inspect `/tmp/com_pais_handy_si.sock` type/owner without connecting, reading a legacy frame, invoking callback or removing it. Reuse inspect_legacy_path and its Absent/Occupied/Unavailable results. Stale/uncertain legacy resources refuse mutation rather than authorize deletion |
| Linux | AF_UNIX SOCK_STREAM under validated current-UID-owned directory (0700), XDG_RUNTIME_DIR or short private fallback, with owner/type/device/inode guards; both endpoints enforce checked SO_PEERCRED UID before any application bytes | Query session DBus `com.pais.handy.SingleInstance` through NameHasOwner/GetNameOwner only; never own that name, serve ExecuteCallback or call it. Errors/unknown scope refuse mutation |
| Windows | Named pipe keyed by SID+store hash, protected current-logon-SID DACL, reject remote clients and verify client token | Query legacy `com.pais.handy-sim` mutex and `com.pais.handy-sic`/`com.pais.handy-siw` target without WM_COPYDATA. Never reserve the mutex or create a compatibility window/message loop. Missing/ambiguous ownership evidence refuses mutation |

Unix channel authorization uses the validated canonical current-UID directory with mode 0700 plus retained directory/socket owner, type, device/inode and single-link checks. Revalidate directory identity before/after bind and before accept/connect; refuse hostile aliases/replacements without touching substitutes. Bind without changing process-global umask, pathname chmod or a bootstrap-permission framework. Socket mode is diagnostic, not a portable security boundary. On macOS, ACLs can grant traversal/connection beyond ordinary mode bits, so mode 0700 alone is not exhaustive access proof: accepted clients and connected servers must each pass kernel-derived expected UID authentication before request parsing, response rendering or any application read/write. Failure closes/refuses the connection with no application bytes or store mutation; protocol UID/PID fields are never authorization evidence. Same-UID identity changes fail closed; no atomic pathname-race guarantee is claimed. Failed/closed socket names are retained rather than removed through a racy stat/check-then-unlink; Step 5 implements reviewed lease-proven reclamation so normal restart remains possible. Never delete the legacy singleton path.

**Informative:** Inspect [Linux unix(7)](https://man7.org/linux/man-pages/man7/unix.7.html) for pathname/socket permission portability and kernel SO_PEERCRED context; inspect [Apple getpeereid(3)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html) for both-side peer effective UID authentication. These references explain the mechanism; they add no external normative requirement or acceptance graph.

Inspect the locked [macOS plugin source](https://raw.githubusercontent.com/tauri-apps/plugins-workspace/single-instance-v2.4.0/plugins/single-instance/src/platform_impl/macos.rs), [Windows source](https://raw.githubusercontent.com/tauri-apps/plugins-workspace/single-instance-v2.4.0/plugins/single-instance/src/platform_impl/windows.rs) and [Linux source](https://raw.githubusercontent.com/tauri-apps/plugins-workspace/single-instance-v2.4.0/plugins/single-instance/src/platform_impl/linux.rs). Cargo.lock resolves 2.4.0; recalculate identities if that dependency changes. Legacy identities do not prove store identity, so refuse conservatively for a same-user legacy owner even with an apparently different store. Do not connect/send an empty command: macOS empty streams can invoke show-window. Detected old instance means unavailable/5 and byte-identical store/revision, for reads or writes that would otherwise go to it.

Minimal AC-005 scope: passive detection and refusal only. **No legacy invocation handoff, worker, spool, capture, acknowledgement, replay or recovery daemon is planned.** Offline CLI and new startup never impersonate an old singleton. Existing historical flags still use their current single-instance flow outside `settings`. Remove obsolete spike decoder/spool/reservation code and handoff tests during implementation, preserving the passive probe and reusable private-resource/peer-UID helpers. No code is removed by this planning command. Older versions do not participate in the new lock: a launch after the final passive probe is not claimed to be atomically fenced or delivered/replayed. This unsupported old-version race is documented; deterministic probes refuse an owner that appears before commit. Real older-binary race observations are pending release evidence, never a demand to recreate replay.

Requests to a running new app carry protocol version, user/store identity, opaque request ID, typed operation, expected revision and dry_run. Bound UTF-8 frames to 1 MiB, reject malformed/oversized input without raw dumps. Domain imports share this technical limit until their owner defines another. Requests contain no raw argv. Use a monotonic 5 s end-to-end response deadline, including lock/persistence/transport wait. Successful M1 reads finish within 2 s; timeout failures are measured separately. No offline fallback write or blind retry after submission/timeout; reread revision/value first. Late replies cannot resurrect an expired request.

### Descriptors, output domain and contract rendering

Descriptor inventory covers every AppSettings field, with wire type, enum/range, mutable/read-only, secret/presence projection, owning domain, runtime effect and validator. Build a reviewed manifest plus an independent test inventory from serialized default settings and all UI adapters; a descriptor omission fails, not silently hides a setting. Existing scalar settings are generic-settable except internal fields (schema, revision, onboarding/migration markers, manager-only model state) and provider configuration, which remain read-only from CLI. API-key fields are secret and never generic-settable. Lists/maps return validation_failed naming their owning domain command, plus `not available in this build` when unregistered; include owning command even when unavailable to satisfy AC-002. Optional values have descriptor-defined null handling; non-null boolean accepts exactly true/false, numeric parses without overflow, enums use existing serde values, strings preserve bytes without trimming meaningful content.

Redact via explicit public projection, never AppSettings Serialize or Debug. `post_process_api_keys` becomes per-provider `{present:boolean}` only; nested/future secrets inherit the rule. All stdout, stderr, parse errors, IPC diagnostics and logs sanitize raw candidate values. Public errors use stable English text and object details, with field/issues where applicable; no OS trace, raw document or credential in details. Reject `paste_method=invalid_method` identically via UI and CLI rather than existing default-to-ctrl_v behavior. UI maps code+field to i18n keys, falling back to a translated generic error, not free-text parsing.

`output get [KEY]` and `output set KEY VALUE` delegate to the same descriptors for existing paste_method, clipboard_handling, auto_submit, auto_submit_key, paste_delay_ms, paste_delay_after_ms, reliable_paste, typing_tool, external_script_path and append_trailing_space. Existing enum values remain unchanged: ctrl_v/direct/none/shift_insert/ctrl_shift_v/external_script; dont_modify/copy_to_clipboard; enter/ctrl_enter/cmd_enter. When 007 ships, it extends the same output descriptor and validation path with paste_method=smart_paste on macOS (validation_failed outside macOS), using the existing commit_mutation service and Domain trait; 008 alone keeps the existing methods. Feature 007 owns the macOS migration/default and removal of the beta reliable_paste switch; the Windows reliable_paste switch/path and Linux methods stay unchanged. This is configuration, not insertion execution. Keep destination 006/009, insertion method and clipboard handling separate. The 007 outcome contract is: "insertion attempted, read observed", "not read", "permission missing", "typed", "clipboard handed over", "insertion failed". Only read observed and typed authorize automatic Enter in 009; not read, permission missing and insertion failed route to recovery in 010. Clipboard handed over is neither Enter nor verified insertion. Outcomes wait for the read/fallback decision and never claim text verified in a field. A committed output setting proves configuration only.

Domain trait registers command schema/help, file/`-` input, pure parsing/validation, preview, typed patch and export formatter. Valid dry-run loads latest revision, validates fully, emits preview/revision with exit 0, and does not persist or call effects. Invalid dry-run returns validation_failed/2 with issues, no success preview. `-` explicitly requests stdin, never prompts; non-UTF-8 text returns byte offset in details. Export `-` human mode is raw exchange content only; JSON is data.content. Fixture domain proves bulk merge/replace/atomic all-or-nothing and live-only unavailable behavior; it never appears in production help.

| Result | Exit |
|---|---|
| success | 0 |
| persistence_failed, internal_error | 1 |
| usage_error, validation_failed, shortcut_conflict, limit_exceeded | 2 |
| revision_conflict (`details.current_revision`) | 3 |
| not_found | 4 |
| unavailable (timeout: `details.outcome=unknown`) | 5 |

The enum+single renderer owns this mapping. Every JSON operation emits exactly one `{ok:true,revision:N,data:...}` or `{ok:false,error:{code,message,details:{...}}}` followed by newline; diagnostics go to stderr. No newly introduced HTTP endpoints: OpenAPI would misrepresent local IPC. Contract is the typed DTO/JSON-schema and command matrix in Step 2; no openapi.yaml generated.

## Gherkin Scenarios and Sequence Diagrams

```gherkin
Feature: Shared live settings transaction
  Scenario: Successful live mutation
    Given the same user's instance has revision 41 and a private channel
    When the CLI sets paste_method to ctrl_v with expected revision 41
    Then the instance persists revision 42 before confirming success
    And native effects finish and the UI receives settings-changed
  Scenario: Stale or timed-out request
    Given current revision is 42
    When expected revision is 41
    Then revision_conflict returns current_revision 42 without mutation
    When an accepted request produces no answer within 5 seconds
    Then unavailable reports outcome unknown without offline fallback
```

```mermaid
sequenceDiagram
  participant CLI
  participant IPC as Private channel
  participant Service as Live commit worker
  participant Disk as Fresh repository and lock
  participant Native as Native effects
  participant UI
  CLI->>IPC: operation, store identity, expected revision
  IPC->>Service: authenticated typed request
  Service->>Disk: lock and reread
  alt revision mismatch or invalid
    Service-->>CLI: error envelope; no write
  else changed candidate
    Service->>Native: prepare reversible effects
    Service->>Disk: atomic persist, revision plus 1
    Service->>Native: apply and confirm
    Service->>UI: settings-changed revision and keys
    Service-->>CLI: success envelope
  else no-op
    Service-->>CLI: current revision unchanged
  end
  alt no reply before deadline
    CLI->>CLI: unavailable, outcome unknown; reread before retry
  end
```

```gherkin
Feature: Safe offline settings access
  Scenario: Read missing or salvageable store
    Given no Handy instance exists
    When the CLI reads settings
    Then defaults or salvage are returned without creating or rewriting the store
    And a warning goes only to stderr when recovery was needed
  Scenario: Startup competes with offline write
    Given a GUI startup and headless write target the same store
    When both acquire shared ownership and transaction guards
    Then the write is visible at startup or routed to the new instance
    And each changed transaction increments revision exactly once
  Scenario: Older instance present
    Given an older same-user Handy instance holds its legacy identity
    When the CLI requests a mutation
    Then unavailable is returned without a callback or store write
```

```mermaid
sequenceDiagram
  participant CLI
  participant Owner as Lease and legacy probe
  participant Repo as Locked fresh repository
  participant App as Starting app
  CLI->>Owner: inspect private lease and legacy identity
  alt authenticated new instance
    Owner-->>CLI: route private request
  else old, busy, or uncertain
    Owner-->>CLI: unavailable; no write
  else proven offline
    CLI->>Repo: take transaction lock
    CLI->>Owner: recheck new lease and passive legacy state
    CLI->>Repo: read or validate patch
    alt read or dry-run
      Repo-->>CLI: projection or preview; no persist
    else changed write
      Repo->>Repo: durable replace with next revision
      Repo-->>CLI: success
    end
    App->>Repo: startup waits, then reads committed store
    CLI->>Owner: release transaction lock
  end
```

## State Model

```gherkin
Feature: Mutation outcome
  Scenario: Refusal or no-op
    Given a current durable revision
    When validation fails or expected revision is stale
    Then the durable state and revision remain unchanged
    When a valid patch changes nothing
    Then success carries the unchanged revision
  Scenario: Failure after durable commit
    When persistence commits but a live effect fails
    Then the revision remains committed and error details state outcome committed
    And the affected runtime state is reconciled before another affected mutation
  Scenario: Timeout at the caller
    When the deadline expires after submission
    Then the caller reports outcome unknown until a fresh read resolves it
```

```mermaid
stateDiagram-v2
  [*] --> Parsed
  Parsed --> Refused: invalid usage or unavailable owner
  Parsed --> Locked: owner established
  Locked --> Refused: validation or revision conflict
  Locked --> NoOp: same normalized value or dry-run
  Locked --> Prepared: valid changed patch
  Prepared --> Refused: prepare or persistence fails, compensate
  Prepared --> Durable: atomic replace succeeded
  Durable --> Applied: native effects and event complete
  Durable --> Reconcile: effect failure, outcome committed
  Applied --> [*]: success
  Reconcile --> Applied: repair live state
  Refused --> [*]: error, disk unchanged
  NoOp --> [*]: success, revision unchanged
  note right of Durable: Client timeout is an unknown observation, not rollback
```

## Data Model

Logical ER diagram only: existing JSON storage gains metadata; no new SQL tables or joins.

```mermaid
erDiagram
  SETTINGS_DOCUMENT ||--|| SETTINGS_REVISION : contains
  SETTINGS_DOCUMENT ||--o{ SETTING_DESCRIPTOR : projected_by
  OPERATION_REQUEST }o--|| SETTINGS_DOCUMENT : targets
  OPERATION_REQUEST ||--|| CLI_ENVELOPE : answers
  SETTINGS_DOCUMENT {
    string store_identity PK
    object settings
    object preserved_unknown_fields
  }
  SETTINGS_REVISION {
    string store_identity PK
    integer value
  }
  SETTING_DESCRIPTOR {
    string key PK
    string wire_type
    string owning_domain
    boolean read_only
    boolean secret
  }
  OPERATION_REQUEST {
    string request_id PK
    string store_identity FK
    string operation
    integer expected_revision
    boolean dry_run
  }
  CLI_ENVELOPE {
    boolean ok
    integer revision
    object data_or_error
  }
```

## Implementation Plan

### Step 1 — Reuse the partial spike and narrow platform startup

**Prerequisite:** Current spec review complete+ready and progression plan READY; this revised plan independently reviewed before code resumes. The user's 2026-10-13 instruction authorizes this plan-only refinement despite existing implementation.md; no code or spec is changed here.
**Required inputs:** Locked plugin 2.4.0, current branch settings_service modules and settings_platform tests, prior progress/implementation checkpoints, isolated stores, platform compilation environments.
**Action:** Modify existing `settings_service/instance_probe.rs` to retain passive fail-closed detection. Refactor existing `platform_unix.rs` to retain private directory/file identity and explicit resource lifetime, removing LegacyReservation and legacy-frame capture dependencies. Enforce kernel peer-UID checks at both accept and connect boundaries before any application bytes under the Unix directory/identity contract above; remove process-global mask changes and pathname permission mutation, with no bootstrap framework. Retain failed/closed socket names for Step 5 lease-proven reclamation. During implementation remove obsolete `offline_guard.rs`, `legacy_invocation.rs` and `tests/settings_platform/handoff.rs`, and their test imports; keep historical-flag regression in the actual parser tests. Modify existing `src-tauri/tests/settings_platform.rs` to test passive presence/uncertainty, no connect/callback/delete and private-resource isolation. Add Linux passive DBus/private IPC support and new `platform_windows.rs` for passive mutex/window inspection and private pipe APIs; no compatibility listener. Modify main.rs for the simple console/GUI dispatch policy above and test its branch selection. Keep production CLI mutation disabled until all-writer migration and repository gates pass.
**FR covered:** FR-001.1: Console and early-path spike, FR-004.1: Private routing and legacy probes
**Execution evidence:** New targeted Cargo and compile transcripts, source/import inventory proving no spool/replay production route, injected legacy-owner store hashes, macOS private directory/identity, both-side peer-before-bytes and multithreaded bind/unrelated-file permission proof without changing process mask, plus Windows PE subsystem check. Inspect [the historical progress](progress.md) and [partial mappings](implementation.md) as context only: their 305 passed/16 ignored supported-stub diagnostic report is not a fresh PASS of this changed scope or certified AC.
**Success criteria:** Retained passive tests and private-resource invariants pass; presence/uncertainty returns unavailable without callback/store mutation; simple Windows dispatch is testable by compile/header checks; changed platform code compiles and automated tests pass in supported build environments. Windows/Linux native, real old-binary and second-account observations are explicitly pending release evidence, not prerequisites for Step 2. MacOS full settings-path/native acceptance is produced after wiring in Step 7, before Implemented.
**Failure handling:** Fix compilation, automated-test or macOS primitive failures before advancing; record unavailable foreign build environment separately from code failure. Never block Step 2 for missing Windows/Linux/older-binary/second-account native proof or obsolete replay work. Resume progress/implementation mappings during implementation to supersede their old blocked gates; do not rewrite those historical checkpoints during this plan refinement.

### Step 2 — Define descriptors, validators and DTO contracts

**Prerequisite:** Step 1 retained primitives and automated checks pass; current UI/writer inventory captured. Pending release evidence does not block this step.
**Required inputs:** All AppSettings fields, UI command validators and current output enums, spec error table.
**Action:** Add the missing `settings_service/{mod.rs,descriptor.rs,validation.rs,protocol.rs,errors.rs}` around the retained Step 1 modules define SettingDescriptor, SettingsMutation, SettingsSnapshot, Revision, PublicSettings, OperationRequest and SettingsError. Build shared validators for every corresponding UI setting and existing domain operation; migrate invalid-to-default behavior to errors. New inline unit tests and `src-tauri/tests/settings_validation.rs` exercise real adapters against an independently maintained cases inventory. Generate `contracts/settings-cli.schema.json` from DTOs during implementation, plus `contracts/settings-cli.md` command matrix; schemas are not OpenAPI. Modify Cargo.toml for reviewed fs2, OS APIs and required Tokio features, not new model dependencies.
**FR covered:** FR-002.1: Complete descriptor and redaction model, FR-003.1: Shared typed validation, FR-005.1: Domain extension interface, FR-006.1: Typed errors and envelope contract
**Execution evidence:** Inventory diff, schema contract tests, valid/invalid/boundary cases including booleans, integers, enums, optional strings, read-only/secret/list keys.
**Success criteria:** Every persisted key classified; 100% settable-key UI/CLI parity including English reasons; unknown and unregistered-domain guidance deterministic; no secret serialization path.
**Failure handling:** Missing descriptor, divergent validation or leaked value is P0; repair before transaction integration.

### Step 3 — Build atomic repository, revision and locking

**Prerequisite:** Step 2 validators and mutation types stable; repository work is isolated/pure, with no running-app adoption before the coordinated Step4 cutover.
**Required inputs:** Pure StoreLocator, raw store shape, extracted normalization/migrations/salvage helpers, stable lock, new-instance lease and passive legacy probe; current Tauri desktop path resolver and already-locked dirs 6.
**Action:** New `settings_service/{store.rs,transaction.rs}` implement isolated non-creating `read_snapshot`, lock/reload/patch `commit_mutation`, pure startup normalization transaction logic, no-op/CAS/overflow and durable replacement. Expose side-effect-free locator/normalization helpers from portable/settings logic, using direct dirs 6 (already locked) to match Tauri native path resolution. Keep existing AppHandle reads/writes/startup and settings-file plugin ownership unchanged; no production CLI mutation reachability. Do not add a temporary full-snapshot-to-patch adapter or global provenance/cache machinery. New `src-tauri/tests/settings_transactions.rs` uses private stores, barriers, two subprocess repository writers and crash/fault injection.
**FR covered:** FR-002.2: Non-mutating fresh reads, FR-003.2: Durable revision and atomic patch, FR-004.2: Startup ownership fence
**Execution evidence:** Isolated repository before/after hashes, revision sequences, independent raw-file reader, crash old-or-new snapshots, lock identity tests across rename and pure normalization/startup-transaction races. Running-app startup/cache/no-stale-plugin-flush proof belongs to the Step4 all-writer cutover.
**Success criteria:** Isolated repository mutations lose no fields; stale CAS refused, no-op/read invariant, secrets/unknown fields preserved and pure startup normalization observes complete generations. No claim that the running app has adopted this repository or that its old cache/writers are already removed.
**Failure handling:** Persistence failure leaves old durable generation; uncertain durable outcome is explicit, never blanket retry; invalid revision continuity blocks writes. Any attempted running-app/service adoption before all-writer closure is blocked until Step4, rather than repaired with an ambiguous snapshot adapter.

### Step 4 — Migrate every app writer and live side effect

**Prerequisite:** Isolated Step3 repository tests PASS; no running-app repository adoption or CLI mutation entry point enabled. Prepare one coordinated cutover covering every existing writer and settings-specific plugin ownership.
**Required inputs:** Full write_settings and store.set inventory, manager repairs, startup migrations, typed native ports.
**Action:** Modify settings.rs, shortcut/mod.rs, commands/{mod.rs,audio.rs,models.rs,history.rs,transcription.rs}, managers/{audio.rs,model.rs} and lib.rs. Replace every snapshot rewrite with a minimal mutation applied to latest state; adapt fallback repairs and secret UI writes. In the same coordinated all-writer cutover, activate AppHandle/service reads, writes and app startup normalization, transfer exclusive settings_store.json ownership and disable that file's plugin autosave/exit-save participation; retain other plugin stores. Never expose repository adoption alongside stale full-snapshot writers. New `settings_service/live.rs` owns ordered effects and reconciliation. Modify settingsStore.ts to consume revisions/typed errors, regenerate bindings.ts via supported specta export and add en/fr i18n keys using existing translation checks. No new screens/layout. New tests verify writer inventory closure and genuine UI adapters, compensation and post-commit failure behavior.
**FR covered:** FR-003.3: All writers and live adapters, FR-006.2: Stable UI errors and translations
**Execution evidence:** Exhaustive writer scan with no direct settings-file writer remaining, settings-specific plugin ownership/autosave/exit-save closure, actual app startup/write race and no-stale-cache-flush proof, parity report, manager mutation race tests, UI event/read ordering tests and native shortcut re-registration evidence.
**Success criteria:** App adopts the repository only with complete all-writer/settings-file ownership closure; startup sees complete commits with no independent stale-cache flush. Every committed app write increments once; rejected values change nothing; native updates/UI refresh complete before success; runtime-only settings stay runtime-only.
**Failure handling:** Any direct writer/cache autosave remaining blocks CLI enablement; main-thread waits, rollback failure or stale effect is P0.

### Step 5 — Wire private IPC and early CLI execution

**Prerequisite:** Steps 1–4 PASS and old writer inventory empty.
**Required inputs:** Same-user/store lease and channel, absolute paths, monotonic Clock, existing cli.rs/main.rs/lib.rs routing.
**Action:** New `settings_service/{ipc.rs,cli.rs,render.rs}`; modify cli.rs with nested clap types, main.rs early return and lib.rs live endpoint lifetime. Authenticate accepted client and connected server kernel UID before any application bytes and validate the protocol; bounded frames, 5 s total deadline, no fallback after timeout. Implement graceful endpoint/lease lifecycle and reviewed lease-proven stale-path reclamation; never treat retained paths as permanent normal-restart refusal or delete the legacy singleton path. Implement generic operations and all parse/error JSON paths. Existing single-instance recording forwarding stays separate. Native FFI is isolated in reviewed platform modules with SAFETY comments; no unsafe domain code.
**FR covered:** FR-001.2: Early settings subcommand path, FR-004.3: Authenticated live and offline routing, FR-006.3: Single renderer and exit boundary
**Execution evidence:** Binary process tests for all old flags and both JSON positions; mocked output ports plus real local socket/pipe process tests, injected foreign-identity denial and late-answer tests; a real second-user run remains pending release evidence.
**Success criteria:** No Tauri/model setup on settings path; live-only requires owner; old or uncertain instance never permits fallback; outputs/exits obey exact contract.
**Failure handling:** Protocol, peer or liveness uncertainty returns unavailable; successful reads exceeding budget or no envelope on parser failure blocks release.

### Step 6 — Deliver output domain and bulk framework

**Prerequisite:** Step 5 generic routing operational in isolated tests.
**Required inputs:** Registered descriptor set, input reader, preview and export ports.
**Action:** New `settings_service/{domains/mod.rs,domains/output.rs,input.rs}` registers existing output get/set, extension aliases/help and bounded file/stdin processing. New test-only domain uses the production framework for import/export, merge/replace, dry-run and live-only fixture operations. New `src-tauri/tests/settings_domains.rs` asserts help contains exactly production domains and export channel behavior. Update README.md CLI usage and recovery/no-blind-retry instructions during implementation.
**FR covered:** FR-002.3: Generic list and secret denial, FR-005.2: Output and bulk framework, FR-006.4: Help export and agent recovery docs
**Execution evidence:** Valid/invalid UTF-8 and offset, invalid dry-run exit 2, valid preview zero mutations/effects, atomic bulk rejection, help snapshots and raw/JSON export tests.
**Success criteria:** Only output production domain; no premature future settings; one shared commit path, no stdin prompt, additive domain registration.
**Failure handling:** Domain-specific policy stays with owning feature; invalid preview cannot be accepted as success.

### Step 7 — Prove macOS acceptance and track release evidence

**Prerequisite:** Steps 1–6 complete; automated platform builds/tests run in supported environments; macOS release binary available. Native Windows/Linux, older-binary and second-account artifacts are not entry gates.
**Required inputs:** Independent AC/FR inventory, actual macOS binary, native harness/schema, isolated installed/portable stores and release-evidence check matrix.
**Action:** Add `src-tauri/tests/{settings_cli.rs,settings_routing.rs}` and `scripts/settings-native-proof.ts`; add `tests/settings-events.spec.ts` for existing frontend behavior. Run the automated commands below and mandatory macOS native checks, capture M1 read latency, no window/tray/model, real new-app IPC, live settings refresh and shortcut registration. The Bun child-process harness uses real binaries/private stores, not browser-only Tauri mocks. Exercise older-owner/foreign-peer branches with injected resources deterministically, then list the exact real Windows/Linux/older-binary/second-account checks as pending release evidence below.
**FR covered:** FR-001.3: Native flag and latency evidence, FR-002.4: Discovery and secret tests, FR-003.4: Parity and concurrency proof, FR-004.4: Real platform routing proof, FR-005.3: Domain acceptance proof, FR-006.5: Every owned code and exit proof
**Execution evidence:** macOS binary hash/OS/tool versions, stdout/stderr/exits, raw store generations, AC/FR/SC assertion matrix, native observations and every latency sample; build/test transcripts for supported code plus explicit pending release evidence rows with exact checks and no PASS.
**Success criteria:** Required automated P0/P1 checks have zero unexplained failures; macOS native proof is complete before Implemented; every AC/SC has a truthful coverage row distinguishing proved assertions from the pending platform/old-binary/second-user subsets. Those pending observations never block implementation steps, feature completion on the integration branch or merge into dev, and are never declared passed.
**Failure handling:** Missing macOS native acceptance blocks Implemented; automated failure/data/secret defects require repair. Missing Windows/Linux/old-binary/second-account native observations stay pending release evidence, with exact check and owner, rather than BLOCKED for this feature or dev merge.

## Resolved Test Commands

Cargo 1.94.0 and Bun 1.3.9 availability rechecked 2026-10-13; tests/builds below have **not been run** by this planning command. Only aarch64-apple-darwin is installed locally; Windows/Linux compilation runs in supported target build environments during implementation. No surfaces.yaml exists; browser testDir is `tests` from playwright.config.ts, Rust integration directory is `src-tauri/tests`.

| Action | Command | Tool / status |
|---|---|---|
| Rust unit | `cargo test --manifest-path src-tauri/Cargo.toml --lib settings_service` | Cargo 1.94.0/rustc 1.94.0 available; service tests planned; Step 1 primitives exist |
| Transaction integration | `cargo test --manifest-path src-tauri/Cargo.toml --test settings_transactions` | Cargo available; target planned |
| Validation parity | `cargo test --manifest-path src-tauri/Cargo.toml --test settings_validation` | Cargo available; target planned |
| Native process/IPC | `cargo test --manifest-path src-tauri/Cargo.toml --test settings_cli --test settings_routing --test settings_platform --test settings_domains` | Cargo available; settings_platform exists and will be narrowed, other targets planned; native dependencies unverified |
| Platform compilation | `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`; `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-unknown-linux-gnu` | Run in supported Windows MSVC/Linux SDK environments; locally only macOS target installed; not run, no cross-build PASS |
| Platform automated tests | `cargo test --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc`; `cargo test --manifest-path src-tauri/Cargo.toml --target x86_64-unknown-linux-gnu` | Run natively on matching build agents, not foreign executable on macOS; target adapters must compile/test, native manual observations remain pending |
| All Rust | `cargo test --manifest-path src-tauri/Cargo.toml` | Available; full native build prerequisite, no result claimed |
| Native release harness | `bun scripts/settings-native-proof.ts --binary <absolute-release-binary> --platform <macos\|windows\|linux> --out <proof-directory>` | Bun 1.3.9 available; script planned, not installed harness |
| Frontend unit/regression | `bun run test:keyboard` | Existing script; Bun available |
| Frontend adapter | `bun test src/stores/settings-errors.test.ts` | Bun available; planned pure error/refresh adapter tests |
| Browser integration | `bun run test:playwright -- tests/settings-events.spec.ts` | Declared @playwright/test, local binary absent: TOOL NEEDED `bun install`, then `bunx playwright install chromium` |
| Visual | N/A for 008; existing UI native observation and frontend regression only | No new screens/mockups; no visual certification claim |
| Type check | `bunx --no-install tsc --noEmit` | tsc 6.0.3 found in tooling resolution, differs from manifest ~5.6.3; use local pinned dependency after bun install |
| Lint | `bun run lint`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | ESLint local binary absent: TOOL NEEDED `bun install`; clippy 0.1.94 available |
| Format | `bun run format:check`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Rustfmt 1.8.0 available; Prettier local install not verified |
| Build | `bun run build`; `bun run tauri build` | Defined scripts; native SDK/model resource requirements from BUILD.md to verify before implementation |
| Full acceptance suite | Rust suite + keyboard + adapter + Playwright + type/lint/format + mandatory macOS native harness; foreign native checks tracked pending | Composite implementation gate; pending release evidence is non-blocking; no current suite PASS |

## Testing Strategy

| Priority / level | Behavior and concrete planned file | Requirement |
|---|---|---|
| P0 unit + parity integration | Descriptor completeness, types, ranges, null, list/read-only/secret refusals, all UI command validation reasons; `settings_validation.rs` and inline validator tests | FR-002/003; AC-002/003; SC-001 |
| P0 file/process integration | Two CLI writers and UI adapter writer with barriers; different/same keys, stale CAS, no-op, startup migration, secret writes, invalid revision, rename lock, crash/flush faults; `settings_transactions.rs` | FR-003; AC-004; SC-002 |
| P0 IPC/process | Live/no instance/older/startup/busy/stale/different-user/store; permissions, protocol/frame errors, deadline and late replies, persisted-but-no-answer reread, passive detected-old refusal with unchanged store, new-app startup race, injected foreign-peer denial and no compatibility listener; `settings_routing.rs`, `settings_platform.rs` | FR-004; AC-005 |
| P0 contract process | All seven owned error codes with actual stdout/stderr/exit, malformed argv, two JSON placements, redaction sentinels in every branch including panic/error diagnostics; `settings_cli.rs` | FR-001/002/006; AC-001/002/007/008; SC-003 |
| P0 unit + boundary | Bulk fixture valid/invalid/dry-run, file/stdin UTF-8, all-or-nothing, export and exact domain help; `settings_domains.rs` | FR-005/006; AC-006/007 |
| P1 frontend + native | UI refresh without restart, native shortcut registration, translated errors and late response ordering; `tests/settings-events.spec.ts`, `settings_platform.rs`, native proof script | FR-003/004; AC-003/005 |
| P1 native release | Old flags inventory, no window/tray/model/single-instance show, Windows cmd/PowerShell interactive and pipe/redirection exits, installed/portable launch | FR-001/006; AC-001/008 |
| P1 performance | M1 release wall time including process startup, 30 fresh process reads offline/live; retain every sample and require every successful read ≤2000 ms; deadline failures separately identified | AC-001/005 |

Seams: inject StoreLocator/Repository fault ports, InstanceProbe, LocalTransport, monotonic Clock/Deadline and LiveSettingsEffects. Audio enumeration is behind AudioDeviceCatalog for shared validators; shortcut registration behind ShortcutRegistry. Clipboard, workspace/activation, AX classification and LLM clients are not called by 008 configuration; reserve typed owner ports in their later features rather than create unused implementations here. Spy only on side-effect ports; test actual validation/service logic and use real filesystem/IPC/process integration. Use barriers/events and deadlines, not arbitrary sleeps.

Native proof records target OS and build, installed/portable store identity, binary SHA, command arguments with secrets omitted, start/end monotonic timestamps, exit, stdout/stderr and durable revisions, UI refresh/shortcut observations and process/window/tray/model state. On Windows test cmd and PowerShell interactive, piped and redirected invocations on oldest supported OS and 24H2; verify no parent console hidden/closed. On macOS require M1 release/new-app evidence; a real older app callback/activation observation is pending release evidence. On Linux verify session DBus plus private socket under X11/Wayland without triggering GUI. Tests never target the real user store.

## Native Evidence Policy and Pending Release Checks

Read [the constitution](../../constitution.md) for the 2026-10-13 native-evidence decision. Required automated tests/compilation remain distinct from native observation; a missing foreign target environment is not a claim of compilation success and does not turn a pending native check into a step gate. No acceptance certification is inferred from the existing partial spike or HANDY_FORCE_AI_STUB build: stub diagnostics must be followed by real macOS release evidence.

| Evidence / owner | Exact verification | Status / effect |
|---|---|---|
| macOS M1 / implementer + spec-test | Actual release binary, private installed/portable stores; offline and real new-app get/set; 30 fresh live/offline reads each ≤2000 ms, 5 s timeout/unknown, CAS/concurrency, secret sentinels, UI refresh/shortcut re-registration, no window/tray/model and old-only flag regression | Required before Implemented; not claimed passed here |
| Windows / platform operator | Native build/tests plus PE subsystem=3; separately run cmd and PowerShell get/set/help/failures interactively, piped and redirected; check exit/output, parent console remains visible/usable, GUI/autostart has no persistent console, old flags and installer/portable identity on oldest supported OS and 24H2 | Native observations **pending release evidence**; never block a step, integration feature completion or dev merge |
| Linux / platform operator | Native build/tests; separately run real app/headless private-socket/DBus routing on X11 and Wayland, installed/portable identity, no activation/tray/model, owner-only endpoint and old flags | Native observations **pending release evidence**; same non-blocking rule |
| Older Handy binary / platform operator | Record old executable version/hash; leave it running without settings channel; get/set returns unavailable/5, store hash/revision unchanged, no show-window callback or old control request; repeat owner appearing before final probe | **pending release evidence**; no replay/delivery test exists in minimal scope; same non-blocking rule |
| Second user/session / platform operator | Real other UID/SID attempts socket/pipe access to private test store; access denied/unavailable, no data/secret leak or changed store; verify installed/portable and endpoint isolation between accounts/sessions | **pending release evidence**; injected identity/ACL unit tests still required; same non-blocking rule |

Pending rows remain attached to the feature's release evidence inventory until actually observed; they never become passed by a macOS fixture, compilation or reviewer verdict. A later observed code/security defect is repaired on its own merits. No legacy spool/replay proof is requested.

## QE Analysis

Criticality High; blast radius Cross-system/shared settings; primary risk Data; confidence Medium until real implementation evidence. Dimensions: functional correctness, regression, CLI contract, data/migration integrity, security, performance, UX/focus and observability. UX is applicable through no activation and existing UI refresh, though new layout/visual baseline is N/A.

| Gate | Required evidence / owner | Known gap and blocking rule |
|---|---|---|
| Plan quality | Independent raw plan review + structural/progression; planner | Current command supplies documentation/review only |
| Data integrity P0 | Concurrency/crash/no-op/CAS transcripts and raw generations; implementer then spec-test | Partial primitive diagnostics only; any direct writer or lost field blocks CLI enablement/delivery |
| Contract parity P0 | Full key inventory, genuine adapter parity and each owned code/exit; spec-test | Future domain codes owned by their features, not falsely counted covered here |
| Security P0 | Injected foreign-identity/ACL denial, hostile endpoint and secret redaction tests; real second-user run pending release evidence; native implementer, independent security review of OS adapters | No vulnerability audit here; automated authentication/secret failure blocks delivery; missing second-account native proof does not |
| Native compatibility P1 | Mandatory macOS real new-app/process/window/M1 latency; other native checks pending release evidence; platform operators | MacOS native missing/failure blocks Implemented; Windows/Linux/old-binary/second-account native missing never blocks steps/completion/dev merge and is never called PASS |
| Evidence integrity | Versioned binary hashes, transcripts and AC assertion mapping; verifier | Planned tests or tool versions never certify runtime success |

Expected artifacts after implementation: `proofs/008-settings-cli/<os>/commands.json`, stdout/stderr transcripts, `acceptance-matrix.json`, `store-generations.json`, `native-observations.md` and latency samples; these paths are proposed outputs, not existing evidence. QE defines sufficiency; independent review judges design, security review inspects native threat boundaries, spec-test executes and records runtime proof. No claim of insertion attempted/verified comes from configuration receipts.

## Risks & Considerations

| Spec-review finding (original 2026-10-12 plus refreshed review) | Decision/test/risk disposition |
|---|---|
| Windows GUI-subsystem shell waiting | Console-subsystem same exe, preserve settings/headless handles, GUI FreeConsole; compilation/PE gate; cmd/PowerShell/GUI native checks pending release evidence |
| Private channel and older instance detection | Authenticated new IPC plus non-intrusive legacy probes; detected old owner -> unavailable/no write. No worker/spool/reservation/replay; deterministic probe tests required, real old-binary proof pending release evidence |
| All writers and cached store ordering | Steps 2–4 precede CLI writes; no settings plugin autosave/exit flush, inventory closure gate |
| Retired clipboard-window example in spec Clarifications | 008 invalid-value parity uses existing paste_method=invalid_method; no clipboard_recovery_* key or recovery-window API is planned. Feature 007 extends output with smart_paste and owns platform validation/migration, six honest outcomes and native compatibility evidence |
| Store isolation and missing native harness | Injected paths for unit/integration; test-only harness binary location/portable store, Rust commands explicit |
| Salvage could write during read | In-memory read normalization only; preserve revision, refuse unrecoverable continuity on mutation |
| List owner absent in 008 | Error names owning domain plus unavailable-in-this-build; help lists only registered domains |
| Invalid bulk dry-run unspecified | Full validation; invalid → validation_failed/2, valid → preview/0 with zero effects |
| 2 s read versus 5 s timeout | Every successful M1 release read ≤2 s; hung/deadline failure measured separately, never success; no invented Windows/Linux budget |
| Revision bootstrap and downgrade | Missing legacy field→0, first commit→1; never reset valid revision. Downgrade/old writers are unsupported concurrent owners and block CLI writes; after a downgrade, prior expected revisions must not be reused until state reconciliation. Backups preserve raw files; no monotonicity promise across external edits or unsupported downgrade |
| Mixed legacy flags | usage_error/2 except debug/json; regression matrix for old-only invocations |
| i18n mapping | Stable code+field translation keys; English backend parity, translated generic fallback |
| Runtime constitution evidence | Required macOS native and automated builds/tests; Windows/Linux/old-binary/second-account exact native checks pending release evidence; availability is not proof |
| Refreshed spec: writer_inventory | Exhaustive startup/internal/UI writer scan in Steps 2–4; real no-op and migration tests; no snapshot overwrite bypass |
| Refreshed spec: timeout_outcome | Test deadline before/after commit, lost response and late reply; outcome unknown, reread, no fallback/replay |
| Refreshed spec: native_evidence_scope | Exact pending rows above; no extra old-version replay requirement; only macOS native evidence required before Implemented |
| Refreshed spec: test_commands | Explicit Cargo unit/process/integration/target commands and injected store/clock/probe/transport/native-effect seams; frontend proof insufficient |
| Refreshed spec: validation_compatibility | Formerly coerced UI values reject consistently with CLI; legacy persisted salvage/migration stays a separate in-memory read/startup-commit path |
| Prepared effects / post-commit failure (refreshed review) | Prepare compensation and truthful committed error, reconcile before next affected mutation; inject failure paths |
| Revision continuity / overflow (refreshed review) | Valid metadata kept through salvage; corrupt revision blocks writes, overflow refused before effects |
| Deadline/redaction all error paths (refreshed review) | Monotonic total timeout, bounded frames and no argv dumps; sentinel tests in stdout/stderr/logs |

No new mockups or Penflow source is generated: 008 is nonvisual and has no Screens section. The program's approved one-pass UI contract preparation before spec-ship remains a later program gate; existing layout is unchanged by this plan. Future owner notes remain separate: 006 reason indicator ≥4 s delegates to 010 bubble when shipped; 007 restores only while changeCount still matches publication; 010 Copy writes new clipboard content, so that guard abandons smart paste restoration without a window-cancellation API; 006/009 ≤1 s activation waits must stay off native main thread; 005/006/009/010 need an early macOS nonactivating surface spike; history conservation must account for WAV-gated save in actions.rs. These are not extra 008 implementation requirements.

## Next Action

After complete+ready independent plan review and READY progression implement, the next command is `$spec-implement 008-settings-cli --model=gpt-6.1-sol`. This plan command does not execute it, commit or push. Resume Step 1 against this revised minimal scope, refresh its checkpoint/mappings during implementation, then follow Steps 2–7. Automated and macOS acceptance gates remain required; pending release evidence never blocks steps, integration-branch feature completion or dev merge.
