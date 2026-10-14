# Microphones domain contract (feature 001)

Executable since Step 5. Inspect [the functional source](../spec.md), [the domain](../../../../src-tauri/src/settings_service/domains/microphones.rs), [the status DTOs](../../../../src-tauri/src/microphones/status.rs) and [the 008 envelope contract](../../008-settings-cli/contracts/settings-cli.md). Every reply is the 008 envelope; with `--json`, stdout holds exactly that one line.

| Operation | Contract |
| --- | --- |
| `handy settings microphones list` | Live operation (D2 of 006): answered by the running Handy's audio manager. Data below. Without a running Handy: `unavailable`, exit 5, `details` `{}`; nothing is enumerated or fabricated by the CLI process. |
| `handy settings microphones set-priority [NAME]...` | The complete order, first choice first, offline or routed. Zero names: the empty order (system default). `--` before a name starting with a hyphen. Data `{"microphone_priority": [...]}`, revision of the commit (unchanged for an identical order). |
| `... set-priority ... --expected-revision N` | `revision_conflict`, exit 3, `details.current_revision`, nothing written, unless the settings are at revision N. |
| `... set-priority ... --dry-run` | `{"dry_run": true, "changes": {"microphone_priority": [...]}}` at the current revision; nothing written, no live effect. |
| Refused names | `validation_failed`, exit 2, `details.field = "microphone_priority"`, `details.items` `[{index, code, message}]` with `code` `microphone_name_empty` (empty or whitespace only) or `microphone_name_duplicate` (exact repeat); no name is echoed. |
| `handy settings set selected_microphone …` / `set microphone_priority …` | `validation_failed`, exit 2, `details.field`, `details.reason = "microphones_domain"`, message naming `handy settings microphones set-priority`. |
| `handy settings get microphone_priority` | The saved order, offline or routed. `selected_microphone` stays readable (read-only, legacy). |

## `list` data

```json
{
  "configured_order": ["USB headset", "MacBook Pro Microphone"],
  "availability": [
    {"name": "USB headset", "status": "unavailable"},
    {"name": "MacBook Pro Microphone", "status": "available"}
  ],
  "availability_observed_at": "2026-10-14T10:00:00.000000+00:00",
  "effective_input": {"scope": "current", "name": "MacBook Pro Microphone", "reason": "fallback", "capture_generation": 12},
  "warning": null
}
```

- `status`: `available`, `unavailable` (a complete enumeration did not find it), `ambiguous` (several present devices share the name), `unknown` (enumeration failed, timed out after 3 s, or could not read every name).
- `effective_input.scope`: `current` (a recording runs or the always-on stream is open), `last_recording` (the last recording since launch), `not_used_yet` (then `name`, `reason` and `capture_generation` are null). `reason`: `first_choice`, `fallback`, `clamshell`, `system_default`.
- `warning`: null or `enumeration_failed`.
- The configured order belongs to the envelope's revision; `availability_observed_at` is the time of a separate device observation. They are two clocks, not one atomic view of changing hardware.

The settings window reads the same projection with the typed command `get_microphone_status` and saves with `set_microphone_priority(order, expected_revision)` (same validation and transaction).
