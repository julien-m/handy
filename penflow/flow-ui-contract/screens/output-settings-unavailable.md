<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: output-settings-unavailable

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_key"
      },
      "event": "change",
      "id": "output-settings-unavailable.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings-unavailable.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "id": "output-settings-unavailable.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings-unavailable.auto-enter.mode",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings-unavailable.clipboard.select",
      "kind": "state_change",
      "node": "output-settings-unavailable.clipboard.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "no_field_bubble"
      },
      "event": "change",
      "id": "output-settings-unavailable.no-field-bubble.toggle",
      "kind": "state_change",
      "node": "output-settings-unavailable.no-field-bubble.toggle",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings-unavailable.paste-method.select",
      "kind": "state_change",
      "node": "output-settings-unavailable.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-unavailable.return-app.info",
      "kind": "state_change",
      "node": "output-settings-unavailable.return-app.info",
      "success_transition": "output-settings-unavailable.return-app.info.details-shown",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "return_to_app"
      },
      "event": "change",
      "id": "output-settings-unavailable.return-app.toggle",
      "kind": "state_change",
      "node": "output-settings-unavailable.return-app.toggle",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "output-settings-unavailable.output"
      ],
      "id": "output-settings-unavailable",
      "role": "screen",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable",
      "test_id": "output-settings-unavailable-screen"
    },
    {
      "children": [
        "output-settings-unavailable.auto-enter.title",
        "output-settings-unavailable.auto-enter.mode",
        "output-settings-unavailable.auto-enter.key",
        "output-settings-unavailable.auto-enter.note"
      ],
      "id": "output-settings-unavailable.auto-enter",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.auto-enter",
      "test_id": "output-settings-unavailable-auto-enter"
    },
    {
      "action": "output-settings-unavailable.auto-enter.key",
      "binding": "output.auto_enter_key",
      "id": "output-settings-unavailable.auto-enter.key",
      "role": "select",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.auto-enter.key",
      "test_id": "output-settings-unavailable-auto-enter-key",
      "text": "Enter"
    },
    {
      "action": "output-settings-unavailable.auto-enter.mode",
      "binding": "output.auto_enter_mode",
      "id": "output-settings-unavailable.auto-enter.mode",
      "role": "select",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.auto-enter.mode",
      "test_id": "output-settings-unavailable-auto-enter-mode",
      "text": "Only in PIN mode"
    },
    {
      "id": "output-settings-unavailable.auto-enter.note",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.auto-enter.note",
      "test_id": "output-settings-unavailable-auto-enter-note",
      "text": "Replaces Auto Submit. No Enter after a failed or unconfirmed insertion, with paste method None, or when the app lost focus."
    },
    {
      "id": "output-settings-unavailable.auto-enter.title",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.auto-enter.title",
      "test_id": "output-settings-unavailable-auto-enter-title",
      "text": "Automatic Enter"
    },
    {
      "children": [
        "output-settings-unavailable.clipboard.title",
        "output-settings-unavailable.clipboard.select"
      ],
      "id": "output-settings-unavailable.clipboard",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.clipboard",
      "test_id": "output-settings-unavailable-clipboard"
    },
    {
      "action": "output-settings-unavailable.clipboard.select",
      "binding": "output.clipboard_handling",
      "id": "output-settings-unavailable.clipboard.select",
      "role": "select",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.clipboard.select",
      "test_id": "output-settings-unavailable-clipboard-select",
      "text": "Don't Modify Clipboard"
    },
    {
      "id": "output-settings-unavailable.clipboard.title",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.clipboard.title",
      "test_id": "output-settings-unavailable-clipboard-title",
      "text": "Clipboard Handling"
    },
    {
      "children": [
        "output-settings-unavailable.no-field-bubble.title",
        "output-settings-unavailable.no-field-bubble.toggle"
      ],
      "id": "output-settings-unavailable.no-field-bubble",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.no-field-bubble",
      "test_id": "output-settings-unavailable-no-field-bubble"
    },
    {
      "id": "output-settings-unavailable.no-field-bubble.title",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.no-field-bubble.title",
      "test_id": "output-settings-unavailable-no-field-bubble-title",
      "text": "Show the bubble when no text field is focused"
    },
    {
      "action": "output-settings-unavailable.no-field-bubble.toggle",
      "binding": "output.no_field_bubble",
      "id": "output-settings-unavailable.no-field-bubble.toggle",
      "role": "toggle",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.no-field-bubble.toggle",
      "test_id": "output-settings-unavailable-no-field-bubble-toggle"
    },
    {
      "children": [
        "output-settings-unavailable.paste-method",
        "output-settings-unavailable.clipboard",
        "output-settings-unavailable.return-app",
        "output-settings-unavailable.auto-enter",
        "output-settings-unavailable.no-field-bubble"
      ],
      "id": "output-settings-unavailable.output",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.output",
      "test_id": "output-settings-unavailable-output",
      "text": "Output"
    },
    {
      "children": [
        "output-settings-unavailable.paste-method.title",
        "output-settings-unavailable.paste-method.select"
      ],
      "id": "output-settings-unavailable.paste-method",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.paste-method",
      "test_id": "output-settings-unavailable-paste-method"
    },
    {
      "action": "output-settings-unavailable.paste-method.select",
      "binding": "output.paste_method",
      "id": "output-settings-unavailable.paste-method.select",
      "role": "select",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.paste-method.select",
      "test_id": "output-settings-unavailable-paste-method-select",
      "text": "Clipboard (Ctrl+V)"
    },
    {
      "id": "output-settings-unavailable.paste-method.title",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.paste-method.title",
      "test_id": "output-settings-unavailable-paste-method-title",
      "text": "Paste Method"
    },
    {
      "children": [
        "output-settings-unavailable.return-app.title",
        "output-settings-unavailable.return-app.info",
        "output-settings-unavailable.return-app.toggle"
      ],
      "id": "output-settings-unavailable.return-app",
      "role": "container",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.return-app",
      "test_id": "output-settings-unavailable-return-app"
    },
    {
      "action": "output-settings-unavailable.return-app.info",
      "id": "output-settings-unavailable.return-app.info",
      "role": "button",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.return-app.info",
      "test_id": "output-settings-unavailable-return-app-info"
    },
    {
      "id": "output-settings-unavailable.return-app.title",
      "role": "text",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.return-app.title",
      "test_id": "output-settings-unavailable-return-app-title",
      "text": "Return to the dictation's application"
    },
    {
      "action": "output-settings-unavailable.return-app.toggle",
      "binding": "output.return_to_app",
      "id": "output-settings-unavailable.return-app.toggle",
      "role": "toggle",
      "screen_id": "output-settings-unavailable",
      "semantic_id": "output-settings-unavailable.return-app.toggle",
      "test_id": "output-settings-unavailable-return-app-toggle"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "destination-application",
    "id": "output-settings-unavailable",
    "root_node": "output-settings-unavailable",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "details-shown"
      }
    ],
    "surface": "desktop",
    "title": "Output settings — destination unavailable",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
