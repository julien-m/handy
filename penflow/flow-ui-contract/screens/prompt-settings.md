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
        "screen": "prompt-settings-invalid",
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
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings.reload-retry",
      "kind": "state_change",
      "node": "prompt-settings.reload-retry",
      "success_transition": "prompt-settings.reload-retry.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.add",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.add",
      "success_transition": "shortcut-settings.ai-dictation.add.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.reset",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.reset",
      "success_transition": "shortcut-settings.ai-dictation.reset.reset",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.shortcut.remove",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.shortcut.remove",
      "success_transition": "shortcut-settings.ai-dictation.shortcut.remove.shortcut-removed",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "prompt-settings.processing",
        "prompt-settings.library",
        "shortcut-settings.ai-dictation"
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
      "id": "prompt-settings.empty",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.empty",
      "test_id": "prompt-settings-empty"
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
        "prompt-settings.count",
        "prompt-settings.load-error",
        "prompt-settings.loading",
        "prompt-settings.empty",
        "prompt-settings.reload-error",
        "prompt-settings.reload-retry"
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
      "id": "prompt-settings.load-error",
      "role": "alert",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.load-error",
      "test_id": "prompt-settings-load-error"
    },
    {
      "id": "prompt-settings.loading",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.loading",
      "test_id": "prompt-settings-loading"
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
        "prompt-settings.prompt.edit",
        "prompt-settings.prompt.status"
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
      "text": "Ctrl + Option + M"
    },
    {
      "id": "prompt-settings.prompt.status",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.prompt.status",
      "test_id": "prompt-settings-prompt-status"
    },
    {
      "id": "prompt-settings.reload-error",
      "role": "alert",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.reload-error",
      "test_id": "prompt-settings-reload-error"
    },
    {
      "action": "prompt-settings.reload-retry",
      "id": "prompt-settings.reload-retry",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "prompt-settings.reload-retry",
      "test_id": "prompt-settings-reload-retry"
    },
    {
      "children": [
        "shortcut-settings.ai-dictation.title",
        "shortcut-settings.ai-dictation.list",
        "shortcut-settings.ai-dictation.last",
        "shortcut-settings.ai-dictation.add",
        "shortcut-settings.ai-dictation.reset"
      ],
      "id": "shortcut-settings.ai-dictation",
      "role": "container",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation",
      "test_id": "shortcut-settings-ai-dictation"
    },
    {
      "action": "shortcut-settings.ai-dictation.add",
      "id": "shortcut-settings.ai-dictation.add",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.add",
      "test_id": "shortcut-settings-ai-dictation-add",
      "text": "Add shortcut"
    },
    {
      "id": "shortcut-settings.ai-dictation.last",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.last",
      "test_id": "shortcut-settings-ai-dictation-last",
      "text": "Each action keeps at least one shortcut. Change or reset it instead."
    },
    {
      "children": [
        "shortcut-settings.ai-dictation.shortcut"
      ],
      "id": "shortcut-settings.ai-dictation.list",
      "repeat_model": {
        "collection": "shortcuts.ai_dictation",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.list",
      "test_id": "shortcut-settings-ai-dictation-list"
    },
    {
      "action": "shortcut-settings.ai-dictation.reset",
      "id": "shortcut-settings.ai-dictation.reset",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.reset",
      "test_id": "shortcut-settings-ai-dictation-reset"
    },
    {
      "children": [
        "shortcut-settings.ai-dictation.shortcut.remove"
      ],
      "id": "shortcut-settings.ai-dictation.shortcut",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.shortcut",
      "test_id": "shortcut-settings-ai-dictation-shortcut",
      "text": "⌥ ⇧ Space"
    },
    {
      "action": "shortcut-settings.ai-dictation.shortcut.remove",
      "id": "shortcut-settings.ai-dictation.shortcut.remove",
      "role": "button",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.shortcut.remove",
      "test_id": "shortcut-settings-ai-dictation-shortcut-remove"
    },
    {
      "id": "shortcut-settings.ai-dictation.title",
      "role": "text",
      "screen_id": "prompt-settings",
      "semantic_id": "shortcut-settings.ai-dictation.title",
      "test_id": "shortcut-settings-ai-dictation-title",
      "text": "Post-Processing Hotkey"
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
      },
      {
        "id": "shortcut-removed"
      },
      {
        "id": "recording"
      },
      {
        "id": "reset"
      },
      {
        "id": "reloaded"
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
