<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: replacement-settings-empty

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-empty.empty.add",
      "kind": "state_change",
      "node": "replacement-settings-empty.empty.add",
      "success_transition": "replacement-settings-empty.empty.add.adding",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.empty.paste",
      "kind": "navigation",
      "node": "replacement-settings-empty.empty.paste",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-empty.export",
      "kind": "state_change",
      "node": "replacement-settings-empty.export",
      "success_transition": "replacement-settings-empty.export.exporting",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-import",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.import",
      "kind": "navigation",
      "node": "replacement-settings-empty.import",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.view-list",
      "kind": "navigation",
      "node": "replacement-settings-empty.view-list",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.view-text",
      "kind": "navigation",
      "node": "replacement-settings-empty.view-text",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "replacement-settings-empty.replacements"
      ],
      "id": "replacement-settings-empty",
      "role": "screen",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty",
      "test_id": "replacement-settings-empty-screen"
    },
    {
      "children": [
        "replacement-settings-empty.empty.title",
        "replacement-settings-empty.empty.body",
        "replacement-settings-empty.empty.add",
        "replacement-settings-empty.empty.paste"
      ],
      "id": "replacement-settings-empty.empty",
      "role": "container",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.empty",
      "test_id": "replacement-settings-empty-empty"
    },
    {
      "action": "replacement-settings-empty.empty.add",
      "id": "replacement-settings-empty.empty.add",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.empty.add",
      "test_id": "replacement-settings-empty-empty-add",
      "text": "Add rule"
    },
    {
      "id": "replacement-settings-empty.empty.body",
      "role": "text",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.empty.body",
      "test_id": "replacement-settings-empty-empty-body",
      "text": "Add a rule, or paste lines such as  Cloud Code;Claude Code"
    },
    {
      "action": "replacement-settings-empty.empty.paste",
      "id": "replacement-settings-empty.empty.paste",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.empty.paste",
      "test_id": "replacement-settings-empty-empty-paste",
      "text": "Paste rules as text"
    },
    {
      "id": "replacement-settings-empty.empty.title",
      "role": "text",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.empty.title",
      "test_id": "replacement-settings-empty-empty-title",
      "text": "No replacement rules yet"
    },
    {
      "action": "replacement-settings-empty.export",
      "id": "replacement-settings-empty.export",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.export",
      "test_id": "replacement-settings-empty-export",
      "text": "Export…"
    },
    {
      "action": "replacement-settings-empty.import",
      "id": "replacement-settings-empty.import",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.import",
      "test_id": "replacement-settings-empty-import",
      "text": "Import…"
    },
    {
      "children": [
        "replacement-settings-empty.rules"
      ],
      "id": "replacement-settings-empty.replacements",
      "role": "container",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.replacements",
      "test_id": "replacement-settings-empty-replacements",
      "text": "Replacements"
    },
    {
      "children": [
        "replacement-settings-empty.rules.title",
        "replacement-settings-empty.views",
        "replacement-settings-empty.import",
        "replacement-settings-empty.export",
        "replacement-settings-empty.empty"
      ],
      "id": "replacement-settings-empty.rules",
      "role": "container",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.rules",
      "test_id": "replacement-settings-empty-rules"
    },
    {
      "id": "replacement-settings-empty.rules.title",
      "role": "text",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.rules.title",
      "test_id": "replacement-settings-empty-rules-title",
      "text": "Replacement rules"
    },
    {
      "action": "replacement-settings-empty.view-list",
      "id": "replacement-settings-empty.view-list",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.view-list",
      "test_id": "replacement-settings-empty-view-list",
      "text": "List"
    },
    {
      "action": "replacement-settings-empty.view-text",
      "id": "replacement-settings-empty.view-text",
      "role": "button",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.view-text",
      "test_id": "replacement-settings-empty-view-text",
      "text": "Text"
    },
    {
      "children": [
        "replacement-settings-empty.view-list",
        "replacement-settings-empty.view-text"
      ],
      "id": "replacement-settings-empty.views",
      "role": "container",
      "screen_id": "replacement-settings-empty",
      "semantic_id": "replacement-settings-empty.views",
      "test_id": "replacement-settings-empty-views"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "word-replacements",
    "id": "replacement-settings-empty",
    "root_node": "replacement-settings-empty",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "adding"
      },
      {
        "id": "exporting"
      }
    ],
    "surface": "desktop",
    "title": "Word replacements — empty",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
