<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.destination",
      "kind": "state_change",
      "node": "recording-indicator.destination",
      "success_transition": "recording-indicator.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator.destination.remove",
      "success_transition": "recording-indicator.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.prompt",
      "kind": "state_change",
      "node": "recording-indicator.prompt",
      "success_transition": "recording-indicator.prompt.prompt-menu-open",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recording-indicator.bar"
      ],
      "id": "recording-indicator",
      "role": "screen",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator",
      "test_id": "recording-indicator-screen"
    },
    {
      "children": [
        "recording-indicator.pill",
        "recording-indicator.prompt",
        "recording-indicator.destination"
      ],
      "id": "recording-indicator.bar",
      "role": "container",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.bar",
      "test_id": "recording-indicator-bar"
    },
    {
      "action": "recording-indicator.destination",
      "children": [
        "recording-indicator.destination.name",
        "recording-indicator.destination.remove"
      ],
      "id": "recording-indicator.destination",
      "role": "button",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.destination",
      "test_id": "recording-indicator-destination"
    },
    {
      "id": "recording-indicator.destination.name",
      "role": "text",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.destination.name",
      "test_id": "recording-indicator-destination-name",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator.destination.remove",
      "id": "recording-indicator.destination.remove",
      "role": "button",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.destination.remove",
      "test_id": "recording-indicator-destination-remove"
    },
    {
      "children": [
        "recording-indicator.pill.timer"
      ],
      "id": "recording-indicator.pill",
      "role": "container",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.pill",
      "test_id": "recording-indicator-pill"
    },
    {
      "id": "recording-indicator.pill.timer",
      "role": "text",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.pill.timer",
      "test_id": "recording-indicator-pill-timer",
      "text": "0:07"
    },
    {
      "action": "recording-indicator.prompt",
      "id": "recording-indicator.prompt",
      "role": "button",
      "screen_id": "recording-indicator",
      "semantic_id": "recording-indicator.prompt",
      "test_id": "recording-indicator-prompt",
      "text": "Minimal correction"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "destination-application",
    "id": "recording-indicator",
    "root_node": "recording-indicator",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "destination-cleared"
      },
      {
        "id": "menu-open"
      },
      {
        "id": "prompt-menu-open"
      }
    ],
    "surface": "desktop",
    "title": "Recording indicator",
    "viewport": {
      "height": 200,
      "width": 560
    }
  }
}
```
