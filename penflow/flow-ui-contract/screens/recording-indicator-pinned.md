<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-pinned

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.destination",
      "kind": "state_change",
      "node": "recording-indicator-pinned.destination",
      "success_transition": "recording-indicator-pinned.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-pinned.destination.remove",
      "success_transition": "recording-indicator-pinned.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.app",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.app",
      "success_transition": "recording-indicator-pinned.menu.app.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.frontmost",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.frontmost",
      "success_transition": "recording-indicator-pinned.menu.frontmost.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "recording-indicator-pinned.menu.frontmost-at-end",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.frontmost-at-end",
      "success_transition": "recording-indicator-pinned.menu.frontmost-at-end.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.unpin",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.unpin",
      "success_transition": "recording-indicator-pinned.menu.unpin.unpinned",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.prompt",
      "kind": "state_change",
      "node": "recording-indicator-pinned.prompt",
      "success_transition": "recording-indicator-pinned.prompt.prompt-menu-open",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recording-indicator-pinned.menu",
        "recording-indicator-pinned.bar"
      ],
      "id": "recording-indicator-pinned",
      "role": "screen",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned",
      "test_id": "recording-indicator-pinned-screen"
    },
    {
      "children": [
        "recording-indicator-pinned.prompt",
        "recording-indicator-pinned.destination",
        "recording-indicator-pinned.pill"
      ],
      "id": "recording-indicator-pinned.bar",
      "role": "container",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.bar",
      "test_id": "recording-indicator-pinned-bar"
    },
    {
      "action": "recording-indicator-pinned.destination",
      "children": [
        "recording-indicator-pinned.destination.name",
        "recording-indicator-pinned.destination.mark",
        "recording-indicator-pinned.destination.remove"
      ],
      "id": "recording-indicator-pinned.destination",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.destination",
      "test_id": "recording-indicator-pinned-destination"
    },
    {
      "id": "recording-indicator-pinned.destination.mark",
      "role": "icon",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.destination.mark",
      "test_id": "recording-indicator-pinned-destination-mark"
    },
    {
      "id": "recording-indicator-pinned.destination.name",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.destination.name",
      "test_id": "recording-indicator-pinned-destination-name",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator-pinned.destination.remove",
      "id": "recording-indicator-pinned.destination.remove",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.destination.remove",
      "test_id": "recording-indicator-pinned-destination-remove"
    },
    {
      "children": [
        "recording-indicator-pinned.menu.now-label",
        "recording-indicator-pinned.menu.frontmost",
        "recording-indicator-pinned.menu.others-label",
        "recording-indicator-pinned.menu.app",
        "recording-indicator-pinned.menu.frontmost-at-end",
        "recording-indicator-pinned.menu.unpin"
      ],
      "id": "recording-indicator-pinned.menu",
      "role": "container",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu",
      "test_id": "recording-indicator-pinned-menu"
    },
    {
      "action": "recording-indicator-pinned.menu.app",
      "id": "recording-indicator-pinned.menu.app",
      "repeat_model": {
        "collection": "destination_menu.apps",
        "instance_id_template": "app-{token}",
        "item_key": "token",
        "item_model": "running_app"
      },
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.app",
      "test_id": "recording-indicator-pinned-menu-app",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator-pinned.menu.frontmost",
      "children": [
        "recording-indicator-pinned.menu.hint"
      ],
      "id": "recording-indicator-pinned.menu.frontmost",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.frontmost",
      "test_id": "recording-indicator-pinned-menu-frontmost",
      "text": "Comet"
    },
    {
      "action": "recording-indicator-pinned.menu.frontmost-at-end",
      "id": "recording-indicator-pinned.menu.frontmost-at-end",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.frontmost-at-end",
      "test_id": "recording-indicator-pinned-menu-frontmost-at-end",
      "text": "Frontmost app at the end"
    },
    {
      "id": "recording-indicator-pinned.menu.hint",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.hint",
      "test_id": "recording-indicator-pinned-menu-hint",
      "text": "This dictation only"
    },
    {
      "id": "recording-indicator-pinned.menu.now-label",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.now-label",
      "test_id": "recording-indicator-pinned-menu-now-label",
      "text": "Frontmost now"
    },
    {
      "id": "recording-indicator-pinned.menu.others-label",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.others-label",
      "test_id": "recording-indicator-pinned-menu-others-label",
      "text": "Other apps"
    },
    {
      "action": "recording-indicator-pinned.menu.unpin",
      "id": "recording-indicator-pinned.menu.unpin",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.menu.unpin",
      "test_id": "recording-indicator-pinned-menu-unpin",
      "text": "Unpin ChatGPT"
    },
    {
      "children": [
        "recording-indicator-pinned.pill.timer"
      ],
      "id": "recording-indicator-pinned.pill",
      "role": "container",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.pill",
      "test_id": "recording-indicator-pinned-pill"
    },
    {
      "id": "recording-indicator-pinned.pill.timer",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.pill.timer",
      "test_id": "recording-indicator-pinned-pill-timer",
      "text": "0:07"
    },
    {
      "action": "recording-indicator-pinned.prompt",
      "children": [
        "recording-indicator-pinned.prompt.shortcut",
        "recording-indicator-pinned.prompt.hint"
      ],
      "id": "recording-indicator-pinned.prompt",
      "role": "button",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.prompt",
      "test_id": "recording-indicator-pinned-prompt",
      "text": "Minimal correction"
    },
    {
      "id": "recording-indicator-pinned.prompt.hint",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.prompt.hint",
      "test_id": "recording-indicator-pinned-prompt-hint",
      "text": "Hover to see options"
    },
    {
      "id": "recording-indicator-pinned.prompt.shortcut",
      "role": "text",
      "screen_id": "recording-indicator-pinned",
      "semantic_id": "recording-indicator-pinned.prompt.shortcut",
      "test_id": "recording-indicator-pinned-prompt-shortcut",
      "text": "⌃ ⌥ M"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "pin-mode-and-enter-rule",
    "id": "recording-indicator-pinned",
    "root_node": "recording-indicator-pinned",
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
        "id": "prompt-menu-open"
      },
      {
        "id": "unpinned"
      }
    ],
    "surface": "desktop",
    "title": "Recording indicator — pinned destination",
    "viewport": {
      "height": 400,
      "width": 560
    }
  }
}
```
