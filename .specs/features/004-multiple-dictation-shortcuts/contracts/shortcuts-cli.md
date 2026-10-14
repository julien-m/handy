# Shortcuts CLI Contract

004 extends the 008 Domain framework, OperationRequest and settings service; no separate transport or renderer. Consult [the plan](../plan.md) for transaction ordering, lifecycle leases and native evidence. Implemented by `src-tauri/src/settings_service/domains/shortcuts.rs` (domain), `src-tauri/src/shortcut/live_change.rs` (native preparation shared with the settings window) and `src-tauri/src/transcription_coordinator/quiescence.rs` (lease).

| Operation | Arguments | Policy / data |
|---|---|---|
| `handy settings shortcuts list` | optional ACTION | Offline read, no mutation; `data.actions` collection, or one action projection |
| `handy settings shortcuts add` | ACTION SHORTCUT `[--expected-revision N] [--dry-run]` | Live-only; two dictation actions; maximum four |
| `handy settings shortcuts remove` | ACTION SHORTCUT `[--expected-revision N] [--dry-run]` | Live-only; SHORTCUT matches the stored entry as typed or with the same normalized combination (any modifier order); last entry refused, removing the first promotes the next |
| `handy settings shortcuts reset` | ACTION `[--expected-revision N] [--dry-run]` | Live-only; single validated default; a reset already at the default is a no-op |

Action projection: `{action_id, shortcuts:[raw strings], default_shortcut, keyboard_implementation}`; ACTION is a known action id (`transcribe`, `transcribe_with_post_process`, `cancel`, ...). Success: `{ok:true,revision:N,data:<projection>}`; `list` without ACTION wraps `data.actions`. A dry run answers `{ok:true,revision:N,data:{dry_run:true,registration_checked:false,changes:<projection>}}`: it validates exactly like the change (limits, conflicts, syntax, expected revision) but writes nothing and never asks the system, so it never certifies that the system will accept the shortcut. Errors: `{ok:false,error:{code,message,details:{...}}}`. One object/newline on stdout with `--json`, diagnostics on stderr; without `--json`, stdout shows `action: first, second (default: shortcut)` per action and stderr the error with its reason, action, owner or limit. No replace, import or reorder subcommand: replacing is `add` then `remove` (two revisions; at four, `remove` first).

| Code | Exit | Details / state |
|---|---|---|
| success | 0 | Actual snapshot revision; no-op/read/preview do not increment; one revision per change |
| shortcut_conflict | 2 | `action_id`, `owning_action`, `owning_shortcut` (stored text), `kind` (`duplicate` or `modifier_overlap`); unchanged storage |
| limit_exceeded | 2 | `action_id`, `limit: 4`; unchanged |
| validation_failed | 2 | `action_id` and `reason`: `last_shortcut`, `alternatives_unsupported` (cancel and other single-shortcut actions), `invalid_shortcut` (parser refusal of the active implementation; its message is not copied, it can quote the candidate), `registration_refused` (the system refused it; the system message stays in the instance log) |
| usage_error | 2 | Invalid argv through the 008 parser boundary |
| revision_conflict | 3 | `current_revision`; unchanged |
| not_found | 4 | Unknown action (no details, the input is not echoed) or remove entry (`action_id`) |
| unavailable | 5 | `reason: instance_required` (no running instance; nothing written), `reason: shortcut_session_busy` and `field: bindings` (a dictation session owns the shortcuts: recording, processing, a held or remembered shortcut), an older Handy or a busy owner (008), or a submitted timeout with `outcome: unknown` |
| persistence_failed | 1 | Durable old state retained; the prepared registrations are undone |
| internal_error | 1 | Committed live failure with `outcome: committed` and `current_revision` (008) |

SHORTCUT uses active backend syntax; the stored first binding remains in the legacy `current_binding`. Every change and the settings window use the same checker, the same registry preparation and compensation (`settings_service::native_changes` → `shortcut::live_change::bindings`) and the same ordered write of the 008 service. The settings window reports a busy session and other refusals of the service through the typed `settings_refused` edit error. Cancel remains one dynamic binding with current Linux limits. 005 registers its prompt actions later with 004's action-agnostic checker.
