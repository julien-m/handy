<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: microphone-settings

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
      "id": "microphone-settings.add",
      "kind": "state_change",
      "node": "microphone-settings.add",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.move-down",
      "kind": "state_change",
      "node": "microphone-settings.choice.move-down",
      "success_transition": "microphone-settings.choice.move-down.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.move-up",
      "kind": "state_change",
      "node": "microphone-settings.choice.move-up",
      "success_transition": "microphone-settings.choice.move-up.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.remove",
      "kind": "state_change",
      "node": "microphone-settings.choice.remove",
      "success_transition": "microphone-settings.choice.remove.removed",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "clamshell"
      },
      "event": "change",
      "id": "microphone-settings.clamshell.select",
      "kind": "state_change",
      "node": "microphone-settings.clamshell.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "draft-dirty",
      "id": "microphone-settings.discard",
      "kind": "state_change",
      "node": "microphone-settings.discard",
      "success_transition": "microphone-settings.discard.discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.refresh",
      "kind": "state_change",
      "node": "microphone-settings.refresh",
      "success_transition": "microphone-settings.refresh.refreshing",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "microphone-settings.save",
      "kind": "state_change",
      "node": "microphone-settings.save",
      "success_transition": "microphone-settings.save.saved",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "microphone-settings.sound",
        "microphone-settings.draft"
      ],
      "id": "microphone-settings",
      "role": "screen",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings",
      "test_id": "microphone-settings-screen"
    },
    {
      "action": "microphone-settings.add",
      "binding": "microphone_priority.new_choice",
      "id": "microphone-settings.add",
      "role": "select",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.add",
      "test_id": "microphone-settings-add",
      "text": "Add microphone…"
    },
    {
      "id": "microphone-settings.checked",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.checked",
      "test_id": "microphone-settings-checked",
      "text": "Checked 3 s ago"
    },
    {
      "children": [
        "microphone-settings.choice.rank",
        "microphone-settings.choice.name",
        "microphone-settings.choice.status",
        "microphone-settings.choice.move-up",
        "microphone-settings.choice.move-down",
        "microphone-settings.choice.remove"
      ],
      "id": "microphone-settings.choice",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice",
      "test_id": "microphone-settings-choice"
    },
    {
      "action": "microphone-settings.choice.move-down",
      "id": "microphone-settings.choice.move-down",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.move-down",
      "test_id": "microphone-settings-choice-move-down"
    },
    {
      "action": "microphone-settings.choice.move-up",
      "id": "microphone-settings.choice.move-up",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.move-up",
      "test_id": "microphone-settings-choice-move-up"
    },
    {
      "id": "microphone-settings.choice.name",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.name",
      "test_id": "microphone-settings-choice-name",
      "text": "USB headset"
    },
    {
      "id": "microphone-settings.choice.rank",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.rank",
      "test_id": "microphone-settings-choice-rank",
      "text": "1"
    },
    {
      "action": "microphone-settings.choice.remove",
      "id": "microphone-settings.choice.remove",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.remove",
      "test_id": "microphone-settings-choice-remove"
    },
    {
      "id": "microphone-settings.choice.status",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choice.status",
      "test_id": "microphone-settings-choice-status",
      "text": "Unavailable"
    },
    {
      "children": [
        "microphone-settings.choice"
      ],
      "id": "microphone-settings.choices",
      "repeat_model": {
        "collection": "microphone_priority.choices",
        "instance_id_template": "microphone-{name}",
        "item_key": "name",
        "item_model": "microphone_choice"
      },
      "role": "list",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.choices",
      "test_id": "microphone-settings-choices"
    },
    {
      "children": [
        "microphone-settings.clamshell.title",
        "microphone-settings.clamshell.select"
      ],
      "id": "microphone-settings.clamshell",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.clamshell",
      "test_id": "microphone-settings-clamshell"
    },
    {
      "action": "microphone-settings.clamshell.select",
      "binding": "microphone_priority.clamshell",
      "id": "microphone-settings.clamshell.select",
      "role": "select",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.clamshell.select",
      "test_id": "microphone-settings-clamshell-select",
      "text": "Studio Display Microphone"
    },
    {
      "id": "microphone-settings.clamshell.title",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.clamshell.title",
      "test_id": "microphone-settings-clamshell-title",
      "text": "Closed-lid microphone"
    },
    {
      "action": "microphone-settings.discard",
      "id": "microphone-settings.discard",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.discard",
      "test_id": "microphone-settings-discard",
      "text": "Discard"
    },
    {
      "children": [
        "microphone-settings.draft.note",
        "microphone-settings.discard",
        "microphone-settings.save"
      ],
      "id": "microphone-settings.draft",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.draft",
      "test_id": "microphone-settings-draft"
    },
    {
      "id": "microphone-settings.draft.note",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.draft.note",
      "test_id": "microphone-settings-draft-note",
      "text": "Unsaved changes. The next recording uses the saved order until you save."
    },
    {
      "children": [
        "microphone-settings.in-use.title",
        "microphone-settings.in-use.device",
        "microphone-settings.in-use.reason"
      ],
      "id": "microphone-settings.in-use",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.in-use",
      "test_id": "microphone-settings-in-use"
    },
    {
      "id": "microphone-settings.in-use.device",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.in-use.device",
      "test_id": "microphone-settings-in-use-device",
      "text": "MacBook Pro Microphone"
    },
    {
      "id": "microphone-settings.in-use.reason",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.in-use.reason",
      "test_id": "microphone-settings-in-use-reason",
      "text": "Fallback · USB headset unavailable"
    },
    {
      "id": "microphone-settings.in-use.title",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.in-use.title",
      "test_id": "microphone-settings-in-use-title",
      "text": "In use now"
    },
    {
      "children": [
        "microphone-settings.priority.title",
        "microphone-settings.choices",
        "microphone-settings.add",
        "microphone-settings.refresh",
        "microphone-settings.checked"
      ],
      "id": "microphone-settings.priority",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.priority",
      "test_id": "microphone-settings-priority"
    },
    {
      "id": "microphone-settings.priority.title",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.priority.title",
      "test_id": "microphone-settings-priority-title",
      "text": "Microphone priority"
    },
    {
      "action": "microphone-settings.refresh",
      "id": "microphone-settings.refresh",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.refresh",
      "test_id": "microphone-settings-refresh",
      "text": "Refresh availability"
    },
    {
      "action": "microphone-settings.save",
      "id": "microphone-settings.save",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.save",
      "test_id": "microphone-settings-save",
      "text": "Save"
    },
    {
      "children": [
        "microphone-settings.priority",
        "microphone-settings.in-use",
        "microphone-settings.clamshell"
      ],
      "id": "microphone-settings.sound",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings.sound",
      "test_id": "microphone-settings-sound",
      "text": "Sound"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "microphone-priority",
    "id": "microphone-settings",
    "root_node": "microphone-settings",
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
        "id": "removed"
      },
      {
        "id": "reordered"
      },
      {
        "id": "saved"
      },
      {
        "id": "saving"
      }
    ],
    "surface": "desktop",
    "title": "Microphone priority",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
