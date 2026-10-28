<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: settings-write-errors

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "binding_ref": {
        "access": "write",
        "entity": "history",
        "field": "limit"
      },
      "event": "change",
      "id": "settings-write-errors.history-limit.input",
      "kind": "state_change",
      "node": "settings-write-errors.history-limit.input",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "settings-write-errors.reload",
      "kind": "state_change",
      "node": "settings-write-errors.reload",
      "success_transition": "settings-write-errors.reload.reloaded",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "settings-write-errors.conflict",
        "settings-write-errors.history",
        "settings-write-errors.save-failed"
      ],
      "id": "settings-write-errors",
      "role": "screen",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors",
      "test_id": "settings-write-errors-screen"
    },
    {
      "children": [
        "settings-write-errors.conflict.title",
        "settings-write-errors.conflict.reload-failed",
        "settings-write-errors.reload"
      ],
      "id": "settings-write-errors.conflict",
      "role": "alert",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.conflict",
      "test_id": "settings-write-errors-conflict"
    },
    {
      "id": "settings-write-errors.conflict.reload-failed",
      "role": "text",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.conflict.reload-failed",
      "states": [
        "reload-failed"
      ],
      "test_id": "settings-write-errors-conflict-reload-failed",
      "text": "The settings could not be reloaded. Try again."
    },
    {
      "id": "settings-write-errors.conflict.title",
      "role": "text",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.conflict.title",
      "test_id": "settings-write-errors-conflict-title",
      "text": "Settings changed elsewhere"
    },
    {
      "children": [
        "settings-write-errors.history-limit"
      ],
      "id": "settings-write-errors.history",
      "role": "container",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.history",
      "test_id": "settings-write-errors-history",
      "text": "History"
    },
    {
      "children": [
        "settings-write-errors.history-limit.title",
        "settings-write-errors.history-limit.input",
        "settings-write-errors.history-limit.error"
      ],
      "id": "settings-write-errors.history-limit",
      "role": "container",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.history-limit",
      "test_id": "settings-write-errors-history-limit"
    },
    {
      "id": "settings-write-errors.history-limit.error",
      "role": "text",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.history-limit.error",
      "test_id": "settings-write-errors-history-limit-error",
      "text": "Enter a whole number from 0 to 1000."
    },
    {
      "action": "settings-write-errors.history-limit.input",
      "binding": "history.limit",
      "id": "settings-write-errors.history-limit.input",
      "role": "input",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.history-limit.input",
      "test_id": "settings-write-errors-history-limit-input",
      "text": "-5"
    },
    {
      "id": "settings-write-errors.history-limit.title",
      "role": "text",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.history-limit.title",
      "test_id": "settings-write-errors-history-limit-title",
      "text": "History Limit"
    },
    {
      "action": "settings-write-errors.reload",
      "id": "settings-write-errors.reload",
      "role": "button",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.reload",
      "test_id": "settings-write-errors-reload",
      "text": "Reload"
    },
    {
      "children": [
        "settings-write-errors.save-failed.title"
      ],
      "id": "settings-write-errors.save-failed",
      "role": "alert",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.save-failed",
      "test_id": "settings-write-errors-save-failed"
    },
    {
      "id": "settings-write-errors.save-failed.title",
      "role": "text",
      "screen_id": "settings-write-errors",
      "semantic_id": "settings-write-errors.save-failed.title",
      "test_id": "settings-write-errors-save-failed-title",
      "text": "Couldn't save settings: the settings file could not be written. Nothing was changed."
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "settings-cli",
    "id": "settings-write-errors",
    "root_node": "settings-write-errors",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "reloaded"
      }
    ],
    "surface": "desktop",
    "title": "Settings write errors",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
