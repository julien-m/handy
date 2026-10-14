<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: microphone-settings-error

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "new_choice"
      },
      "event": "change",
      "id": "microphone-settings-error.add",
      "kind": "state_change",
      "node": "microphone-settings-error.add",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.move-down",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.move-down",
      "success_transition": "microphone-settings-error.choice.move-down.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.move-up",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.move-up",
      "success_transition": "microphone-settings-error.choice.move-up.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.remove",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.remove",
      "success_transition": "microphone-settings-error.choice.remove.removed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-dirty",
      "id": "microphone-settings-error.discard",
      "kind": "state_change",
      "node": "microphone-settings-error.discard",
      "success_transition": "microphone-settings-error.discard.discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.reload",
      "kind": "state_change",
      "node": "microphone-settings-error.reload",
      "success_transition": "microphone-settings-error.reload.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.retry",
      "kind": "state_change",
      "node": "microphone-settings-error.retry",
      "success_transition": "microphone-settings-error.retry.refreshing",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "microphone-settings-error.save",
      "kind": "state_change",
      "node": "microphone-settings-error.save",
      "success_transition": "microphone-settings-error.save.saved",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "microphone-settings-error.conflict",
        "microphone-settings-error.sound",
        "microphone-settings-error.draft"
      ],
      "id": "microphone-settings-error",
      "role": "screen",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error",
      "test_id": "microphone-settings-error-screen"
    },
    {
      "action": "microphone-settings-error.add",
      "binding": "microphone_priority.new_choice",
      "id": "microphone-settings-error.add",
      "role": "select",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.add",
      "test_id": "microphone-settings-error-add",
      "text": "USB headset"
    },
    {
      "id": "microphone-settings-error.add-error",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.add-error",
      "test_id": "microphone-settings-error-add-error",
      "text": "USB headset is already in the list."
    },
    {
      "children": [
        "microphone-settings-error.choice.rank",
        "microphone-settings-error.choice.name",
        "microphone-settings-error.choice.status",
        "microphone-settings-error.choice.move-up",
        "microphone-settings-error.choice.move-down",
        "microphone-settings-error.choice.remove"
      ],
      "id": "microphone-settings-error.choice",
      "role": "container",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice",
      "test_id": "microphone-settings-error-choice"
    },
    {
      "action": "microphone-settings-error.choice.move-down",
      "id": "microphone-settings-error.choice.move-down",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.move-down",
      "test_id": "microphone-settings-error-choice-move-down"
    },
    {
      "action": "microphone-settings-error.choice.move-up",
      "id": "microphone-settings-error.choice.move-up",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.move-up",
      "test_id": "microphone-settings-error-choice-move-up"
    },
    {
      "id": "microphone-settings-error.choice.name",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.name",
      "test_id": "microphone-settings-error-choice-name",
      "text": "USB headset"
    },
    {
      "id": "microphone-settings-error.choice.rank",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.rank",
      "test_id": "microphone-settings-error-choice-rank",
      "text": "1"
    },
    {
      "action": "microphone-settings-error.choice.remove",
      "id": "microphone-settings-error.choice.remove",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.remove",
      "test_id": "microphone-settings-error-choice-remove"
    },
    {
      "id": "microphone-settings-error.choice.status",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choice.status",
      "test_id": "microphone-settings-error-choice-status",
      "text": "Unknown"
    },
    {
      "children": [
        "microphone-settings-error.choice"
      ],
      "id": "microphone-settings-error.choices",
      "repeat_model": {
        "collection": "microphone_priority.choices",
        "instance_id_template": "microphone-{name}",
        "item_key": "name",
        "item_model": "microphone_choice"
      },
      "role": "list",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.choices",
      "test_id": "microphone-settings-error-choices"
    },
    {
      "children": [
        "microphone-settings-error.conflict.title",
        "microphone-settings-error.reload"
      ],
      "id": "microphone-settings-error.conflict",
      "role": "alert",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.conflict",
      "test_id": "microphone-settings-error-conflict"
    },
    {
      "id": "microphone-settings-error.conflict.title",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.conflict.title",
      "test_id": "microphone-settings-error-conflict-title",
      "text": "Settings changed elsewhere"
    },
    {
      "action": "microphone-settings-error.discard",
      "id": "microphone-settings-error.discard",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.discard",
      "test_id": "microphone-settings-error-discard",
      "text": "Discard"
    },
    {
      "children": [
        "microphone-settings-error.draft.note",
        "microphone-settings-error.discard",
        "microphone-settings-error.save"
      ],
      "id": "microphone-settings-error.draft",
      "role": "container",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.draft",
      "test_id": "microphone-settings-error-draft"
    },
    {
      "id": "microphone-settings-error.draft.note",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.draft.note",
      "test_id": "microphone-settings-error-draft-note",
      "text": "Unsaved changes. The next recording uses the saved order until you save."
    },
    {
      "children": [
        "microphone-settings-error.priority.title",
        "microphone-settings-error.choices",
        "microphone-settings-error.refresh-error",
        "microphone-settings-error.retry",
        "microphone-settings-error.add",
        "microphone-settings-error.add-error"
      ],
      "id": "microphone-settings-error.priority",
      "role": "container",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.priority",
      "test_id": "microphone-settings-error-priority"
    },
    {
      "id": "microphone-settings-error.priority.title",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.priority.title",
      "test_id": "microphone-settings-error-priority-title",
      "text": "Microphone priority"
    },
    {
      "id": "microphone-settings-error.refresh-error",
      "role": "text",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.refresh-error",
      "test_id": "microphone-settings-error-refresh-error",
      "text": "Couldn't check devices"
    },
    {
      "action": "microphone-settings-error.reload",
      "id": "microphone-settings-error.reload",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.reload",
      "test_id": "microphone-settings-error-reload",
      "text": "Reload"
    },
    {
      "action": "microphone-settings-error.retry",
      "id": "microphone-settings-error.retry",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.retry",
      "test_id": "microphone-settings-error-retry",
      "text": "Retry"
    },
    {
      "action": "microphone-settings-error.save",
      "id": "microphone-settings-error.save",
      "role": "button",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.save",
      "test_id": "microphone-settings-error-save",
      "text": "Save"
    },
    {
      "children": [
        "microphone-settings-error.priority"
      ],
      "id": "microphone-settings-error.sound",
      "role": "container",
      "screen_id": "microphone-settings-error",
      "semantic_id": "microphone-settings-error.sound",
      "test_id": "microphone-settings-error-sound",
      "text": "Sound"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "microphone-priority",
    "id": "microphone-settings-error",
    "root_node": "microphone-settings-error",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "discarded"
      },
      {
        "id": "refreshing"
      },
      {
        "id": "reloaded"
      },
      {
        "id": "removed"
      },
      {
        "id": "reordered"
      },
      {
        "id": "saved"
      }
    ],
    "surface": "desktop",
    "title": "Microphone priority — errors",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
