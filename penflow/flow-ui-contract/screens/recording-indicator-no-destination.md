<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-no-destination

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-no-destination.destination",
      "kind": "state_change",
      "node": "recording-indicator-no-destination.destination",
      "success_transition": "recording-indicator-no-destination.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-no-destination.prompt",
      "kind": "state_change",
      "node": "recording-indicator-no-destination.prompt",
      "success_transition": "recording-indicator-no-destination.prompt.prompt-menu-open",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recording-indicator-no-destination.bar"
      ],
      "id": "recording-indicator-no-destination",
      "role": "screen",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination",
      "test_id": "recording-indicator-no-destination-screen"
    },
    {
      "children": [
        "recording-indicator-no-destination.prompt",
        "recording-indicator-no-destination.destination",
        "recording-indicator-no-destination.pill"
      ],
      "id": "recording-indicator-no-destination.bar",
      "role": "container",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.bar",
      "test_id": "recording-indicator-no-destination-bar"
    },
    {
      "action": "recording-indicator-no-destination.destination",
      "children": [
        "recording-indicator-no-destination.destination.name"
      ],
      "id": "recording-indicator-no-destination.destination",
      "role": "button",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.destination",
      "test_id": "recording-indicator-no-destination-destination"
    },
    {
      "id": "recording-indicator-no-destination.destination.name",
      "role": "text",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.destination.name",
      "test_id": "recording-indicator-no-destination-destination-name",
      "text": "Frontmost app"
    },
    {
      "children": [
        "recording-indicator-no-destination.pill.timer"
      ],
      "id": "recording-indicator-no-destination.pill",
      "role": "container",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.pill",
      "test_id": "recording-indicator-no-destination-pill"
    },
    {
      "id": "recording-indicator-no-destination.pill.timer",
      "role": "text",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.pill.timer",
      "test_id": "recording-indicator-no-destination-pill-timer",
      "text": "0:07"
    },
    {
      "action": "recording-indicator-no-destination.prompt",
      "children": [
        "recording-indicator-no-destination.prompt.shortcut",
        "recording-indicator-no-destination.prompt.hint"
      ],
      "id": "recording-indicator-no-destination.prompt",
      "role": "button",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.prompt",
      "test_id": "recording-indicator-no-destination-prompt",
      "text": "Minimal correction"
    },
    {
      "id": "recording-indicator-no-destination.prompt.hint",
      "role": "text",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.prompt.hint",
      "test_id": "recording-indicator-no-destination-prompt-hint",
      "text": "Hover to see options"
    },
    {
      "id": "recording-indicator-no-destination.prompt.shortcut",
      "role": "text",
      "screen_id": "recording-indicator-no-destination",
      "semantic_id": "recording-indicator-no-destination.prompt.shortcut",
      "test_id": "recording-indicator-no-destination-prompt-shortcut",
      "text": "⌃ ⌥ M"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "destination-application",
    "id": "recording-indicator-no-destination",
    "root_node": "recording-indicator-no-destination",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "menu-open"
      },
      {
        "id": "prompt-menu-open"
      }
    ],
    "surface": "desktop",
    "title": "Recording indicator — frontmost app",
    "viewport": {
      "height": 200,
      "width": 560
    }
  }
}
```
