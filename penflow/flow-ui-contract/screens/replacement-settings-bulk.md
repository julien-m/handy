<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: replacement-settings-bulk

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "replacement-settings-bulk.apply",
      "kind": "state_change",
      "node": "replacement-settings-bulk.apply",
      "success_transition": "replacement-settings-bulk.apply.applied",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-bulk.cancel",
      "kind": "navigation",
      "node": "replacement-settings-bulk.cancel",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-bulk.commit-failure.reload",
      "kind": "state_change",
      "node": "replacement-settings-bulk.commit-failure.reload",
      "success_transition": "replacement-settings-bulk.commit-failure.reload.reloaded",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "replacement_draft",
        "field": "text"
      },
      "event": "change",
      "id": "replacement-settings-bulk.editor",
      "kind": "state_change",
      "node": "replacement-settings-bulk.editor",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-bulk.export",
      "kind": "state_change",
      "node": "replacement-settings-bulk.export",
      "success_transition": "replacement-settings-bulk.export.exporting",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-import",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-bulk.import",
      "kind": "navigation",
      "node": "replacement-settings-bulk.import",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-bulk.view-list",
      "kind": "navigation",
      "node": "replacement-settings-bulk.view-list",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-bulk.view-text",
      "kind": "navigation",
      "node": "replacement-settings-bulk.view-text",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "replacement-settings-bulk.replacements",
        "replacement-settings-bulk.apply-bar",
        "replacement-settings-bulk.commit-failure"
      ],
      "id": "replacement-settings-bulk",
      "role": "screen",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk",
      "test_id": "replacement-settings-bulk-screen"
    },
    {
      "action": "replacement-settings-bulk.apply",
      "id": "replacement-settings-bulk.apply",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.apply",
      "test_id": "replacement-settings-bulk-apply",
      "text": "Apply"
    },
    {
      "children": [
        "replacement-settings-bulk.apply-bar.note",
        "replacement-settings-bulk.cancel",
        "replacement-settings-bulk.apply"
      ],
      "id": "replacement-settings-bulk.apply-bar",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.apply-bar",
      "test_id": "replacement-settings-bulk-apply-bar"
    },
    {
      "id": "replacement-settings-bulk.apply-bar.note",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.apply-bar.note",
      "test_id": "replacement-settings-bulk-apply-bar-note",
      "text": "Fix line 4 to apply. Saved rules are unchanged."
    },
    {
      "action": "replacement-settings-bulk.cancel",
      "id": "replacement-settings-bulk.cancel",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.cancel",
      "test_id": "replacement-settings-bulk-cancel",
      "text": "Cancel"
    },
    {
      "children": [
        "replacement-settings-bulk.commit-failure.message",
        "replacement-settings-bulk.commit-failure.reload"
      ],
      "id": "replacement-settings-bulk.commit-failure",
      "role": "alert",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.commit-failure",
      "test_id": "replacement-settings-bulk-commit-failure"
    },
    {
      "id": "replacement-settings-bulk.commit-failure.message",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.commit-failure.message",
      "test_id": "replacement-settings-bulk-commit-failure-message",
      "text": "The rules changed elsewhere, for example from the command line. Reload them to continue. Saved rules are unchanged."
    },
    {
      "action": "replacement-settings-bulk.commit-failure.reload",
      "id": "replacement-settings-bulk.commit-failure.reload",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.commit-failure.reload",
      "test_id": "replacement-settings-bulk-commit-failure-reload",
      "text": "Reload"
    },
    {
      "action": "replacement-settings-bulk.editor",
      "binding": "replacement_draft.text",
      "id": "replacement-settings-bulk.editor",
      "role": "input",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.editor",
      "test_id": "replacement-settings-bulk-editor",
      "text": "recognized;replacement\nCloud Code;Claude Code\nCloud Sonnet;Claude Sonnet\nCloud Opus\nOpen Claw;OpenClaw"
    },
    {
      "action": "replacement-settings-bulk.export",
      "id": "replacement-settings-bulk.export",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.export",
      "test_id": "replacement-settings-bulk-export",
      "text": "Export…"
    },
    {
      "id": "replacement-settings-bulk.hint",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.hint",
      "test_id": "replacement-settings-bulk-hint",
      "text": "One rule per line: recognized;replacement. A tab also works for spreadsheet paste. Lines starting with # are ignored."
    },
    {
      "action": "replacement-settings-bulk.import",
      "id": "replacement-settings-bulk.import",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.import",
      "test_id": "replacement-settings-bulk-import",
      "text": "Import…"
    },
    {
      "children": [
        "replacement-settings-bulk.preview.title",
        "replacement-settings-bulk.preview.summary",
        "replacement-settings-bulk.preview.lines"
      ],
      "id": "replacement-settings-bulk.preview",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview",
      "test_id": "replacement-settings-bulk-preview"
    },
    {
      "children": [
        "replacement-settings-bulk.preview.line.number",
        "replacement-settings-bulk.preview.line.kind",
        "replacement-settings-bulk.preview.line.text"
      ],
      "id": "replacement-settings-bulk.preview.line",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.line",
      "test_id": "replacement-settings-bulk-preview-line"
    },
    {
      "id": "replacement-settings-bulk.preview.line.kind",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.line.kind",
      "test_id": "replacement-settings-bulk-preview-line-kind",
      "text": "Ignored"
    },
    {
      "id": "replacement-settings-bulk.preview.line.number",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.line.number",
      "test_id": "replacement-settings-bulk-preview-line-number",
      "text": "1"
    },
    {
      "id": "replacement-settings-bulk.preview.line.text",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.line.text",
      "test_id": "replacement-settings-bulk-preview-line-text",
      "text": "Header line"
    },
    {
      "children": [
        "replacement-settings-bulk.preview.line"
      ],
      "id": "replacement-settings-bulk.preview.lines",
      "repeat_model": {
        "collection": "replacement_preview.lines",
        "instance_id_template": "preview-line-{number}",
        "item_key": "number",
        "item_model": "preview_line"
      },
      "role": "list",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.lines",
      "test_id": "replacement-settings-bulk-preview-lines"
    },
    {
      "id": "replacement-settings-bulk.preview.summary",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.summary",
      "test_id": "replacement-settings-bulk-preview-summary",
      "text": "1 error · Apply unavailable"
    },
    {
      "id": "replacement-settings-bulk.preview.title",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.preview.title",
      "test_id": "replacement-settings-bulk-preview-title",
      "text": "Preview"
    },
    {
      "children": [
        "replacement-settings-bulk.rules"
      ],
      "id": "replacement-settings-bulk.replacements",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.replacements",
      "test_id": "replacement-settings-bulk-replacements",
      "text": "Replacements"
    },
    {
      "children": [
        "replacement-settings-bulk.rules.title",
        "replacement-settings-bulk.views",
        "replacement-settings-bulk.import",
        "replacement-settings-bulk.export",
        "replacement-settings-bulk.editor",
        "replacement-settings-bulk.hint",
        "replacement-settings-bulk.preview"
      ],
      "id": "replacement-settings-bulk.rules",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.rules",
      "test_id": "replacement-settings-bulk-rules"
    },
    {
      "id": "replacement-settings-bulk.rules.title",
      "role": "text",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.rules.title",
      "test_id": "replacement-settings-bulk-rules-title",
      "text": "Replacement rules"
    },
    {
      "action": "replacement-settings-bulk.view-list",
      "id": "replacement-settings-bulk.view-list",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.view-list",
      "test_id": "replacement-settings-bulk-view-list",
      "text": "List"
    },
    {
      "action": "replacement-settings-bulk.view-text",
      "id": "replacement-settings-bulk.view-text",
      "role": "button",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.view-text",
      "test_id": "replacement-settings-bulk-view-text",
      "text": "Text"
    },
    {
      "children": [
        "replacement-settings-bulk.view-list",
        "replacement-settings-bulk.view-text"
      ],
      "id": "replacement-settings-bulk.views",
      "role": "container",
      "screen_id": "replacement-settings-bulk",
      "semantic_id": "replacement-settings-bulk.views",
      "test_id": "replacement-settings-bulk-views"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "word-replacements",
    "id": "replacement-settings-bulk",
    "root_node": "replacement-settings-bulk",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "applied"
      },
      {
        "id": "exporting"
      },
      {
        "id": "reloaded"
      }
    ],
    "surface": "desktop",
    "title": "Word replacements — text and preview",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
