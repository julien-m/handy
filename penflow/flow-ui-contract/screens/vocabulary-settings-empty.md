<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: vocabulary-settings-empty

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "vocabulary-settings-empty.add",
      "kind": "state_change",
      "node": "vocabulary-settings-empty.add",
      "success_transition": "vocabulary-settings-empty.add.term-added",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "vocabulary_draft",
        "field": "term"
      },
      "event": "change",
      "id": "vocabulary-settings-empty.new-term",
      "kind": "state_change",
      "node": "vocabulary-settings-empty.new-term",
      "trigger": "change"
    },
    {
      "destination": {
        "screen": "vocabulary-settings-paste",
        "state": "default"
      },
      "event": "navigate",
      "id": "vocabulary-settings-empty.paste",
      "kind": "navigation",
      "node": "vocabulary-settings-empty.paste",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "vocabulary-settings-empty.vocabulary"
      ],
      "id": "vocabulary-settings-empty",
      "role": "screen",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty",
      "test_id": "vocabulary-settings-empty-screen"
    },
    {
      "action": "vocabulary-settings-empty.add",
      "id": "vocabulary-settings-empty.add",
      "role": "button",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.add",
      "test_id": "vocabulary-settings-empty-add",
      "text": "Add"
    },
    {
      "children": [
        "vocabulary-settings-empty.empty.title",
        "vocabulary-settings-empty.empty.body"
      ],
      "id": "vocabulary-settings-empty.empty",
      "role": "container",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.empty",
      "test_id": "vocabulary-settings-empty-empty"
    },
    {
      "id": "vocabulary-settings-empty.empty.body",
      "role": "text",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.empty.body",
      "test_id": "vocabulary-settings-empty-empty-body",
      "text": "Add names such as Claude Code or OpenClaw, one at a time or as a pasted list."
    },
    {
      "id": "vocabulary-settings-empty.empty.title",
      "role": "text",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.empty.title",
      "test_id": "vocabulary-settings-empty-empty-title",
      "text": "No terms yet"
    },
    {
      "id": "vocabulary-settings-empty.error",
      "role": "alert",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.error",
      "states": [
        "error"
      ],
      "test_id": "vocabulary-settings-empty-error",
      "text": "The vocabulary changed in the meantime and was reloaded. Try again."
    },
    {
      "id": "vocabulary-settings-empty.list-error",
      "role": "alert",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.list-error",
      "states": [
        "list-error"
      ],
      "test_id": "vocabulary-settings-empty-list-error",
      "text": "Couldn't read the saved vocabulary. Nothing was changed; retry to load it."
    },
    {
      "id": "vocabulary-settings-empty.loading",
      "role": "text",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.loading",
      "states": [
        "loading"
      ],
      "test_id": "vocabulary-settings-empty-loading",
      "text": "Loading the vocabulary…"
    },
    {
      "action": "vocabulary-settings-empty.new-term",
      "binding": "vocabulary_draft.term",
      "id": "vocabulary-settings-empty.new-term",
      "role": "input",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.new-term",
      "test_id": "vocabulary-settings-empty-new-term",
      "text": "Add a term"
    },
    {
      "action": "vocabulary-settings-empty.paste",
      "id": "vocabulary-settings-empty.paste",
      "role": "button",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.paste",
      "test_id": "vocabulary-settings-empty-paste",
      "text": "Paste a list…"
    },
    {
      "children": [
        "vocabulary-settings-empty.status.title"
      ],
      "id": "vocabulary-settings-empty.status",
      "role": "container",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.status",
      "test_id": "vocabulary-settings-empty-status"
    },
    {
      "id": "vocabulary-settings-empty.status.title",
      "role": "text",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.status.title",
      "test_id": "vocabulary-settings-empty-status-title",
      "text": "Support unknown until the model loads"
    },
    {
      "children": [
        "vocabulary-settings-empty.terms.title",
        "vocabulary-settings-empty.status",
        "vocabulary-settings-empty.empty",
        "vocabulary-settings-empty.new-term",
        "vocabulary-settings-empty.add",
        "vocabulary-settings-empty.paste",
        "vocabulary-settings-empty.loading",
        "vocabulary-settings-empty.list-error",
        "vocabulary-settings-empty.error"
      ],
      "id": "vocabulary-settings-empty.terms",
      "role": "container",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.terms",
      "test_id": "vocabulary-settings-empty-terms"
    },
    {
      "id": "vocabulary-settings-empty.terms.title",
      "role": "text",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.terms.title",
      "test_id": "vocabulary-settings-empty-terms-title",
      "text": "Vocabulary"
    },
    {
      "children": [
        "vocabulary-settings-empty.terms"
      ],
      "id": "vocabulary-settings-empty.vocabulary",
      "role": "container",
      "screen_id": "vocabulary-settings-empty",
      "semantic_id": "vocabulary-settings-empty.vocabulary",
      "test_id": "vocabulary-settings-empty-vocabulary",
      "text": "Vocabulary"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recognition-vocabulary",
    "id": "vocabulary-settings-empty",
    "root_node": "vocabulary-settings-empty",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "term-added"
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
    "title": "Vocabulary — empty, model loading",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
