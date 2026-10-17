<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: vocabulary-settings-paste

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "destination": {
        "screen": "vocabulary-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "vocabulary-settings-paste.apply",
      "kind": "navigation",
      "node": "vocabulary-settings-paste.apply",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "vocabulary-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "vocabulary-settings-paste.cancel",
      "kind": "navigation",
      "node": "vocabulary-settings-paste.cancel",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "vocabulary_draft",
        "field": "text"
      },
      "event": "change",
      "id": "vocabulary-settings-paste.editor",
      "kind": "state_change",
      "node": "vocabulary-settings-paste.editor",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "vocabulary-settings-paste.vocabulary",
        "vocabulary-settings-paste.apply-bar"
      ],
      "id": "vocabulary-settings-paste",
      "role": "screen",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste",
      "test_id": "vocabulary-settings-paste-screen"
    },
    {
      "action": "vocabulary-settings-paste.apply",
      "id": "vocabulary-settings-paste.apply",
      "role": "button",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.apply",
      "test_id": "vocabulary-settings-paste-apply",
      "text": "Add 3 terms"
    },
    {
      "children": [
        "vocabulary-settings-paste.apply-bar.note",
        "vocabulary-settings-paste.cancel",
        "vocabulary-settings-paste.apply"
      ],
      "id": "vocabulary-settings-paste.apply-bar",
      "role": "container",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.apply-bar",
      "test_id": "vocabulary-settings-paste-apply-bar"
    },
    {
      "id": "vocabulary-settings-paste.apply-bar.note",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.apply-bar.note",
      "test_id": "vocabulary-settings-paste-apply-bar-note",
      "text": "Fix line 5 to add the terms. Your vocabulary is unchanged."
    },
    {
      "action": "vocabulary-settings-paste.cancel",
      "id": "vocabulary-settings-paste.cancel",
      "role": "button",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.cancel",
      "test_id": "vocabulary-settings-paste-cancel",
      "text": "Cancel"
    },
    {
      "action": "vocabulary-settings-paste.editor",
      "binding": "vocabulary_draft.text",
      "id": "vocabulary-settings-paste.editor",
      "role": "input",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.editor",
      "test_id": "vocabulary-settings-paste-editor",
      "text": "Claude Code\nClaude Sonnet\nclaude code\nOpenClaw\n<prompt> template"
    },
    {
      "id": "vocabulary-settings-paste.error",
      "role": "alert",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.error",
      "states": [
        "error"
      ],
      "test_id": "vocabulary-settings-paste-error",
      "text": "The vocabulary changed in the meantime and was reloaded. Try again."
    },
    {
      "id": "vocabulary-settings-paste.list-error",
      "role": "alert",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.list-error",
      "states": [
        "list-error"
      ],
      "test_id": "vocabulary-settings-paste-list-error",
      "text": "Couldn't read the saved vocabulary. Nothing was changed; retry to load it."
    },
    {
      "id": "vocabulary-settings-paste.loading",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.loading",
      "states": [
        "loading"
      ],
      "test_id": "vocabulary-settings-paste-loading",
      "text": "Loading the vocabulary…"
    },
    {
      "children": [
        "vocabulary-settings-paste.preview.title",
        "vocabulary-settings-paste.preview.summary",
        "vocabulary-settings-paste.preview.lines"
      ],
      "id": "vocabulary-settings-paste.preview",
      "role": "container",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview",
      "test_id": "vocabulary-settings-paste-preview"
    },
    {
      "children": [
        "vocabulary-settings-paste.preview.line.kind",
        "vocabulary-settings-paste.preview.line.text"
      ],
      "id": "vocabulary-settings-paste.preview.line",
      "role": "container",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.line",
      "test_id": "vocabulary-settings-paste-preview-line"
    },
    {
      "id": "vocabulary-settings-paste.preview.line.kind",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.line.kind",
      "test_id": "vocabulary-settings-paste-preview-line-kind",
      "text": "Add"
    },
    {
      "id": "vocabulary-settings-paste.preview.line.text",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.line.text",
      "test_id": "vocabulary-settings-paste-preview-line-text",
      "text": "Claude Code"
    },
    {
      "children": [
        "vocabulary-settings-paste.preview.line"
      ],
      "id": "vocabulary-settings-paste.preview.lines",
      "repeat_model": {
        "collection": "vocabulary_preview.lines",
        "instance_id_template": "vocabulary-line-{number}",
        "item_key": "number",
        "item_model": "vocabulary_line"
      },
      "role": "list",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.lines",
      "test_id": "vocabulary-settings-paste-preview-lines"
    },
    {
      "id": "vocabulary-settings-paste.preview.summary",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.summary",
      "test_id": "vocabulary-settings-paste-preview-summary",
      "text": "3 to add · 1 duplicate · 1 error"
    },
    {
      "id": "vocabulary-settings-paste.preview.title",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.preview.title",
      "test_id": "vocabulary-settings-paste-preview-title",
      "text": "Preview"
    },
    {
      "children": [
        "vocabulary-settings-paste.terms.title",
        "vocabulary-settings-paste.editor",
        "vocabulary-settings-paste.preview",
        "vocabulary-settings-paste.loading",
        "vocabulary-settings-paste.list-error",
        "vocabulary-settings-paste.error"
      ],
      "id": "vocabulary-settings-paste.terms",
      "role": "container",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.terms",
      "test_id": "vocabulary-settings-paste-terms"
    },
    {
      "id": "vocabulary-settings-paste.terms.title",
      "role": "text",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.terms.title",
      "test_id": "vocabulary-settings-paste-terms-title",
      "text": "Paste terms"
    },
    {
      "children": [
        "vocabulary-settings-paste.terms"
      ],
      "id": "vocabulary-settings-paste.vocabulary",
      "role": "container",
      "screen_id": "vocabulary-settings-paste",
      "semantic_id": "vocabulary-settings-paste.vocabulary",
      "test_id": "vocabulary-settings-paste-vocabulary",
      "text": "Vocabulary"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recognition-vocabulary",
    "id": "vocabulary-settings-paste",
    "root_node": "vocabulary-settings-paste",
    "states": [
      {
        "id": "default"
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
    "title": "Vocabulary — paste preview",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
