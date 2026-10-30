<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: vocabulary-settings-unsupported

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "vocabulary-settings-unsupported.add",
      "kind": "state_change",
      "node": "vocabulary-settings-unsupported.add",
      "success_transition": "vocabulary-settings-unsupported.add.term-added",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "vocabulary_draft",
        "field": "term"
      },
      "event": "change",
      "id": "vocabulary-settings-unsupported.new-term",
      "kind": "state_change",
      "node": "vocabulary-settings-unsupported.new-term",
      "trigger": "change"
    },
    {
      "destination": {
        "screen": "vocabulary-settings-paste",
        "state": "default"
      },
      "event": "navigate",
      "id": "vocabulary-settings-unsupported.paste",
      "kind": "navigation",
      "node": "vocabulary-settings-unsupported.paste",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "vocabulary-settings-unsupported.term.remove",
      "kind": "state_change",
      "node": "vocabulary-settings-unsupported.term.remove",
      "success_transition": "vocabulary-settings-unsupported.term.remove.term-removed",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "vocabulary-settings-unsupported.vocabulary"
      ],
      "id": "vocabulary-settings-unsupported",
      "role": "screen",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported",
      "test_id": "vocabulary-settings-unsupported-screen"
    },
    {
      "action": "vocabulary-settings-unsupported.add",
      "id": "vocabulary-settings-unsupported.add",
      "role": "button",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.add",
      "test_id": "vocabulary-settings-unsupported-add",
      "text": "Add"
    },
    {
      "id": "vocabulary-settings-unsupported.count",
      "role": "text",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.count",
      "test_id": "vocabulary-settings-unsupported-count",
      "text": "3 of 500 terms"
    },
    {
      "id": "vocabulary-settings-unsupported.error",
      "role": "alert",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.error",
      "states": [
        "error"
      ],
      "test_id": "vocabulary-settings-unsupported-error",
      "text": "The vocabulary changed in the meantime and was reloaded. Try again."
    },
    {
      "children": [
        "vocabulary-settings-unsupported.term"
      ],
      "id": "vocabulary-settings-unsupported.list",
      "repeat_model": {
        "collection": "vocabulary.terms",
        "instance_id_template": "term-{index}",
        "item_key": "index",
        "item_model": "vocabulary_term"
      },
      "role": "list",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.list",
      "test_id": "vocabulary-settings-unsupported-list"
    },
    {
      "id": "vocabulary-settings-unsupported.list-error",
      "role": "alert",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.list-error",
      "states": [
        "list-error"
      ],
      "test_id": "vocabulary-settings-unsupported-list-error",
      "text": "Couldn't read the saved vocabulary. Nothing was changed; retry to load it."
    },
    {
      "id": "vocabulary-settings-unsupported.loading",
      "role": "text",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.loading",
      "states": [
        "loading"
      ],
      "test_id": "vocabulary-settings-unsupported-loading",
      "text": "Loading the vocabulary…"
    },
    {
      "action": "vocabulary-settings-unsupported.new-term",
      "binding": "vocabulary_draft.term",
      "id": "vocabulary-settings-unsupported.new-term",
      "role": "input",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.new-term",
      "test_id": "vocabulary-settings-unsupported-new-term",
      "text": "Add a term"
    },
    {
      "action": "vocabulary-settings-unsupported.paste",
      "id": "vocabulary-settings-unsupported.paste",
      "role": "button",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.paste",
      "test_id": "vocabulary-settings-unsupported-paste",
      "text": "Paste a list…"
    },
    {
      "children": [
        "vocabulary-settings-unsupported.status.title"
      ],
      "id": "vocabulary-settings-unsupported.status",
      "role": "container",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.status",
      "test_id": "vocabulary-settings-unsupported-status"
    },
    {
      "id": "vocabulary-settings-unsupported.status.title",
      "role": "text",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.status.title",
      "test_id": "vocabulary-settings-unsupported-status-title",
      "text": "Parakeet V3 can't use the vocabulary during recognition"
    },
    {
      "children": [
        "vocabulary-settings-unsupported.term.remove"
      ],
      "id": "vocabulary-settings-unsupported.term",
      "role": "text",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.term",
      "test_id": "vocabulary-settings-unsupported-term",
      "text": "Claude Code"
    },
    {
      "action": "vocabulary-settings-unsupported.term.remove",
      "id": "vocabulary-settings-unsupported.term.remove",
      "role": "button",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.term.remove",
      "test_id": "vocabulary-settings-unsupported-term-remove"
    },
    {
      "children": [
        "vocabulary-settings-unsupported.terms.title",
        "vocabulary-settings-unsupported.status",
        "vocabulary-settings-unsupported.list",
        "vocabulary-settings-unsupported.new-term",
        "vocabulary-settings-unsupported.add",
        "vocabulary-settings-unsupported.paste",
        "vocabulary-settings-unsupported.loading",
        "vocabulary-settings-unsupported.list-error",
        "vocabulary-settings-unsupported.error",
        "vocabulary-settings-unsupported.count"
      ],
      "id": "vocabulary-settings-unsupported.terms",
      "role": "container",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.terms",
      "test_id": "vocabulary-settings-unsupported-terms"
    },
    {
      "id": "vocabulary-settings-unsupported.terms.title",
      "role": "text",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.terms.title",
      "test_id": "vocabulary-settings-unsupported-terms-title",
      "text": "Vocabulary"
    },
    {
      "children": [
        "vocabulary-settings-unsupported.terms"
      ],
      "id": "vocabulary-settings-unsupported.vocabulary",
      "role": "container",
      "screen_id": "vocabulary-settings-unsupported",
      "semantic_id": "vocabulary-settings-unsupported.vocabulary",
      "test_id": "vocabulary-settings-unsupported-vocabulary",
      "text": "Vocabulary"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recognition-vocabulary",
    "id": "vocabulary-settings-unsupported",
    "root_node": "vocabulary-settings-unsupported",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "term-added"
      },
      {
        "id": "term-removed"
      },
      {
        "id": "loading"
      },
      {
        "id": "list-error"
      },
      {
        "id": "error"
      }
    ],
    "surface": "desktop",
    "title": "Vocabulary — model without support",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
