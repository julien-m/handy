# Settings CLI wire contract

This is the foundation contract. Persistence, actual Tauri command parity and executable CLI routing are wired in subsequent steps; this document does not claim those paths already work.

Inspect [the functional source](../spec.md) and [typed protocol](../../../../src-tauri/src/settings_service/protocol.rs). Generate [the JSON schema](settings-cli.schema.json) from `contract_schema()`; its finite metadata comes from the explicit descriptor inventory, not a raw store object. This is JSON Schema, not OpenAPI.

| Operation | Public contract |
| --- | --- |
| `settings get [KEY]` | All 63 known fields or one recognized field; API keys become provider `{present: bool}` entries |
| `settings keys` | Key, canonical value type, nullable flag, allowed values/range, read-only/secret flags, owner and runtime effect category |
| `settings set KEY VALUE` | One scalar value validated by descriptor; list fields name their owner command, readonly/secret fields direct the user to settings UI |
| Domains | `DomainExtension` supplies metadata; actual domain registration/help, file/stdin input and dry-run orchestration are Step 6 |

The scalar CLI grammar accepts exact booleans `true`/`false`, unsigned integers, finite bounded numbers, canonical enum strings and literal strings. Nullable fields accept exact `null` to clear. Whitespace in meaningful strings is preserved. UI device `default`/`Default` sentinels normalize to null before the shared validator. Disk-only log-level integers/uppercase and overlay-position `none` aliases are rejected as mutation input. Prompt selection requires current membership. Build policy may disable updater mutation. Widget step increments are not imposed.

Both pure validation ports use fixed English categories/reasons; the existing Tauri writers are migrated in Step 4. Numeric/channel native availability and prepared runtime effects are checked by their later owning adapters.

Success is `{"ok":true,"revision":N,"data":…}`. Failure is `{"ok":false,"error":{"code":C,"message":M,"details":{…}}}`. Details always remain an object; recognized fields/issues, current revision or an outcome (`unknown` after a timeout or unconfirmed durability, `committed` when a live effect failed after a durable write) are the only admitted details. Unknown key/candidate/parser text never enters errors. Revision is an unsigned 64-bit integer independent of settings schema version.

| Category | Exit |
| --- | --- |
| `usage_error`, `validation_failed`, `shortcut_conflict`, `limit_exceeded` | 2 |
| `revision_conflict` | 3 |
| `not_found` | 4 |
| `unavailable` | 5 |
| `persistence_failed`, `internal_error` | 1 |
| Success | 0 |

Owned categories are usage, validation, revision conflict, not found, unavailable, persistence failure and internal error. Shortcut conflict and limit exceeded retain the documented extension mapping; no current domain-handler behavior is certified by their DTO definitions.

Request DTOs carry protocol version, request/store identity and a typed get/keys/set/domain operation. Peer authentication, frame bounds, deadline enforcement, single-envelope rendering and unknown-outcome handling are Step 5 obligations. No network listener or MCP server is introduced.
