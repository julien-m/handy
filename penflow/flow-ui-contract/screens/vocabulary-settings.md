<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: vocabulary-settings

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "vocabulary-settings.add",
      "kind": "state_change",
      "node": "vocabulary-settings.add",
      "success_transition": "vocabulary-settings.add.term-added",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "vocabulary_draft",
        "field": "term"
      },
      "event": "change",
      "id": "vocabulary-settings.new-term",
      "kind": "state_change",
      "node": "vocabulary-settings.new-term",
      "trigger": "change"
    },
    {
      "destination": {
        "screen": "vocabulary-settings-paste",
        "state": "default"
      },
      "event": "navigate",
      "id": "vocabulary-settings.paste",
      "kind": "navigation",
      "node": "vocabulary-settings.paste",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "vocabulary-settings.term.remove",
      "kind": "state_change",
      "node": "vocabulary-settings.term.remove",
      "success_transition": "vocabulary-settings.term.remove.term-removed",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "vocabulary-settings.vocabulary"
      ],
      "id": "vocabulary-settings",
      "role": "screen",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings",
      "test_id": "vocabulary-settings-screen"
    },
    {
      "action": "vocabulary-settings.add",
      "id": "vocabulary-settings.add",
      "role": "button",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.add",
      "test_id": "vocabulary-settings-add",
      "text": "Add"
    },
    {
      "id": "vocabulary-settings.count",
      "role": "text",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.count",
      "test_id": "vocabulary-settings-count",
      "text": "45 of 500 terms"
    },
    {
      "children": [
        "vocabulary-settings.term"
      ],
      "id": "vocabulary-settings.list",
      "repeat_model": {
        "collection": "vocabulary.terms",
        "instance_id_template": "term-{key}",
        "item_key": "key",
        "item_model": "vocabulary_term"
      },
      "role": "list",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.list",
      "test_id": "vocabulary-settings-list"
    },
    {
      "action": "vocabulary-settings.new-term",
      "binding": "vocabulary_draft.term",
      "id": "vocabulary-settings.new-term",
      "role": "input",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.new-term",
      "test_id": "vocabulary-settings-new-term",
      "text": "Add a term"
    },
    {
      "action": "vocabulary-settings.paste",
      "id": "vocabulary-settings.paste",
      "role": "button",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.paste",
      "test_id": "vocabulary-settings-paste",
      "text": "Paste a list…"
    },
    {
      "children": [
        "vocabulary-settings.status.title"
      ],
      "id": "vocabulary-settings.status",
      "role": "alert",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.status",
      "test_id": "vocabulary-settings-status"
    },
    {
      "id": "vocabulary-settings.status.title",
      "role": "text",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.status.title",
      "test_id": "vocabulary-settings-status-title",
      "text": "Used during recognition by Whisper Small"
    },
    {
      "children": [
        "vocabulary-settings.term.remove"
      ],
      "id": "vocabulary-settings.term",
      "role": "text",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.term",
      "test_id": "vocabulary-settings-term",
      "text": "Claude Code"
    },
    {
      "action": "vocabulary-settings.term.remove",
      "id": "vocabulary-settings.term.remove",
      "role": "button",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.term.remove",
      "test_id": "vocabulary-settings-term-remove"
    },
    {
      "children": [
        "vocabulary-settings.terms.title",
        "vocabulary-settings.status",
        "vocabulary-settings.list",
        "vocabulary-settings.new-term",
        "vocabulary-settings.add",
        "vocabulary-settings.paste",
        "vocabulary-settings.count"
      ],
      "id": "vocabulary-settings.terms",
      "role": "container",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.terms",
      "test_id": "vocabulary-settings-terms"
    },
    {
      "id": "vocabulary-settings.terms.title",
      "role": "text",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.terms.title",
      "test_id": "vocabulary-settings-terms-title",
      "text": "Vocabulary"
    },
    {
      "children": [
        "vocabulary-settings.terms"
      ],
      "id": "vocabulary-settings.vocabulary",
      "role": "container",
      "screen_id": "vocabulary-settings",
      "semantic_id": "vocabulary-settings.vocabulary",
      "test_id": "vocabulary-settings-vocabulary",
      "text": "Vocabulary"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recognition-vocabulary",
    "id": "vocabulary-settings",
    "root_node": "vocabulary-settings",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "term-added"
      },
      {
        "id": "term-removed"
      }
    ],
    "surface": "desktop",
    "title": "Vocabulary",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
