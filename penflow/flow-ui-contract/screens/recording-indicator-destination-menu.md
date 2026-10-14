<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-destination-menu

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.destination",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.destination",
      "success_transition": "recording-indicator-destination-menu.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.destination.remove",
      "success_transition": "recording-indicator-destination-menu.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.app",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.app",
      "success_transition": "recording-indicator-destination-menu.menu.app.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.frontmost",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.frontmost",
      "success_transition": "recording-indicator-destination-menu.menu.frontmost.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "success_transition": "recording-indicator-destination-menu.menu.frontmost-at-end.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.pin",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.pin",
      "success_transition": "recording-indicator-destination-menu.menu.pin.pinned",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.prompt",
      "success_transition": "recording-indicator-destination-menu.prompt.prompt-menu-open",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recording-indicator-destination-menu.menu",
        "recording-indicator-destination-menu.bar"
      ],
      "id": "recording-indicator-destination-menu",
      "role": "screen",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu",
      "test_id": "recording-indicator-destination-menu-screen"
    },
    {
      "children": [
        "recording-indicator-destination-menu.prompt",
        "recording-indicator-destination-menu.destination",
        "recording-indicator-destination-menu.pill"
      ],
      "id": "recording-indicator-destination-menu.bar",
      "role": "container",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.bar",
      "test_id": "recording-indicator-destination-menu-bar"
    },
    {
      "action": "recording-indicator-destination-menu.destination",
      "children": [
        "recording-indicator-destination-menu.destination.name",
        "recording-indicator-destination-menu.destination.remove"
      ],
      "id": "recording-indicator-destination-menu.destination",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.destination",
      "test_id": "recording-indicator-destination-menu-destination"
    },
    {
      "id": "recording-indicator-destination-menu.destination.name",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.destination.name",
      "test_id": "recording-indicator-destination-menu-destination-name",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator-destination-menu.destination.remove",
      "id": "recording-indicator-destination-menu.destination.remove",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.destination.remove",
      "test_id": "recording-indicator-destination-menu-destination-remove"
    },
    {
      "children": [
        "recording-indicator-destination-menu.menu.now-label",
        "recording-indicator-destination-menu.menu.frontmost",
        "recording-indicator-destination-menu.menu.others-label",
        "recording-indicator-destination-menu.menu.app",
        "recording-indicator-destination-menu.menu.frontmost-at-end",
        "recording-indicator-destination-menu.menu.pin"
      ],
      "id": "recording-indicator-destination-menu.menu",
      "role": "container",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu",
      "test_id": "recording-indicator-destination-menu-menu"
    },
    {
      "action": "recording-indicator-destination-menu.menu.app",
      "id": "recording-indicator-destination-menu.menu.app",
      "repeat_model": {
        "collection": "destination_menu.apps",
        "instance_id_template": "app-{bundle_id}",
        "item_key": "bundle_id",
        "item_model": "running_app"
      },
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.app",
      "test_id": "recording-indicator-destination-menu-menu-app",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator-destination-menu.menu.frontmost",
      "id": "recording-indicator-destination-menu.menu.frontmost",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.frontmost",
      "test_id": "recording-indicator-destination-menu-menu-frontmost",
      "text": "Comet"
    },
    {
      "action": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "id": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "test_id": "recording-indicator-destination-menu-menu-frontmost-at-end",
      "text": "Frontmost app at the end"
    },
    {
      "id": "recording-indicator-destination-menu.menu.now-label",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.now-label",
      "test_id": "recording-indicator-destination-menu-menu-now-label",
      "text": "Frontmost now"
    },
    {
      "id": "recording-indicator-destination-menu.menu.others-label",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.others-label",
      "test_id": "recording-indicator-destination-menu-menu-others-label",
      "text": "Other apps"
    },
    {
      "action": "recording-indicator-destination-menu.menu.pin",
      "id": "recording-indicator-destination-menu.menu.pin",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.menu.pin",
      "test_id": "recording-indicator-destination-menu-menu-pin",
      "text": "Pin ChatGPT"
    },
    {
      "children": [
        "recording-indicator-destination-menu.pill.timer"
      ],
      "id": "recording-indicator-destination-menu.pill",
      "role": "container",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.pill",
      "test_id": "recording-indicator-destination-menu-pill"
    },
    {
      "id": "recording-indicator-destination-menu.pill.timer",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.pill.timer",
      "test_id": "recording-indicator-destination-menu-pill-timer",
      "text": "0:07"
    },
    {
      "action": "recording-indicator-destination-menu.prompt",
      "children": [
        "recording-indicator-destination-menu.prompt.shortcut",
        "recording-indicator-destination-menu.prompt.hint"
      ],
      "id": "recording-indicator-destination-menu.prompt",
      "role": "button",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.prompt",
      "test_id": "recording-indicator-destination-menu-prompt",
      "text": "Minimal correction"
    },
    {
      "id": "recording-indicator-destination-menu.prompt.hint",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.prompt.hint",
      "test_id": "recording-indicator-destination-menu-prompt-hint",
      "text": "Hover to see options"
    },
    {
      "id": "recording-indicator-destination-menu.prompt.shortcut",
      "role": "text",
      "screen_id": "recording-indicator-destination-menu",
      "semantic_id": "recording-indicator-destination-menu.prompt.shortcut",
      "test_id": "recording-indicator-destination-menu-prompt-shortcut",
      "text": "⌃ ⌥ M"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "destination-application",
    "id": "recording-indicator-destination-menu",
    "root_node": "recording-indicator-destination-menu",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "destination-changed"
      },
      {
        "id": "destination-cleared"
      },
      {
        "id": "menu-open"
      },
      {
        "id": "pinned"
      },
      {
        "id": "prompt-menu-open"
      }
    ],
    "surface": "desktop",
    "title": "Recording indicator — destination menu",
    "viewport": {
      "height": 420,
      "width": 560
    }
  }
}
```
