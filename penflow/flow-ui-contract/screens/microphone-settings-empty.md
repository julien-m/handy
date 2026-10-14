<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: microphone-settings-empty

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
      "id": "microphone-settings-empty.add",
      "kind": "state_change",
      "node": "microphone-settings-empty.add",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "clamshell"
      },
      "event": "change",
      "id": "microphone-settings-empty.clamshell.select",
      "kind": "state_change",
      "node": "microphone-settings-empty.clamshell.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-empty.refresh",
      "kind": "state_change",
      "node": "microphone-settings-empty.refresh",
      "success_transition": "microphone-settings-empty.refresh.refreshing",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "microphone-settings-empty.sound"
      ],
      "id": "microphone-settings-empty",
      "role": "screen",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty",
      "test_id": "microphone-settings-empty-screen"
    },
    {
      "action": "microphone-settings-empty.add",
      "binding": "microphone_priority.new_choice",
      "id": "microphone-settings-empty.add",
      "role": "select",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.add",
      "test_id": "microphone-settings-empty-add",
      "text": "Add microphone…"
    },
    {
      "children": [
        "microphone-settings-empty.clamshell.title",
        "microphone-settings-empty.clamshell.select"
      ],
      "id": "microphone-settings-empty.clamshell",
      "role": "container",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.clamshell",
      "test_id": "microphone-settings-empty-clamshell"
    },
    {
      "action": "microphone-settings-empty.clamshell.select",
      "binding": "microphone_priority.clamshell",
      "id": "microphone-settings-empty.clamshell.select",
      "role": "select",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.clamshell.select",
      "test_id": "microphone-settings-empty-clamshell-select",
      "text": "None"
    },
    {
      "id": "microphone-settings-empty.clamshell.title",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.clamshell.title",
      "test_id": "microphone-settings-empty-clamshell-title",
      "text": "Closed-lid microphone"
    },
    {
      "children": [
        "microphone-settings-empty.empty.title",
        "microphone-settings-empty.empty.body"
      ],
      "id": "microphone-settings-empty.empty",
      "role": "container",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.empty",
      "test_id": "microphone-settings-empty-empty"
    },
    {
      "id": "microphone-settings-empty.empty.body",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.empty.body",
      "test_id": "microphone-settings-empty-empty-body",
      "text": "Handy uses the system default microphone until you add one."
    },
    {
      "id": "microphone-settings-empty.empty.title",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.empty.title",
      "test_id": "microphone-settings-empty-empty-title",
      "text": "No preferred microphone"
    },
    {
      "children": [
        "microphone-settings-empty.in-use.title",
        "microphone-settings-empty.in-use.device"
      ],
      "id": "microphone-settings-empty.in-use",
      "role": "container",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.in-use",
      "test_id": "microphone-settings-empty-in-use"
    },
    {
      "id": "microphone-settings-empty.in-use.device",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.in-use.device",
      "test_id": "microphone-settings-empty-in-use-device",
      "text": "Not used yet since launch"
    },
    {
      "id": "microphone-settings-empty.in-use.title",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.in-use.title",
      "test_id": "microphone-settings-empty-in-use-title",
      "text": "In use now"
    },
    {
      "children": [
        "microphone-settings-empty.priority.title",
        "microphone-settings-empty.empty",
        "microphone-settings-empty.add",
        "microphone-settings-empty.refresh"
      ],
      "id": "microphone-settings-empty.priority",
      "role": "container",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.priority",
      "test_id": "microphone-settings-empty-priority"
    },
    {
      "id": "microphone-settings-empty.priority.title",
      "role": "text",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.priority.title",
      "test_id": "microphone-settings-empty-priority-title",
      "text": "Microphone priority"
    },
    {
      "action": "microphone-settings-empty.refresh",
      "id": "microphone-settings-empty.refresh",
      "role": "button",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.refresh",
      "test_id": "microphone-settings-empty-refresh",
      "text": "Refresh availability"
    },
    {
      "children": [
        "microphone-settings-empty.priority",
        "microphone-settings-empty.in-use",
        "microphone-settings-empty.clamshell"
      ],
      "id": "microphone-settings-empty.sound",
      "role": "container",
      "screen_id": "microphone-settings-empty",
      "semantic_id": "microphone-settings-empty.sound",
      "test_id": "microphone-settings-empty-sound",
      "text": "Sound"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "microphone-priority",
    "id": "microphone-settings-empty",
    "root_node": "microphone-settings-empty",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "refreshing"
      }
    ],
    "surface": "desktop",
    "title": "Microphone priority — empty",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
