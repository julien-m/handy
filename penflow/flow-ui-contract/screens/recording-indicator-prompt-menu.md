<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-prompt-menu

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.destination",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.destination",
      "success_transition": "recording-indicator-prompt-menu.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.destination.remove",
      "success_transition": "recording-indicator-prompt-menu.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.menu.none",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.menu.none",
      "success_transition": "recording-indicator-prompt-menu.menu.none.prompt-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.menu.prompt",
      "success_transition": "recording-indicator-prompt-menu.menu.prompt.prompt-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.prompt",
      "success_transition": "recording-indicator-prompt-menu.prompt.prompt-menu-open",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recording-indicator-prompt-menu.menu",
        "recording-indicator-prompt-menu.bar"
      ],
      "id": "recording-indicator-prompt-menu",
      "role": "screen",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu",
      "test_id": "recording-indicator-prompt-menu-screen"
    },
    {
      "children": [
        "recording-indicator-prompt-menu.prompt",
        "recording-indicator-prompt-menu.destination",
        "recording-indicator-prompt-menu.pill"
      ],
      "id": "recording-indicator-prompt-menu.bar",
      "role": "container",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.bar",
      "test_id": "recording-indicator-prompt-menu-bar"
    },
    {
      "action": "recording-indicator-prompt-menu.destination",
      "children": [
        "recording-indicator-prompt-menu.destination.name",
        "recording-indicator-prompt-menu.destination.remove"
      ],
      "id": "recording-indicator-prompt-menu.destination",
      "role": "button",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.destination",
      "test_id": "recording-indicator-prompt-menu-destination"
    },
    {
      "id": "recording-indicator-prompt-menu.destination.name",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.destination.name",
      "test_id": "recording-indicator-prompt-menu-destination-name",
      "text": "ChatGPT"
    },
    {
      "action": "recording-indicator-prompt-menu.destination.remove",
      "id": "recording-indicator-prompt-menu.destination.remove",
      "role": "button",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.destination.remove",
      "test_id": "recording-indicator-prompt-menu-destination-remove"
    },
    {
      "children": [
        "recording-indicator-prompt-menu.menu.label",
        "recording-indicator-prompt-menu.menu.none",
        "recording-indicator-prompt-menu.menu.prompt",
        "recording-indicator-prompt-menu.menu.note"
      ],
      "id": "recording-indicator-prompt-menu.menu",
      "role": "container",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu",
      "test_id": "recording-indicator-prompt-menu-menu"
    },
    {
      "id": "recording-indicator-prompt-menu.menu.label",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu.label",
      "test_id": "recording-indicator-prompt-menu-menu-label",
      "text": "Prompt for this dictation"
    },
    {
      "action": "recording-indicator-prompt-menu.menu.none",
      "id": "recording-indicator-prompt-menu.menu.none",
      "role": "button",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu.none",
      "test_id": "recording-indicator-prompt-menu-menu-none",
      "text": "None (no AI processing)"
    },
    {
      "id": "recording-indicator-prompt-menu.menu.note",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu.note",
      "test_id": "recording-indicator-prompt-menu-menu-note",
      "text": "Applies to this dictation only. Default stays Minimal correction."
    },
    {
      "action": "recording-indicator-prompt-menu.menu.prompt",
      "children": [
        "recording-indicator-prompt-menu.menu.prompt.shortcut"
      ],
      "id": "recording-indicator-prompt-menu.menu.prompt",
      "repeat_model": {
        "collection": "prompt_library.prompts",
        "instance_id_template": "prompt-{id}",
        "item_key": "id",
        "item_model": "ai_prompt"
      },
      "role": "button",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu.prompt",
      "test_id": "recording-indicator-prompt-menu-menu-prompt",
      "text": "Minimal correction"
    },
    {
      "id": "recording-indicator-prompt-menu.menu.prompt.shortcut",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.menu.prompt.shortcut",
      "test_id": "recording-indicator-prompt-menu-menu-prompt-shortcut",
      "text": "⌃ ⌥ M"
    },
    {
      "children": [
        "recording-indicator-prompt-menu.pill.timer"
      ],
      "id": "recording-indicator-prompt-menu.pill",
      "role": "container",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.pill",
      "test_id": "recording-indicator-prompt-menu-pill"
    },
    {
      "id": "recording-indicator-prompt-menu.pill.timer",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.pill.timer",
      "test_id": "recording-indicator-prompt-menu-pill-timer",
      "text": "0:07"
    },
    {
      "action": "recording-indicator-prompt-menu.prompt",
      "children": [
        "recording-indicator-prompt-menu.prompt.shortcut"
      ],
      "id": "recording-indicator-prompt-menu.prompt",
      "role": "button",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.prompt",
      "test_id": "recording-indicator-prompt-menu-prompt",
      "text": "Minimal correction"
    },
    {
      "id": "recording-indicator-prompt-menu.prompt.shortcut",
      "role": "text",
      "screen_id": "recording-indicator-prompt-menu",
      "semantic_id": "recording-indicator-prompt-menu.prompt.shortcut",
      "test_id": "recording-indicator-prompt-menu-prompt-shortcut",
      "text": "⌃ ⌥ M"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "custom-ai-prompts",
    "id": "recording-indicator-prompt-menu",
    "root_node": "recording-indicator-prompt-menu",
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
        "id": "prompt-chosen"
      },
      {
        "id": "prompt-menu-open"
      }
    ],
    "surface": "desktop",
    "title": "Recording indicator — prompt menu",
    "viewport": {
      "height": 460,
      "width": 560
    }
  }
}
```
