<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: prompt-settings

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "binding_ref": {
        "access": "write",
        "entity": "post_processing",
        "field": "default_prompt"
      },
      "event": "change",
      "id": "prompt-settings.default.select",
      "kind": "state_change",
      "node": "prompt-settings.default.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "post_processing",
        "field": "enabled"
      },
      "event": "change",
      "id": "prompt-settings.enabled.toggle",
      "kind": "state_change",
      "node": "prompt-settings.enabled.toggle",
      "trigger": "change"
    },
    {
      "destination": {
        "screen": "prompt-settings-editor",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings.new",
      "kind": "navigation",
      "node": "prompt-settings.new",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "prompt-settings-editor",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings.prompt.edit",
      "kind": "navigation",
      "node": "prompt-settings.prompt.edit",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "prompt-settings.processing",
        "prompt-settings.library"
      ],
      "id": "prompt-settings",
      "role": "screen",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings",
      "test_id": "prompt-settings-screen"
    },
    {
      "id": "prompt-settings.count",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.count",
      "test_id": "prompt-settings-count",
      "text": "7 of 50 prompts"
    },
    {
      "children": [
        "prompt-settings.default.title",
        "prompt-settings.default.select"
      ],
      "id": "prompt-settings.default",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.default",
      "test_id": "prompt-settings-default"
    },
    {
      "action": "prompt-settings.default.select",
      "binding": "post_processing.default_prompt",
      "id": "prompt-settings.default.select",
      "role": "select",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.default.select",
      "test_id": "prompt-settings-default-select",
      "text": "Minimal correction"
    },
    {
      "id": "prompt-settings.default.title",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.default.title",
      "test_id": "prompt-settings-default-title",
      "text": "Default prompt"
    },
    {
      "children": [
        "prompt-settings.enabled.title",
        "prompt-settings.enabled.toggle"
      ],
      "id": "prompt-settings.enabled",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.enabled",
      "test_id": "prompt-settings-enabled"
    },
    {
      "id": "prompt-settings.enabled.title",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.enabled.title",
      "test_id": "prompt-settings-enabled-title",
      "text": "AI post-processing"
    },
    {
      "action": "prompt-settings.enabled.toggle",
      "binding": "post_processing.enabled",
      "id": "prompt-settings.enabled.toggle",
      "role": "toggle",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.enabled.toggle",
      "test_id": "prompt-settings-enabled-toggle"
    },
    {
      "children": [
        "prompt-settings.list",
        "prompt-settings.new",
        "prompt-settings.count"
      ],
      "id": "prompt-settings.library",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.library",
      "test_id": "prompt-settings-library",
      "text": "Prompts"
    },
    {
      "children": [
        "prompt-settings.prompt"
      ],
      "id": "prompt-settings.list",
      "repeat_model": {
        "collection": "prompt_library.prompts",
        "instance_id_template": "prompt-{id}",
        "item_key": "id",
        "item_model": "ai_prompt"
      },
      "role": "list",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.list",
      "test_id": "prompt-settings-list"
    },
    {
      "action": "prompt-settings.new",
      "id": "prompt-settings.new",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.new",
      "test_id": "prompt-settings-new",
      "text": "New prompt"
    },
    {
      "children": [
        "prompt-settings.enabled",
        "prompt-settings.default"
      ],
      "id": "prompt-settings.processing",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.processing",
      "test_id": "prompt-settings-processing",
      "text": "AI post-processing"
    },
    {
      "children": [
        "prompt-settings.prompt.name",
        "prompt-settings.prompt.provider",
        "prompt-settings.prompt.shortcut",
        "prompt-settings.prompt.edit"
      ],
      "id": "prompt-settings.prompt",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt",
      "test_id": "prompt-settings-prompt"
    },
    {
      "action": "prompt-settings.prompt.edit",
      "id": "prompt-settings.prompt.edit",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt.edit",
      "test_id": "prompt-settings-prompt-edit"
    },
    {
      "id": "prompt-settings.prompt.name",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt.name",
      "test_id": "prompt-settings-prompt-name",
      "text": "Minimal correction"
    },
    {
      "id": "prompt-settings.prompt.provider",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt.provider",
      "test_id": "prompt-settings-prompt-provider",
      "text": "Local · llama3.2 (Ollama)"
    },
    {
      "id": "prompt-settings.prompt.shortcut",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt.shortcut",
      "test_id": "prompt-settings-prompt-shortcut",
      "text": "⌃ ⌥ M"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "custom-ai-prompts",
    "id": "prompt-settings",
    "root_node": "prompt-settings",
    "states": [
      {
        "id": "default"
      }
    ],
    "surface": "desktop",
    "title": "AI prompts",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
