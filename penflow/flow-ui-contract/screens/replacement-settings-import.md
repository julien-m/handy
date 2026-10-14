<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: replacement-settings-import

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "replacement-settings-import.dialog.apply",
      "kind": "state_change",
      "node": "replacement-settings-import.dialog.apply",
      "success_transition": "replacement-settings-import.dialog.apply.imported",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-import.dialog.cancel",
      "kind": "navigation",
      "node": "replacement-settings-import.dialog.cancel",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "replacement_import",
        "field": "mode"
      },
      "event": "change",
      "id": "replacement-settings-import.dialog.merge",
      "kind": "state_change",
      "node": "replacement-settings-import.dialog.merge",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "replacement_import",
        "field": "replace"
      },
      "event": "change",
      "id": "replacement-settings-import.dialog.replace",
      "kind": "state_change",
      "node": "replacement-settings-import.dialog.replace",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "replacement-settings-import.unreadable",
        "replacement-settings-import.dialog"
      ],
      "id": "replacement-settings-import",
      "role": "screen",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import",
      "test_id": "replacement-settings-import-screen"
    },
    {
      "children": [
        "replacement-settings-import.dialog.file",
        "replacement-settings-import.dialog.mode-label",
        "replacement-settings-import.dialog.merge",
        "replacement-settings-import.dialog.replace",
        "replacement-settings-import.dialog.summary",
        "replacement-settings-import.dialog.cancel",
        "replacement-settings-import.dialog.apply"
      ],
      "id": "replacement-settings-import.dialog",
      "role": "modal",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog",
      "test_id": "replacement-settings-import-dialog",
      "text": "Import rules"
    },
    {
      "id": "replacement-settings-import.dialog.adds",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.adds",
      "test_id": "replacement-settings-import-dialog-adds",
      "text": "5 additions"
    },
    {
      "action": "replacement-settings-import.dialog.apply",
      "id": "replacement-settings-import.dialog.apply",
      "role": "button",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.apply",
      "test_id": "replacement-settings-import-dialog-apply",
      "text": "Apply import"
    },
    {
      "action": "replacement-settings-import.dialog.cancel",
      "id": "replacement-settings-import.dialog.cancel",
      "role": "button",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.cancel",
      "test_id": "replacement-settings-import-dialog-cancel",
      "text": "Cancel"
    },
    {
      "id": "replacement-settings-import.dialog.errors",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.errors",
      "test_id": "replacement-settings-import-dialog-errors",
      "text": "0 errors"
    },
    {
      "id": "replacement-settings-import.dialog.example",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.example",
      "test_id": "replacement-settings-import-dialog-example",
      "text": "cloud code → Claude Code CLI (update, keeps position 1)"
    },
    {
      "id": "replacement-settings-import.dialog.file",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.file",
      "test_id": "replacement-settings-import-dialog-file",
      "text": "rules-team.csv · 7 lines · UTF-8"
    },
    {
      "action": "replacement-settings-import.dialog.merge",
      "binding": "replacement_import.mode",
      "id": "replacement-settings-import.dialog.merge",
      "role": "toggle",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.merge",
      "test_id": "replacement-settings-import-dialog-merge",
      "text": "Merge: update matching rules, add new ones at the end"
    },
    {
      "id": "replacement-settings-import.dialog.mode-label",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.mode-label",
      "test_id": "replacement-settings-import-dialog-mode-label",
      "text": "Mode"
    },
    {
      "action": "replacement-settings-import.dialog.replace",
      "binding": "replacement_import.replace",
      "id": "replacement-settings-import.dialog.replace",
      "role": "toggle",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.replace",
      "test_id": "replacement-settings-import-dialog-replace",
      "text": "Replace: the file becomes the whole rule set"
    },
    {
      "children": [
        "replacement-settings-import.dialog.updates",
        "replacement-settings-import.dialog.adds",
        "replacement-settings-import.dialog.errors",
        "replacement-settings-import.dialog.example"
      ],
      "id": "replacement-settings-import.dialog.summary",
      "role": "container",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.summary",
      "test_id": "replacement-settings-import-dialog-summary"
    },
    {
      "id": "replacement-settings-import.dialog.updates",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.dialog.updates",
      "test_id": "replacement-settings-import-dialog-updates",
      "text": "1 update"
    },
    {
      "children": [
        "replacement-settings-import.unreadable.title"
      ],
      "id": "replacement-settings-import.unreadable",
      "role": "alert",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.unreadable",
      "test_id": "replacement-settings-import-unreadable"
    },
    {
      "id": "replacement-settings-import.unreadable.title",
      "role": "text",
      "screen_id": "replacement-settings-import",
      "semantic_id": "replacement-settings-import.unreadable.title",
      "test_id": "replacement-settings-import-unreadable-title",
      "text": "Couldn't import notes.rtf: it is not UTF-8 text"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "word-replacements",
    "id": "replacement-settings-import",
    "root_node": "replacement-settings-import",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "imported"
      }
    ],
    "surface": "desktop",
    "title": "Word replacements — import preview",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
