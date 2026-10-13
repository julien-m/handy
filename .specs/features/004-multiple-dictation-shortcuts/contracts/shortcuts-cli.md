# Shortcuts CLI Contract

004 extends the 008 Domain framework, OperationRequest and SettingsMutation; no separate transport or renderer. Consult [the plan](../plan.md) for transaction ordering, lifecycle leases and native evidence.

| Operation | Arguments | Policy / data |
|---|---|---|
| `handy settings shortcuts list` | optional ACTION | Offline read, no mutation; actions collection or action projection |
| `handy settings shortcuts add` | ACTION SHORTCUT | Live-only; two dictation actions; maximum four |
| `handy settings shortcuts remove` | ACTION SHORTCUT | Live-only; normalize lookup; last entry refused, removing first promotes next |
| `handy settings shortcuts reset` | ACTION | Live-only; single validated default; unchanged other action policy |

Success: `{ok:true,revision:N,data:{action_id,shortcuts:[raw strings],default_shortcut,keyboard_implementation}}`; collection list wraps `data.actions`. Errors: `{ok:false,error:{code,message,details:{...}}}`. One object/newline on stdout, diagnostics on stderr. Expected revision, JSON placements and preview use existing 008 framework. Pure offline preview has no OS feasibility claim. No replace, import or reorder subcommand is added.

| Code | Exit | Details / state |
|---|---|---|
| success | 0 | Actual snapshot revision; no-op/read/preview do not increment |
| shortcut_conflict | 2 | owning_action, owning_shortcut, kind; unchanged storage |
| limit_exceeded | 2 | action_id, limit:4; unchanged |
| validation_failed | 2 | last removal, syntax, unsupported action operation or native refusal; reason/action_id; OS reason when applicable |
| usage_error | 2 | Invalid argv through 008 parser boundary |
| revision_conflict | 3 | current_revision; unchanged |
| not_found | 4 | Unknown action or remove entry |
| unavailable | 5 | No authenticated owner, busy lifecycle; submitted timeout outcome unknown |
| persistence_failed | 1 | Durable old state retained; compensation verified or degraded failure made explicit |
| internal_error | 1 | Failed compensation or committed live failure with truthful outcome/current_revision |

SHORTCUT uses active backend syntax; stored first binding remains in legacy current_binding. Reset and every mutation use the same checker, registry preparation/compensation and settings.lock commit path as UI. Cancel remains one dynamic binding with current Linux limits. 005 registers its prompt actions later with 004's action-agnostic checker.
