<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: prompt-settings-editor

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "destination": {
        "screen": "prompt-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings-editor.cancel",
      "kind": "navigation",
      "node": "prompt-settings-editor.cancel",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.delete",
      "kind": "state_change",
      "node": "prompt-settings-editor.delete",
      "success_transition": "prompt-settings-editor.delete.confirm-delete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.delete-confirm",
      "kind": "state_change",
      "node": "prompt-settings-editor.delete-confirm",
      "success_transition": "prompt-settings-editor.delete-confirm.deleted",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.form.add-shortcut",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.add-shortcut",
      "success_transition": "prompt-settings-editor.form.add-shortcut.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.form.fix",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.fix",
      "success_transition": "prompt-settings-editor.form.fix.fix-field-open",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "provider_fix",
        "field": "api_key"
      },
      "event": "change",
      "id": "prompt-settings-editor.form.fix-field",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.fix-field",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "model"
      },
      "event": "change",
      "id": "prompt-settings-editor.form.model",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.model",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "name"
      },
      "event": "change",
      "id": "prompt-settings-editor.form.name",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.name",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "provider"
      },
      "event": "change",
      "id": "prompt-settings-editor.form.provider",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.provider",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.form.shortcut-recorder",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.shortcut-recorder",
      "success_transition": "prompt-settings-editor.form.shortcut-recorder.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.form.shortcut.remove",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.shortcut.remove",
      "success_transition": "prompt-settings-editor.form.shortcut.remove.shortcut-removed",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "text"
      },
      "event": "change",
      "id": "prompt-settings-editor.form.text",
      "kind": "state_change",
      "node": "prompt-settings-editor.form.text",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.reload-retry",
      "kind": "state_change",
      "node": "prompt-settings-editor.reload-retry",
      "success_transition": "prompt-settings-editor.reload-retry.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "prompt-settings-editor.save",
      "kind": "state_change",
      "node": "prompt-settings-editor.save",
      "success_transition": "prompt-settings-editor.save.saved",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.test.retry",
      "kind": "state_change",
      "node": "prompt-settings-editor.test.retry",
      "success_transition": "prompt-settings-editor.test.retry.tested",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-editor.test.run",
      "kind": "state_change",
      "node": "prompt-settings-editor.test.run",
      "success_transition": "prompt-settings-editor.test.run.tested",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_test",
        "field": "sample"
      },
      "event": "change",
      "id": "prompt-settings-editor.test.sample",
      "kind": "state_change",
      "node": "prompt-settings-editor.test.sample",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "prompt-settings-editor.editor",
        "prompt-settings-editor.test",
        "prompt-settings-editor.actions",
        "prompt-settings-editor.deleted-elsewhere",
        "prompt-settings-editor.changed-elsewhere",
        "prompt-settings-editor.delete-dialog",
        "prompt-settings-editor.reload-error",
        "prompt-settings-editor.reload-retry"
      ],
      "id": "prompt-settings-editor",
      "role": "screen",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor",
      "test_id": "prompt-settings-editor-screen"
    },
    {
      "children": [
        "prompt-settings-editor.actions.note",
        "prompt-settings-editor.delete",
        "prompt-settings-editor.cancel",
        "prompt-settings-editor.save",
        "prompt-settings-editor.saved",
        "prompt-settings-editor.actions-error"
      ],
      "id": "prompt-settings-editor.actions",
      "role": "container",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.actions",
      "test_id": "prompt-settings-editor-actions"
    },
    {
      "id": "prompt-settings-editor.actions-error",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.actions-error",
      "test_id": "prompt-settings-editor-actions-error"
    },
    {
      "id": "prompt-settings-editor.actions.note",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.actions.note",
      "test_id": "prompt-settings-editor-actions-note",
      "text": "Changes apply to the next dictation after you save."
    },
    {
      "action": "prompt-settings-editor.cancel",
      "id": "prompt-settings-editor.cancel",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.cancel",
      "test_id": "prompt-settings-editor-cancel",
      "text": "Cancel"
    },
    {
      "id": "prompt-settings-editor.changed-elsewhere",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.changed-elsewhere",
      "test_id": "prompt-settings-editor-changed-elsewhere"
    },
    {
      "action": "prompt-settings-editor.delete",
      "id": "prompt-settings-editor.delete",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.delete",
      "test_id": "prompt-settings-editor-delete",
      "text": "Delete prompt"
    },
    {
      "action": "prompt-settings-editor.delete-confirm",
      "id": "prompt-settings-editor.delete-confirm",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.delete-confirm",
      "test_id": "prompt-settings-editor-delete-confirm"
    },
    {
      "children": [
        "prompt-settings-editor.delete-confirm"
      ],
      "id": "prompt-settings-editor.delete-dialog",
      "role": "modal",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.delete-dialog",
      "test_id": "prompt-settings-editor-delete-dialog"
    },
    {
      "id": "prompt-settings-editor.deleted-elsewhere",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.deleted-elsewhere",
      "test_id": "prompt-settings-editor-deleted-elsewhere"
    },
    {
      "children": [
        "prompt-settings-editor.form"
      ],
      "id": "prompt-settings-editor.editor",
      "role": "container",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.editor",
      "test_id": "prompt-settings-editor-editor",
      "text": "Edit prompt"
    },
    {
      "children": [
        "prompt-settings-editor.form.name-label",
        "prompt-settings-editor.form.name",
        "prompt-settings-editor.form.text-label",
        "prompt-settings-editor.form.text",
        "prompt-settings-editor.form.text-hint",
        "prompt-settings-editor.form.provider-label",
        "prompt-settings-editor.form.provider",
        "prompt-settings-editor.form.model-label",
        "prompt-settings-editor.form.model",
        "prompt-settings-editor.form.endpoint",
        "prompt-settings-editor.form.shortcuts-label",
        "prompt-settings-editor.form.shortcuts",
        "prompt-settings-editor.form.add-shortcut",
        "prompt-settings-editor.form.name-error",
        "prompt-settings-editor.form.not-ready",
        "prompt-settings-editor.form.fix-field",
        "prompt-settings-editor.form.text-error",
        "prompt-settings-editor.form.provider-error",
        "prompt-settings-editor.form.shortcuts-hint",
        "prompt-settings-editor.form.shortcut-recorder",
        "prompt-settings-editor.form.shortcuts-conflicts-unavailable",
        "prompt-settings-editor.form.shortcuts-error"
      ],
      "id": "prompt-settings-editor.form",
      "role": "container",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form",
      "test_id": "prompt-settings-editor-form"
    },
    {
      "action": "prompt-settings-editor.form.add-shortcut",
      "id": "prompt-settings-editor.form.add-shortcut",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.add-shortcut",
      "test_id": "prompt-settings-editor-form-add-shortcut",
      "text": "Add shortcut"
    },
    {
      "id": "prompt-settings-editor.form.endpoint",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.endpoint",
      "test_id": "prompt-settings-editor-form-endpoint",
      "text": "Endpoint http://localhost:11434/v1 · Ready"
    },
    {
      "action": "prompt-settings-editor.form.fix",
      "id": "prompt-settings-editor.form.fix",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.fix",
      "test_id": "prompt-settings-editor-form-fix"
    },
    {
      "action": "prompt-settings-editor.form.fix-field",
      "binding": "provider_fix.api_key",
      "id": "prompt-settings-editor.form.fix-field",
      "role": "input",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.fix-field",
      "test_id": "prompt-settings-editor-form-fix-field"
    },
    {
      "action": "prompt-settings-editor.form.model",
      "binding": "prompt_draft.model",
      "id": "prompt-settings-editor.form.model",
      "role": "select",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.model",
      "test_id": "prompt-settings-editor-form-model",
      "text": "llama3.2"
    },
    {
      "id": "prompt-settings-editor.form.model-label",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.model-label",
      "test_id": "prompt-settings-editor-form-model-label",
      "text": "Model"
    },
    {
      "action": "prompt-settings-editor.form.name",
      "binding": "prompt_draft.name",
      "id": "prompt-settings-editor.form.name",
      "role": "input",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.name",
      "test_id": "prompt-settings-editor-form-name",
      "text": "Shorten"
    },
    {
      "id": "prompt-settings-editor.form.name-error",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.name-error",
      "test_id": "prompt-settings-editor-form-name-error",
      "text": "Enter a name (1–60 characters)."
    },
    {
      "id": "prompt-settings-editor.form.name-label",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.name-label",
      "test_id": "prompt-settings-editor-form-name-label",
      "text": "Name"
    },
    {
      "children": [
        "prompt-settings-editor.form.not-ready.title",
        "prompt-settings-editor.form.fix"
      ],
      "id": "prompt-settings-editor.form.not-ready",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.not-ready",
      "test_id": "prompt-settings-editor-form-not-ready"
    },
    {
      "id": "prompt-settings-editor.form.not-ready.title",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.not-ready.title",
      "test_id": "prompt-settings-editor-form-not-ready-title"
    },
    {
      "action": "prompt-settings-editor.form.provider",
      "binding": "prompt_draft.provider",
      "id": "prompt-settings-editor.form.provider",
      "role": "select",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.provider",
      "test_id": "prompt-settings-editor-form-provider",
      "text": "Custom endpoint"
    },
    {
      "id": "prompt-settings-editor.form.provider-error",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.provider-error",
      "test_id": "prompt-settings-editor-form-provider-error"
    },
    {
      "id": "prompt-settings-editor.form.provider-label",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.provider-label",
      "test_id": "prompt-settings-editor-form-provider-label",
      "text": "Provider"
    },
    {
      "children": [
        "prompt-settings-editor.form.shortcut.remove"
      ],
      "id": "prompt-settings-editor.form.shortcut",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcut",
      "test_id": "prompt-settings-editor-form-shortcut",
      "text": "Ctrl + Option + S"
    },
    {
      "id": "prompt-settings-editor.form.shortcut-conflict",
      "role": "icon",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcut-conflict",
      "test_id": "prompt-settings-editor-form-shortcut-conflict"
    },
    {
      "action": "prompt-settings-editor.form.shortcut-recorder",
      "id": "prompt-settings-editor.form.shortcut-recorder",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcut-recorder",
      "test_id": "prompt-settings-editor-form-shortcut-recorder"
    },
    {
      "id": "prompt-settings-editor.form.shortcut-refused",
      "role": "icon",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcut-refused",
      "test_id": "prompt-settings-editor-form-shortcut-refused"
    },
    {
      "action": "prompt-settings-editor.form.shortcut.remove",
      "id": "prompt-settings-editor.form.shortcut.remove",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcut.remove",
      "test_id": "prompt-settings-editor-form-shortcut-remove"
    },
    {
      "children": [
        "prompt-settings-editor.form.shortcut",
        "prompt-settings-editor.form.shortcut-refused",
        "prompt-settings-editor.form.shortcut-conflict"
      ],
      "id": "prompt-settings-editor.form.shortcuts",
      "repeat_model": {
        "collection": "prompt_draft.shortcuts",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcuts",
      "test_id": "prompt-settings-editor-form-shortcuts"
    },
    {
      "id": "prompt-settings-editor.form.shortcuts-conflicts-unavailable",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcuts-conflicts-unavailable",
      "test_id": "prompt-settings-editor-form-shortcuts-conflicts-unavailable"
    },
    {
      "id": "prompt-settings-editor.form.shortcuts-error",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcuts-error",
      "test_id": "prompt-settings-editor-form-shortcuts-error"
    },
    {
      "id": "prompt-settings-editor.form.shortcuts-hint",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcuts-hint",
      "test_id": "prompt-settings-editor-form-shortcuts-hint"
    },
    {
      "id": "prompt-settings-editor.form.shortcuts-label",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.shortcuts-label",
      "test_id": "prompt-settings-editor-form-shortcuts-label",
      "text": "Shortcuts (up to 2)"
    },
    {
      "action": "prompt-settings-editor.form.text",
      "binding": "prompt_draft.text",
      "id": "prompt-settings-editor.form.text",
      "role": "input",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.text",
      "test_id": "prompt-settings-editor-form-text",
      "text": "Shorten the transcript to the essentials. Keep names and numbers.\n\n${output}"
    },
    {
      "id": "prompt-settings-editor.form.text-error",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.text-error",
      "test_id": "prompt-settings-editor-form-text-error"
    },
    {
      "id": "prompt-settings-editor.form.text-hint",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.text-hint",
      "test_id": "prompt-settings-editor-form-text-hint",
      "text": "${output} is replaced by the transcript."
    },
    {
      "id": "prompt-settings-editor.form.text-label",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.form.text-label",
      "test_id": "prompt-settings-editor-form-text-label",
      "text": "Prompt"
    },
    {
      "id": "prompt-settings-editor.reload-error",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.reload-error",
      "test_id": "prompt-settings-editor-reload-error"
    },
    {
      "action": "prompt-settings-editor.reload-retry",
      "id": "prompt-settings-editor.reload-retry",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.reload-retry",
      "test_id": "prompt-settings-editor-reload-retry"
    },
    {
      "action": "prompt-settings-editor.save",
      "id": "prompt-settings-editor.save",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.save",
      "test_id": "prompt-settings-editor-save",
      "text": "Save"
    },
    {
      "id": "prompt-settings-editor.saved",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.saved",
      "test_id": "prompt-settings-editor-saved"
    },
    {
      "children": [
        "prompt-settings-editor.test.sample",
        "prompt-settings-editor.test.run",
        "prompt-settings-editor.test.note",
        "prompt-settings-editor.test.result",
        "prompt-settings-editor.test.error",
        "prompt-settings-editor.test.retry",
        "prompt-settings-editor.test.outdated"
      ],
      "id": "prompt-settings-editor.test",
      "role": "container",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test",
      "test_id": "prompt-settings-editor-test",
      "text": "Test"
    },
    {
      "children": [
        "prompt-settings-editor.test.error.title"
      ],
      "id": "prompt-settings-editor.test.error",
      "role": "alert",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.error",
      "test_id": "prompt-settings-editor-test-error"
    },
    {
      "id": "prompt-settings-editor.test.error.title",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.error.title",
      "test_id": "prompt-settings-editor-test-error-title",
      "text": "Test failed: the provider returned an error"
    },
    {
      "id": "prompt-settings-editor.test.meta",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.meta",
      "test_id": "prompt-settings-editor-test-meta",
      "text": "llama3.2 · Custom endpoint · 1.4 s"
    },
    {
      "id": "prompt-settings-editor.test.note",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.note",
      "test_id": "prompt-settings-editor-test-note",
      "text": "Nothing is inserted and history is not changed."
    },
    {
      "id": "prompt-settings-editor.test.outdated",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.outdated",
      "test_id": "prompt-settings-editor-test-outdated"
    },
    {
      "id": "prompt-settings-editor.test.output",
      "role": "text",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.output",
      "test_id": "prompt-settings-editor-test-output",
      "text": "Meeting moves to Thursday 3 pm; Julien sends the agenda first."
    },
    {
      "children": [
        "prompt-settings-editor.test.output",
        "prompt-settings-editor.test.meta"
      ],
      "id": "prompt-settings-editor.test.result",
      "role": "container",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.result",
      "test_id": "prompt-settings-editor-test-result"
    },
    {
      "action": "prompt-settings-editor.test.retry",
      "id": "prompt-settings-editor.test.retry",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.retry",
      "test_id": "prompt-settings-editor-test-retry"
    },
    {
      "action": "prompt-settings-editor.test.run",
      "id": "prompt-settings-editor.test.run",
      "role": "button",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.run",
      "test_id": "prompt-settings-editor-test-run",
      "text": "Run test"
    },
    {
      "action": "prompt-settings-editor.test.sample",
      "binding": "prompt_test.sample",
      "id": "prompt-settings-editor.test.sample",
      "role": "input",
      "screen_id": "prompt-settings-editor",
      "semantic_id": "prompt-settings-editor.test.sample",
      "test_id": "prompt-settings-editor-test-sample",
      "text": "So basically what I wanted to say is that the meeting moves to Thursday at three, and Julien will send the agenda before."
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "custom-ai-prompts",
    "id": "prompt-settings-editor",
    "root_node": "prompt-settings-editor",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "confirm-delete"
      },
      {
        "id": "recording"
      },
      {
        "id": "saved"
      },
      {
        "id": "shortcut-removed"
      },
      {
        "id": "tested"
      },
      {
        "id": "deleted"
      },
      {
        "id": "reloaded"
      },
      {
        "id": "fix-field-open"
      }
    ],
    "surface": "desktop",
    "title": "AI prompts — edit and test",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
