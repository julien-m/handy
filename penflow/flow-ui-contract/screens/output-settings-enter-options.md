<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: output-settings-enter-options

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.always",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.always",
      "success_transition": "output-settings-enter-options.auto-enter.always.mode-chosen",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_key"
      },
      "event": "change",
      "id": "output-settings-enter-options.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "id": "output-settings-enter-options.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.mode",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.never",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.never",
      "success_transition": "output-settings-enter-options.auto-enter.never.mode-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.pin-only",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.pin-only",
      "success_transition": "output-settings-enter-options.auto-enter.pin-only.mode-chosen",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "output-settings-enter-options.output",
        "output-settings-enter-options.auto-enter.note"
      ],
      "id": "output-settings-enter-options",
      "role": "screen",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options",
      "test_id": "output-settings-enter-options-screen"
    },
    {
      "children": [
        "output-settings-enter-options.auto-enter.title",
        "output-settings-enter-options.auto-enter.mode",
        "output-settings-enter-options.auto-enter.key",
        "output-settings-enter-options.auto-enter.options"
      ],
      "id": "output-settings-enter-options.auto-enter",
      "role": "container",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter",
      "test_id": "output-settings-enter-options-auto-enter"
    },
    {
      "action": "output-settings-enter-options.auto-enter.always",
      "id": "output-settings-enter-options.auto-enter.always",
      "role": "button",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.always",
      "test_id": "output-settings-enter-options-auto-enter-always",
      "text": "Always"
    },
    {
      "action": "output-settings-enter-options.auto-enter.key",
      "binding": "output.auto_enter_key",
      "id": "output-settings-enter-options.auto-enter.key",
      "role": "select",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.key",
      "test_id": "output-settings-enter-options-auto-enter-key",
      "text": "Enter"
    },
    {
      "action": "output-settings-enter-options.auto-enter.mode",
      "binding": "output.auto_enter_mode",
      "id": "output-settings-enter-options.auto-enter.mode",
      "role": "select",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.mode",
      "test_id": "output-settings-enter-options-auto-enter-mode",
      "text": "Only in PIN mode"
    },
    {
      "action": "output-settings-enter-options.auto-enter.never",
      "id": "output-settings-enter-options.auto-enter.never",
      "role": "button",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.never",
      "test_id": "output-settings-enter-options-auto-enter-never",
      "text": "Never"
    },
    {
      "id": "output-settings-enter-options.auto-enter.note",
      "role": "text",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.note",
      "test_id": "output-settings-enter-options-auto-enter-note",
      "text": "Replaces Auto Submit. No Enter after a failed or unconfirmed insertion, with paste method None, or when the app lost focus."
    },
    {
      "children": [
        "output-settings-enter-options.auto-enter.never",
        "output-settings-enter-options.auto-enter.always",
        "output-settings-enter-options.auto-enter.pin-only"
      ],
      "id": "output-settings-enter-options.auto-enter.options",
      "role": "container",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.options",
      "test_id": "output-settings-enter-options-auto-enter-options"
    },
    {
      "action": "output-settings-enter-options.auto-enter.pin-only",
      "id": "output-settings-enter-options.auto-enter.pin-only",
      "role": "button",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.pin-only",
      "test_id": "output-settings-enter-options-auto-enter-pin-only",
      "text": "Only in PIN mode"
    },
    {
      "id": "output-settings-enter-options.auto-enter.title",
      "role": "text",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.auto-enter.title",
      "test_id": "output-settings-enter-options-auto-enter-title",
      "text": "Automatic Enter"
    },
    {
      "children": [
        "output-settings-enter-options.auto-enter"
      ],
      "id": "output-settings-enter-options.output",
      "role": "container",
      "screen_id": "output-settings-enter-options",
      "semantic_id": "output-settings-enter-options.output",
      "test_id": "output-settings-enter-options-output",
      "text": "Output"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "pin-mode-and-enter-rule",
    "id": "output-settings-enter-options",
    "root_node": "output-settings-enter-options",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "mode-chosen"
      }
    ],
    "surface": "desktop",
    "title": "Output settings — automatic Enter",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
