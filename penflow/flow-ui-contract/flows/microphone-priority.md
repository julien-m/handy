<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Flow: microphone-priority

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
  "flow": {
    "entry_screen": "microphone-settings",
    "id": "microphone-priority"
  },
  "relations": [],
  "screens": [
    {
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
  ],
  "transitions": [
    {
      "action": "move-down",
      "causal": {
        "action_visibility": "user-visible",
        "fanout": {
          "group": "move-down",
          "policy": "exclusive"
        },
        "outcome": "move-down-complete",
        "version": 1
      },
      "flow": "microphone-priority",
      "from_screen": "microphone-settings",
      "from_state": "editing",
      "guard": "valid-choice",
      "id": "move-down-complete",
      "kind": "state",
      "label": "Draft order after move-down",
      "to_screen": "microphone-settings",
      "to_state": "move-down-complete",
      "trigger": "move-down"
    },
    {
      "action": "move-up",
      "causal": {
        "action_visibility": "user-visible",
        "fanout": {
          "group": "move-up",
          "policy": "exclusive"
        },
        "outcome": "move-up-complete",
        "version": 1
      },
      "flow": "microphone-priority",
      "from_screen": "microphone-settings",
      "from_state": "editing",
      "guard": "valid-choice",
      "id": "move-up-complete",
      "kind": "state",
      "label": "Draft order after move-up",
      "to_screen": "microphone-settings",
      "to_state": "move-up-complete",
      "trigger": "move-up"
    },
    {
      "action": "remove",
      "causal": {
        "action_visibility": "user-visible",
        "fanout": {
          "group": "remove",
          "policy": "exclusive"
        },
        "outcome": "remove-complete",
        "version": 1
      },
      "flow": "microphone-priority",
      "from_screen": "microphone-settings",
      "from_state": "editing",
      "guard": "valid-choice",
      "id": "remove-complete",
      "kind": "state",
      "label": "Draft order after remove",
      "to_screen": "microphone-settings",
      "to_state": "remove-complete",
      "trigger": "remove"
    },
    {
      "action": "save",
      "causal": {
        "action_visibility": "user-visible",
        "fanout": {
          "group": "save",
          "policy": "exclusive"
        },
        "outcome": "priority-saved",
        "version": 1
      },
      "flow": "microphone-priority",
      "from_screen": "microphone-settings",
      "from_state": "editing",
      "guard": "valid-choice",
      "id": "save-complete",
      "kind": "state",
      "label": "Save the complete priority list",
      "to_screen": "microphone-settings",
      "to_state": "saved",
      "trigger": "save"
    }
  ]
}
```
