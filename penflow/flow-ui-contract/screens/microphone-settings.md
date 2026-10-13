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
        "entity": "microphone",
        "field": "new_choice"
      },
      "event": "change",
      "id": "choose-microphone",
      "kind": "state_change",
      "node": "microphone-choose",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "valid-choice",
      "id": "move-down",
      "kind": "state_change",
      "node": "microphone-move-down",
      "success_transition": "move-down-complete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "valid-choice",
      "id": "move-up",
      "kind": "state_change",
      "node": "microphone-move-up",
      "success_transition": "move-up-complete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "valid-choice",
      "id": "remove",
      "kind": "state_change",
      "node": "microphone-remove",
      "success_transition": "remove-complete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "valid-choice",
      "id": "save",
      "kind": "state_change",
      "node": "microphone-save",
      "success_transition": "save-complete",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "action": "choose-microphone",
      "binding": "microphone.new_choice",
      "id": "microphone-choose",
      "role": "select",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-choose",
      "test_id": "microphone-choose",
      "text": "Add microphone"
    },
    {
      "id": "microphone-clamshell",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-clamshell",
      "test_id": "microphone-clamshell",
      "text": "Closed-lid override is tried first. Your configured order remains unchanged."
    },
    {
      "children": [
        "microphone-choose",
        "microphone-move-up",
        "microphone-move-down",
        "microphone-remove",
        "microphone-save"
      ],
      "id": "microphone-controls",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-controls",
      "test_id": "microphone-controls"
    },
    {
      "id": "microphone-description",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-description",
      "test_id": "microphone-description",
      "text": "Handy uses the first available microphone. Unavailable choices stay in this list."
    },
    {
      "binding": "microphone.effective",
      "id": "microphone-effective",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-effective",
      "test_id": "microphone-effective",
      "text": "Next input: MacBook microphone · Fallback from unavailable first choice"
    },
    {
      "children": [
        "microphone-effective",
        "microphone-clamshell"
      ],
      "id": "microphone-effective-section",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-effective-section",
      "test_id": "microphone-effective-section"
    },
    {
      "id": "microphone-example-first",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-example-first",
      "test_id": "microphone-example-first",
      "text": "1  USB headset · Unavailable — sample data"
    },
    {
      "id": "microphone-example-second",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-example-second",
      "test_id": "microphone-example-second",
      "text": "2  MacBook microphone · Available — sample data"
    },
    {
      "children": [
        "microphone-title",
        "microphone-description"
      ],
      "id": "microphone-header",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-header",
      "test_id": "microphone-header"
    },
    {
      "action": "move-down",
      "id": "microphone-move-down",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-move-down",
      "test_id": "microphone-move-down",
      "text": "Move down"
    },
    {
      "action": "move-up",
      "id": "microphone-move-up",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-move-up",
      "test_id": "microphone-move-up",
      "text": "Move up"
    },
    {
      "id": "microphone-priority-label",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-priority-label",
      "test_id": "microphone-priority-label",
      "text": "Configured order"
    },
    {
      "binding": "microphone.priority",
      "children": [
        "microphone-example-first",
        "microphone-example-second"
      ],
      "id": "microphone-priority-list",
      "repeat_model": {
        "collection": "microphone.priority",
        "instance_id_template": "microphone-choice-{identity}",
        "item_key": "identity",
        "item_model": "microphone-choice"
      },
      "role": "list",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-priority-list",
      "test_id": "microphone-priority-list"
    },
    {
      "children": [
        "microphone-priority-label",
        "microphone-priority-list"
      ],
      "id": "microphone-priority-section",
      "role": "container",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-priority-section",
      "test_id": "microphone-priority-section"
    },
    {
      "action": "remove",
      "id": "microphone-remove",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-remove",
      "test_id": "microphone-remove",
      "text": "Remove"
    },
    {
      "action": "save",
      "id": "microphone-save",
      "role": "button",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-save",
      "test_id": "microphone-save",
      "text": "Save"
    },
    {
      "children": [
        "microphone-header",
        "microphone-priority-section",
        "microphone-effective-section",
        "microphone-controls"
      ],
      "id": "microphone-settings",
      "role": "screen",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-settings",
      "test_id": "microphone-settings"
    },
    {
      "id": "microphone-title",
      "role": "text",
      "screen_id": "microphone-settings",
      "semantic_id": "microphone-title",
      "test_id": "microphone-title",
      "text": "Microphone priority"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "microphone-priority",
    "id": "microphone-settings",
    "root_node": "microphone-settings",
    "states": [
      {
        "id": "editing"
      },
      {
        "id": "saving"
      },
      {
        "id": "saved"
      },
      {
        "id": "error"
      },
      {
        "id": "move-up-complete"
      },
      {
        "id": "move-down-complete"
      },
      {
        "id": "remove-complete"
      }
    ],
    "surface": "web",
    "title": "Microphone priority",
    "viewport": {
      "height": 680,
      "width": 900
    }
  }
}
```
