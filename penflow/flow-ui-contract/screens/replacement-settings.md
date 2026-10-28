<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: replacement-settings

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.add",
      "kind": "state_change",
      "node": "replacement-settings.add",
      "success_transition": "replacement-settings.add.rule-added",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.commit-failure.reload",
      "kind": "state_change",
      "node": "replacement-settings.commit-failure.reload",
      "success_transition": "replacement-settings.commit-failure.reload.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.delete-dialog.cancel",
      "kind": "state_change",
      "node": "replacement-settings.delete-dialog.cancel",
      "success_transition": "replacement-settings.delete-dialog.cancel.delete-cancelled",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.delete-dialog.confirm",
      "kind": "state_change",
      "node": "replacement-settings.delete-dialog.confirm",
      "success_transition": "replacement-settings.delete-dialog.confirm.rule-removed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-dirty",
      "id": "replacement-settings.discard",
      "kind": "state_change",
      "node": "replacement-settings.discard",
      "success_transition": "replacement-settings.discard.discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.export",
      "kind": "state_change",
      "node": "replacement-settings.export",
      "success_transition": "replacement-settings.export.exporting",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-import",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings.import",
      "kind": "navigation",
      "node": "replacement-settings.import",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "replacement_draft",
        "field": "recognized"
      },
      "event": "change",
      "id": "replacement-settings.new-recognized",
      "kind": "state_change",
      "node": "replacement-settings.new-recognized",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "replacement_draft",
        "field": "replacement"
      },
      "event": "change",
      "id": "replacement-settings.new-replacement",
      "kind": "state_change",
      "node": "replacement-settings.new-replacement",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.rule.delete",
      "kind": "state_change",
      "node": "replacement-settings.rule.delete",
      "success_transition": "replacement-settings.rule.delete.confirm-delete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.rule.edit",
      "kind": "state_change",
      "node": "replacement-settings.rule.edit",
      "success_transition": "replacement-settings.rule.edit.editing",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "replacement-settings.save",
      "kind": "state_change",
      "node": "replacement-settings.save",
      "success_transition": "replacement-settings.save.saved",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.unsaved.discard",
      "kind": "state_change",
      "node": "replacement-settings.unsaved.discard",
      "success_transition": "replacement-settings.unsaved.discard.left",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings.unsaved.stay",
      "kind": "state_change",
      "node": "replacement-settings.unsaved.stay",
      "success_transition": "replacement-settings.unsaved.stay.stayed",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings.view-list",
      "kind": "navigation",
      "node": "replacement-settings.view-list",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings.view-text",
      "kind": "navigation",
      "node": "replacement-settings.view-text",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "replacement-settings.replacements",
        "replacement-settings.draft",
        "replacement-settings.commit-failure",
        "replacement-settings.unsaved",
        "replacement-settings.delete-dialog"
      ],
      "id": "replacement-settings",
      "role": "screen",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings",
      "test_id": "replacement-settings-screen"
    },
    {
      "action": "replacement-settings.add",
      "id": "replacement-settings.add",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.add",
      "test_id": "replacement-settings-add",
      "text": "Add rule"
    },
    {
      "children": [
        "replacement-settings.commit-failure.message",
        "replacement-settings.commit-failure.reload"
      ],
      "id": "replacement-settings.commit-failure",
      "role": "alert",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.commit-failure",
      "test_id": "replacement-settings-commit-failure"
    },
    {
      "id": "replacement-settings.commit-failure.message",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.commit-failure.message",
      "test_id": "replacement-settings-commit-failure-message",
      "text": "The rules changed elsewhere, for example from the command line. Reload them to continue. Reloading shows the saved rules and discards your unsaved changes. Saved rules are unchanged."
    },
    {
      "action": "replacement-settings.commit-failure.reload",
      "id": "replacement-settings.commit-failure.reload",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.commit-failure.reload",
      "test_id": "replacement-settings-commit-failure-reload",
      "text": "Reload"
    },
    {
      "id": "replacement-settings.count",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.count",
      "test_id": "replacement-settings-count",
      "text": "3 of 1,000 rules · case-insensitive, whole words"
    },
    {
      "children": [
        "replacement-settings.delete-dialog.body",
        "replacement-settings.delete-dialog.cancel",
        "replacement-settings.delete-dialog.confirm"
      ],
      "id": "replacement-settings.delete-dialog",
      "role": "modal",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.delete-dialog",
      "test_id": "replacement-settings-delete-dialog",
      "text": "Delete this rule?"
    },
    {
      "id": "replacement-settings.delete-dialog.body",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.delete-dialog.body",
      "test_id": "replacement-settings-delete-dialog-body",
      "text": "Cloud Code → Claude Code will be removed from the list. Save to apply the change."
    },
    {
      "action": "replacement-settings.delete-dialog.cancel",
      "id": "replacement-settings.delete-dialog.cancel",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.delete-dialog.cancel",
      "test_id": "replacement-settings-delete-dialog-cancel",
      "text": "Cancel"
    },
    {
      "action": "replacement-settings.delete-dialog.confirm",
      "id": "replacement-settings.delete-dialog.confirm",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.delete-dialog.confirm",
      "test_id": "replacement-settings-delete-dialog-confirm",
      "text": "Delete"
    },
    {
      "action": "replacement-settings.discard",
      "id": "replacement-settings.discard",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.discard",
      "test_id": "replacement-settings-discard",
      "text": "Discard"
    },
    {
      "children": [
        "replacement-settings.draft.note",
        "replacement-settings.discard",
        "replacement-settings.save"
      ],
      "id": "replacement-settings.draft",
      "role": "container",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.draft",
      "test_id": "replacement-settings-draft"
    },
    {
      "id": "replacement-settings.draft.note",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.draft.note",
      "test_id": "replacement-settings-draft-note",
      "text": "Unsaved changes. Dictations use the saved rules until you save."
    },
    {
      "action": "replacement-settings.export",
      "id": "replacement-settings.export",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.export",
      "test_id": "replacement-settings-export",
      "text": "Export…"
    },
    {
      "action": "replacement-settings.import",
      "id": "replacement-settings.import",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.import",
      "test_id": "replacement-settings-import",
      "text": "Import…"
    },
    {
      "children": [
        "replacement-settings.rule"
      ],
      "id": "replacement-settings.list",
      "repeat_model": {
        "collection": "replacement_rules.rules",
        "instance_id_template": "rule-{key}",
        "item_key": "key",
        "item_model": "replacement_rule"
      },
      "role": "list",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.list",
      "test_id": "replacement-settings-list"
    },
    {
      "action": "replacement-settings.new-recognized",
      "binding": "replacement_draft.recognized",
      "id": "replacement-settings.new-recognized",
      "role": "input",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.new-recognized",
      "test_id": "replacement-settings-new-recognized",
      "text": "Recognized text"
    },
    {
      "action": "replacement-settings.new-replacement",
      "binding": "replacement_draft.replacement",
      "id": "replacement-settings.new-replacement",
      "role": "input",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.new-replacement",
      "test_id": "replacement-settings-new-replacement",
      "text": "Replacement"
    },
    {
      "children": [
        "replacement-settings.rules"
      ],
      "id": "replacement-settings.replacements",
      "role": "container",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.replacements",
      "test_id": "replacement-settings-replacements",
      "text": "Replacements"
    },
    {
      "children": [
        "replacement-settings.rule.recognized",
        "replacement-settings.rule.replacement",
        "replacement-settings.rule.edit",
        "replacement-settings.rule.delete"
      ],
      "id": "replacement-settings.rule",
      "role": "container",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rule",
      "test_id": "replacement-settings-rule"
    },
    {
      "action": "replacement-settings.rule.delete",
      "id": "replacement-settings.rule.delete",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rule.delete",
      "test_id": "replacement-settings-rule-delete"
    },
    {
      "action": "replacement-settings.rule.edit",
      "id": "replacement-settings.rule.edit",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rule.edit",
      "test_id": "replacement-settings-rule-edit"
    },
    {
      "id": "replacement-settings.rule.recognized",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rule.recognized",
      "test_id": "replacement-settings-rule-recognized",
      "text": "Cloud Code"
    },
    {
      "id": "replacement-settings.rule.replacement",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rule.replacement",
      "test_id": "replacement-settings-rule-replacement",
      "text": "Claude Code"
    },
    {
      "children": [
        "replacement-settings.rules.title",
        "replacement-settings.views",
        "replacement-settings.import",
        "replacement-settings.export",
        "replacement-settings.new-recognized",
        "replacement-settings.new-replacement",
        "replacement-settings.add",
        "replacement-settings.list",
        "replacement-settings.count"
      ],
      "id": "replacement-settings.rules",
      "role": "container",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rules",
      "test_id": "replacement-settings-rules"
    },
    {
      "id": "replacement-settings.rules.title",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.rules.title",
      "test_id": "replacement-settings-rules-title",
      "text": "Replacement rules"
    },
    {
      "action": "replacement-settings.save",
      "id": "replacement-settings.save",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.save",
      "test_id": "replacement-settings-save",
      "text": "Save"
    },
    {
      "children": [
        "replacement-settings.unsaved.body",
        "replacement-settings.unsaved.stay",
        "replacement-settings.unsaved.discard"
      ],
      "id": "replacement-settings.unsaved",
      "role": "modal",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.unsaved",
      "test_id": "replacement-settings-unsaved",
      "text": "Unsaved changes"
    },
    {
      "id": "replacement-settings.unsaved.body",
      "role": "text",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.unsaved.body",
      "test_id": "replacement-settings-unsaved-body",
      "text": "Some of your replacement rules are not saved. Stay to save them, or discard them and leave."
    },
    {
      "action": "replacement-settings.unsaved.discard",
      "id": "replacement-settings.unsaved.discard",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.unsaved.discard",
      "test_id": "replacement-settings-unsaved-discard",
      "text": "Discard and leave"
    },
    {
      "action": "replacement-settings.unsaved.stay",
      "id": "replacement-settings.unsaved.stay",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.unsaved.stay",
      "test_id": "replacement-settings-unsaved-stay",
      "text": "Stay"
    },
    {
      "action": "replacement-settings.view-list",
      "id": "replacement-settings.view-list",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.view-list",
      "test_id": "replacement-settings-view-list",
      "text": "List"
    },
    {
      "action": "replacement-settings.view-text",
      "id": "replacement-settings.view-text",
      "role": "button",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.view-text",
      "test_id": "replacement-settings-view-text",
      "text": "Text"
    },
    {
      "children": [
        "replacement-settings.view-list",
        "replacement-settings.view-text"
      ],
      "id": "replacement-settings.views",
      "role": "container",
      "screen_id": "replacement-settings",
      "semantic_id": "replacement-settings.views",
      "test_id": "replacement-settings-views"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "word-replacements",
    "id": "replacement-settings",
    "root_node": "replacement-settings",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "confirm-delete"
      },
      {
        "id": "discarded"
      },
      {
        "id": "editing"
      },
      {
        "id": "exporting"
      },
      {
        "id": "rule-added"
      },
      {
        "id": "saved"
      },
      {
        "id": "reloaded"
      },
      {
        "id": "stayed"
      },
      {
        "id": "left"
      },
      {
        "id": "delete-cancelled"
      },
      {
        "id": "rule-removed"
      }
    ],
    "surface": "desktop",
    "title": "Word replacements",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
