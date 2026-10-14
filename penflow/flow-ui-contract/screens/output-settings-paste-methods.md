<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: output-settings-paste-methods

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings-paste-methods.clipboard.select",
      "kind": "state_change",
      "node": "output-settings-paste-methods.clipboard.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.clipboard",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.clipboard",
      "success_transition": "output-settings-paste-methods.paste-method.clipboard.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.direct",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.direct",
      "success_transition": "output-settings-paste-methods.paste-method.direct.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.none",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.none",
      "success_transition": "output-settings-paste-methods.paste-method.none.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.script",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.script",
      "success_transition": "output-settings-paste-methods.paste-method.script.method-chosen",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings-paste-methods.paste-method.select",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.smart",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.smart",
      "success_transition": "output-settings-paste-methods.paste-method.smart.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.permission.open",
      "kind": "state_change",
      "node": "output-settings-paste-methods.permission.open",
      "success_transition": "output-settings-paste-methods.permission.open.permission-requested",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "output-settings-paste-methods.permission",
        "output-settings-paste-methods.output"
      ],
      "id": "output-settings-paste-methods",
      "role": "screen",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods",
      "test_id": "output-settings-paste-methods-screen"
    },
    {
      "children": [
        "output-settings-paste-methods.clipboard.title",
        "output-settings-paste-methods.clipboard.select"
      ],
      "id": "output-settings-paste-methods.clipboard",
      "role": "container",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.clipboard",
      "test_id": "output-settings-paste-methods-clipboard"
    },
    {
      "action": "output-settings-paste-methods.clipboard.select",
      "binding": "output.clipboard_handling",
      "id": "output-settings-paste-methods.clipboard.select",
      "role": "select",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.clipboard.select",
      "test_id": "output-settings-paste-methods-clipboard-select",
      "text": "Don't modify clipboard"
    },
    {
      "id": "output-settings-paste-methods.clipboard.title",
      "role": "text",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.clipboard.title",
      "test_id": "output-settings-paste-methods-clipboard-title",
      "text": "Clipboard handling"
    },
    {
      "children": [
        "output-settings-paste-methods.paste-method",
        "output-settings-paste-methods.clipboard"
      ],
      "id": "output-settings-paste-methods.output",
      "role": "container",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.output",
      "test_id": "output-settings-paste-methods-output",
      "text": "Output"
    },
    {
      "children": [
        "output-settings-paste-methods.paste-method.title",
        "output-settings-paste-methods.paste-method.select",
        "output-settings-paste-methods.paste-method.options"
      ],
      "id": "output-settings-paste-methods.paste-method",
      "role": "container",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method",
      "test_id": "output-settings-paste-methods-paste-method"
    },
    {
      "action": "output-settings-paste-methods.paste-method.clipboard",
      "id": "output-settings-paste-methods.paste-method.clipboard",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.clipboard",
      "test_id": "output-settings-paste-methods-paste-method-clipboard",
      "text": "Clipboard (⌘V)"
    },
    {
      "action": "output-settings-paste-methods.paste-method.direct",
      "id": "output-settings-paste-methods.paste-method.direct",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.direct",
      "test_id": "output-settings-paste-methods-paste-method-direct",
      "text": "Direct typing"
    },
    {
      "action": "output-settings-paste-methods.paste-method.none",
      "id": "output-settings-paste-methods.paste-method.none",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.none",
      "test_id": "output-settings-paste-methods-paste-method-none",
      "text": "None"
    },
    {
      "children": [
        "output-settings-paste-methods.paste-method.smart",
        "output-settings-paste-methods.paste-method.clipboard",
        "output-settings-paste-methods.paste-method.direct",
        "output-settings-paste-methods.paste-method.script",
        "output-settings-paste-methods.paste-method.none"
      ],
      "id": "output-settings-paste-methods.paste-method.options",
      "role": "container",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.options",
      "test_id": "output-settings-paste-methods-paste-method-options"
    },
    {
      "action": "output-settings-paste-methods.paste-method.script",
      "id": "output-settings-paste-methods.paste-method.script",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.script",
      "test_id": "output-settings-paste-methods-paste-method-script",
      "text": "External script"
    },
    {
      "action": "output-settings-paste-methods.paste-method.select",
      "binding": "output.paste_method",
      "id": "output-settings-paste-methods.paste-method.select",
      "role": "select",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.select",
      "test_id": "output-settings-paste-methods-paste-method-select",
      "text": "Smart paste"
    },
    {
      "action": "output-settings-paste-methods.paste-method.smart",
      "id": "output-settings-paste-methods.paste-method.smart",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.smart",
      "test_id": "output-settings-paste-methods-paste-method-smart",
      "text": "Smart paste"
    },
    {
      "id": "output-settings-paste-methods.paste-method.title",
      "role": "text",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.paste-method.title",
      "test_id": "output-settings-paste-methods-paste-method-title",
      "text": "Paste method"
    },
    {
      "children": [
        "output-settings-paste-methods.permission.title",
        "output-settings-paste-methods.permission.open"
      ],
      "id": "output-settings-paste-methods.permission",
      "role": "alert",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.permission",
      "test_id": "output-settings-paste-methods-permission"
    },
    {
      "action": "output-settings-paste-methods.permission.open",
      "id": "output-settings-paste-methods.permission.open",
      "role": "button",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.permission.open",
      "test_id": "output-settings-paste-methods-permission-open",
      "text": "Open System Settings"
    },
    {
      "id": "output-settings-paste-methods.permission.title",
      "role": "text",
      "screen_id": "output-settings-paste-methods",
      "semantic_id": "output-settings-paste-methods.permission.title",
      "test_id": "output-settings-paste-methods-permission-title",
      "text": "Accessibility permission is off"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "smart-paste",
    "id": "output-settings-paste-methods",
    "root_node": "output-settings-paste-methods",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "method-chosen"
      },
      {
        "id": "permission-requested"
      }
    ],
    "surface": "desktop",
    "title": "Output settings — paste methods",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
