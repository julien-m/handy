<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: output-settings

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
      "guard": "automatic-enter-available",
      "id": "output-settings.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "guard": "automatic-enter-available",
      "id": "output-settings.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings.auto-enter.mode",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings.clipboard.select",
      "kind": "state_change",
      "node": "output-settings.clipboard.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "history",
        "field": "limit"
      },
      "event": "change",
      "id": "output-settings.history-limit.input",
      "kind": "state_change",
      "node": "output-settings.history-limit.input",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "no_field_bubble"
      },
      "event": "change",
      "id": "output-settings.no-field-bubble.toggle",
      "kind": "state_change",
      "node": "output-settings.no-field-bubble.toggle",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings.paste-method.select",
      "kind": "state_change",
      "node": "output-settings.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "destination-return-available",
      "id": "output-settings.return-app.info",
      "kind": "state_change",
      "node": "output-settings.return-app.info",
      "success_transition": "output-settings.return-app.info.details-shown",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "return_to_app"
      },
      "event": "change",
      "guard": "destination-return-available",
      "id": "output-settings.return-app.toggle",
      "kind": "state_change",
      "node": "output-settings.return-app.toggle",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "output-settings.output",
        "output-settings.history"
      ],
      "id": "output-settings",
      "role": "screen",
      "screen_id": "output-settings",
      "semantic_id": "output-settings",
      "test_id": "output-settings-screen"
    },
    {
      "children": [
        "output-settings.auto-enter.title",
        "output-settings.auto-enter.mode",
        "output-settings.auto-enter.key",
        "output-settings.auto-enter.note"
      ],
      "id": "output-settings.auto-enter",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.auto-enter",
      "test_id": "output-settings-auto-enter"
    },
    {
      "action": "output-settings.auto-enter.key",
      "binding": "output.auto_enter_key",
      "id": "output-settings.auto-enter.key",
      "role": "select",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.auto-enter.key",
      "test_id": "output-settings-auto-enter-key",
      "text": "Enter"
    },
    {
      "action": "output-settings.auto-enter.mode",
      "binding": "output.auto_enter_mode",
      "id": "output-settings.auto-enter.mode",
      "role": "select",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.auto-enter.mode",
      "test_id": "output-settings-auto-enter-mode",
      "text": "Only in PIN mode"
    },
    {
      "id": "output-settings.auto-enter.note",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.auto-enter.note",
      "test_id": "output-settings-auto-enter-note",
      "text": "Replaces Auto Submit. No Enter after a failed or unconfirmed insertion, with paste method None, or when the app lost focus."
    },
    {
      "id": "output-settings.auto-enter.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.auto-enter.title",
      "test_id": "output-settings-auto-enter-title",
      "text": "Automatic Enter"
    },
    {
      "children": [
        "output-settings.clipboard.title",
        "output-settings.clipboard.select"
      ],
      "id": "output-settings.clipboard",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.clipboard",
      "test_id": "output-settings-clipboard"
    },
    {
      "action": "output-settings.clipboard.select",
      "binding": "output.clipboard_handling",
      "id": "output-settings.clipboard.select",
      "role": "select",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.clipboard.select",
      "test_id": "output-settings-clipboard-select",
      "text": "Don't Modify Clipboard"
    },
    {
      "id": "output-settings.clipboard.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.clipboard.title",
      "test_id": "output-settings-clipboard-title",
      "text": "Clipboard Handling"
    },
    {
      "children": [
        "output-settings.history-limit"
      ],
      "id": "output-settings.history",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.history",
      "states": [
        "no-refused-write"
      ],
      "test_id": "output-settings-history",
      "text": "History"
    },
    {
      "children": [
        "output-settings.history-limit.title",
        "output-settings.history-limit.input"
      ],
      "id": "output-settings.history-limit",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.history-limit",
      "test_id": "output-settings-history-limit"
    },
    {
      "action": "output-settings.history-limit.input",
      "binding": "history.limit",
      "id": "output-settings.history-limit.input",
      "role": "input",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.history-limit.input",
      "test_id": "output-settings-history-limit-input",
      "text": "500"
    },
    {
      "id": "output-settings.history-limit.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.history-limit.title",
      "test_id": "output-settings-history-limit-title",
      "text": "History Limit"
    },
    {
      "children": [
        "output-settings.no-field-bubble.title",
        "output-settings.no-field-bubble.toggle"
      ],
      "id": "output-settings.no-field-bubble",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.no-field-bubble",
      "test_id": "output-settings-no-field-bubble"
    },
    {
      "id": "output-settings.no-field-bubble.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.no-field-bubble.title",
      "test_id": "output-settings-no-field-bubble-title",
      "text": "Show the bubble when no text field is focused"
    },
    {
      "action": "output-settings.no-field-bubble.toggle",
      "binding": "output.no_field_bubble",
      "id": "output-settings.no-field-bubble.toggle",
      "role": "toggle",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.no-field-bubble.toggle",
      "test_id": "output-settings-no-field-bubble-toggle"
    },
    {
      "children": [
        "output-settings.paste-method",
        "output-settings.clipboard",
        "output-settings.return-app",
        "output-settings.auto-enter",
        "output-settings.no-field-bubble"
      ],
      "id": "output-settings.output",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.output",
      "test_id": "output-settings-output",
      "text": "Output"
    },
    {
      "children": [
        "output-settings.paste-method.title",
        "output-settings.paste-method.select"
      ],
      "id": "output-settings.paste-method",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.paste-method",
      "test_id": "output-settings-paste-method"
    },
    {
      "action": "output-settings.paste-method.select",
      "binding": "output.paste_method",
      "id": "output-settings.paste-method.select",
      "role": "select",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.paste-method.select",
      "test_id": "output-settings-paste-method-select",
      "text": "Smart paste"
    },
    {
      "id": "output-settings.paste-method.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.paste-method.title",
      "test_id": "output-settings-paste-method-title",
      "text": "Paste Method"
    },
    {
      "children": [
        "output-settings.return-app.title",
        "output-settings.return-app.info",
        "output-settings.return-app.toggle"
      ],
      "id": "output-settings.return-app",
      "role": "container",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.return-app",
      "test_id": "output-settings-return-app"
    },
    {
      "action": "output-settings.return-app.info",
      "id": "output-settings.return-app.info",
      "role": "button",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.return-app.info",
      "test_id": "output-settings-return-app-info"
    },
    {
      "id": "output-settings.return-app.title",
      "role": "text",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.return-app.title",
      "test_id": "output-settings-return-app-title",
      "text": "Return to the dictation's application"
    },
    {
      "action": "output-settings.return-app.toggle",
      "binding": "output.return_to_app",
      "id": "output-settings.return-app.toggle",
      "role": "toggle",
      "screen_id": "output-settings",
      "semantic_id": "output-settings.return-app.toggle",
      "test_id": "output-settings-return-app-toggle"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "smart-paste",
    "id": "output-settings",
    "root_node": "output-settings",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "details-shown"
      }
    ],
    "surface": "desktop",
    "title": "Output settings",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
