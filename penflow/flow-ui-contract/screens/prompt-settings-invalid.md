<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: prompt-settings-invalid

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
      "id": "prompt-settings-invalid.cancel",
      "kind": "navigation",
      "node": "prompt-settings-invalid.cancel",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.delete",
      "kind": "state_change",
      "node": "prompt-settings-invalid.delete",
      "success_transition": "prompt-settings-invalid.delete.confirm-delete",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.delete-confirm",
      "kind": "state_change",
      "node": "prompt-settings-invalid.delete-confirm",
      "success_transition": "prompt-settings-invalid.delete-confirm.deleted",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.form.fix",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.fix",
      "success_transition": "prompt-settings-invalid.form.fix.fix-field-open",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "provider_fix",
        "field": "api_key"
      },
      "event": "change",
      "id": "prompt-settings-invalid.form.fix-field",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.fix-field",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "model"
      },
      "event": "change",
      "id": "prompt-settings-invalid.form.model",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.model",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "name"
      },
      "event": "change",
      "id": "prompt-settings-invalid.form.name",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.name",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "provider"
      },
      "event": "change",
      "id": "prompt-settings-invalid.form.provider",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.provider",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.form.shortcut-recorder",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.shortcut-recorder",
      "success_transition": "prompt-settings-invalid.form.shortcut-recorder.recording",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_draft",
        "field": "text"
      },
      "event": "change",
      "id": "prompt-settings-invalid.form.text",
      "kind": "state_change",
      "node": "prompt-settings-invalid.form.text",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.reload-retry",
      "kind": "state_change",
      "node": "prompt-settings-invalid.reload-retry",
      "success_transition": "prompt-settings-invalid.reload-retry.reloaded",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "prompt-settings-editor",
        "state": "saved"
      },
      "event": "navigate",
      "id": "prompt-settings-invalid.save",
      "kind": "navigation",
      "node": "prompt-settings-invalid.save",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.test.retry",
      "kind": "state_change",
      "node": "prompt-settings-invalid.test.retry",
      "success_transition": "prompt-settings-invalid.test.retry.tested",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings-invalid.test.run",
      "kind": "state_change",
      "node": "prompt-settings-invalid.test.run",
      "success_transition": "prompt-settings-invalid.test.run.tested",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "prompt_test",
        "field": "sample"
      },
      "event": "change",
      "id": "prompt-settings-invalid.test.sample",
      "kind": "state_change",
      "node": "prompt-settings-invalid.test.sample",
      "trigger": "change"
    }
  ],
  "nodes": [
    {
      "children": [
        "prompt-settings-invalid.editor",
        "prompt-settings-invalid.test",
        "prompt-settings-invalid.actions",
        "prompt-settings-invalid.deleted-elsewhere",
        "prompt-settings-invalid.changed-elsewhere",
        "prompt-settings-invalid.delete-dialog",
        "prompt-settings-invalid.reload-error",
        "prompt-settings-invalid.reload-retry"
      ],
      "id": "prompt-settings-invalid",
      "role": "screen",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid",
      "test_id": "prompt-settings-invalid-screen"
    },
    {
      "children": [
        "prompt-settings-invalid.actions.note",
        "prompt-settings-invalid.cancel",
        "prompt-settings-invalid.save",
        "prompt-settings-invalid.delete",
        "prompt-settings-invalid.saved",
        "prompt-settings-invalid.actions-error"
      ],
      "id": "prompt-settings-invalid.actions",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.actions",
      "test_id": "prompt-settings-invalid-actions"
    },
    {
      "id": "prompt-settings-invalid.actions-error",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.actions-error",
      "test_id": "prompt-settings-invalid-actions-error"
    },
    {
      "id": "prompt-settings-invalid.actions.note",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.actions.note",
      "test_id": "prompt-settings-invalid-actions-note",
      "text": "Fix the highlighted fields to save."
    },
    {
      "action": "prompt-settings-invalid.cancel",
      "id": "prompt-settings-invalid.cancel",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.cancel",
      "test_id": "prompt-settings-invalid-cancel",
      "text": "Cancel"
    },
    {
      "id": "prompt-settings-invalid.changed-elsewhere",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.changed-elsewhere",
      "test_id": "prompt-settings-invalid-changed-elsewhere"
    },
    {
      "action": "prompt-settings-invalid.delete",
      "id": "prompt-settings-invalid.delete",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.delete",
      "test_id": "prompt-settings-invalid-delete"
    },
    {
      "action": "prompt-settings-invalid.delete-confirm",
      "id": "prompt-settings-invalid.delete-confirm",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.delete-confirm",
      "test_id": "prompt-settings-invalid-delete-confirm"
    },
    {
      "children": [
        "prompt-settings-invalid.delete-confirm"
      ],
      "id": "prompt-settings-invalid.delete-dialog",
      "role": "modal",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.delete-dialog",
      "test_id": "prompt-settings-invalid-delete-dialog"
    },
    {
      "id": "prompt-settings-invalid.deleted-elsewhere",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.deleted-elsewhere",
      "test_id": "prompt-settings-invalid-deleted-elsewhere"
    },
    {
      "children": [
        "prompt-settings-invalid.form"
      ],
      "id": "prompt-settings-invalid.editor",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.editor",
      "test_id": "prompt-settings-invalid-editor",
      "text": "New prompt"
    },
    {
      "children": [
        "prompt-settings-invalid.form.name-label",
        "prompt-settings-invalid.form.name",
        "prompt-settings-invalid.form.name-error",
        "prompt-settings-invalid.form.text-label",
        "prompt-settings-invalid.form.text",
        "prompt-settings-invalid.form.provider-label",
        "prompt-settings-invalid.form.provider",
        "prompt-settings-invalid.form.model-label",
        "prompt-settings-invalid.form.model",
        "prompt-settings-invalid.form.not-ready",
        "prompt-settings-invalid.form.text-hint",
        "prompt-settings-invalid.form.endpoint",
        "prompt-settings-invalid.form.fix-field",
        "prompt-settings-invalid.form.text-error",
        "prompt-settings-invalid.form.provider-error",
        "prompt-settings-invalid.form.shortcuts-hint",
        "prompt-settings-invalid.form.shortcut-recorder",
        "prompt-settings-invalid.form.shortcut-refused",
        "prompt-settings-invalid.form.shortcut-conflict",
        "prompt-settings-invalid.form.shortcuts-conflicts-unavailable",
        "prompt-settings-invalid.form.shortcuts-error"
      ],
      "id": "prompt-settings-invalid.form",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form",
      "test_id": "prompt-settings-invalid-form"
    },
    {
      "id": "prompt-settings-invalid.form.endpoint",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.endpoint",
      "test_id": "prompt-settings-invalid-form-endpoint",
      "text": "Endpoint https://api.openai.com/v1"
    },
    {
      "action": "prompt-settings-invalid.form.fix",
      "id": "prompt-settings-invalid.form.fix",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.fix",
      "test_id": "prompt-settings-invalid-form-fix",
      "text": "Add API key"
    },
    {
      "action": "prompt-settings-invalid.form.fix-field",
      "binding": "provider_fix.api_key",
      "id": "prompt-settings-invalid.form.fix-field",
      "role": "input",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.fix-field",
      "test_id": "prompt-settings-invalid-form-fix-field"
    },
    {
      "action": "prompt-settings-invalid.form.model",
      "binding": "prompt_draft.model",
      "id": "prompt-settings-invalid.form.model",
      "role": "select",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.model",
      "test_id": "prompt-settings-invalid-form-model",
      "text": "gpt-4.1-mini"
    },
    {
      "id": "prompt-settings-invalid.form.model-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.model-label",
      "test_id": "prompt-settings-invalid-form-model-label",
      "text": "Model"
    },
    {
      "action": "prompt-settings-invalid.form.name",
      "binding": "prompt_draft.name",
      "id": "prompt-settings-invalid.form.name",
      "role": "input",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.name",
      "test_id": "prompt-settings-invalid-form-name",
      "text": "Prompt name"
    },
    {
      "id": "prompt-settings-invalid.form.name-error",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.name-error",
      "test_id": "prompt-settings-invalid-form-name-error",
      "text": "Enter a name (1–60 characters)."
    },
    {
      "id": "prompt-settings-invalid.form.name-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.name-label",
      "test_id": "prompt-settings-invalid-form-name-label",
      "text": "Name"
    },
    {
      "children": [
        "prompt-settings-invalid.form.not-ready.title",
        "prompt-settings-invalid.form.fix"
      ],
      "id": "prompt-settings-invalid.form.not-ready",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.not-ready",
      "test_id": "prompt-settings-invalid-form-not-ready"
    },
    {
      "id": "prompt-settings-invalid.form.not-ready.title",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.not-ready.title",
      "test_id": "prompt-settings-invalid-form-not-ready-title",
      "text": "OpenAI is not ready: API key missing"
    },
    {
      "action": "prompt-settings-invalid.form.provider",
      "binding": "prompt_draft.provider",
      "id": "prompt-settings-invalid.form.provider",
      "role": "select",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.provider",
      "test_id": "prompt-settings-invalid-form-provider",
      "text": "OpenAI"
    },
    {
      "id": "prompt-settings-invalid.form.provider-error",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.provider-error",
      "test_id": "prompt-settings-invalid-form-provider-error"
    },
    {
      "id": "prompt-settings-invalid.form.provider-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.provider-label",
      "test_id": "prompt-settings-invalid-form-provider-label",
      "text": "Provider"
    },
    {
      "id": "prompt-settings-invalid.form.shortcut-conflict",
      "role": "icon",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcut-conflict",
      "test_id": "prompt-settings-invalid-form-shortcut-conflict"
    },
    {
      "action": "prompt-settings-invalid.form.shortcut-recorder",
      "id": "prompt-settings-invalid.form.shortcut-recorder",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcut-recorder",
      "test_id": "prompt-settings-invalid-form-shortcut-recorder"
    },
    {
      "id": "prompt-settings-invalid.form.shortcut-refused",
      "role": "icon",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcut-refused",
      "test_id": "prompt-settings-invalid-form-shortcut-refused"
    },
    {
      "id": "prompt-settings-invalid.form.shortcuts-conflicts-unavailable",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcuts-conflicts-unavailable",
      "test_id": "prompt-settings-invalid-form-shortcuts-conflicts-unavailable"
    },
    {
      "id": "prompt-settings-invalid.form.shortcuts-error",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcuts-error",
      "test_id": "prompt-settings-invalid-form-shortcuts-error"
    },
    {
      "id": "prompt-settings-invalid.form.shortcuts-hint",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.shortcuts-hint",
      "test_id": "prompt-settings-invalid-form-shortcuts-hint"
    },
    {
      "action": "prompt-settings-invalid.form.text",
      "binding": "prompt_draft.text",
      "id": "prompt-settings-invalid.form.text",
      "role": "input",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.text",
      "test_id": "prompt-settings-invalid-form-text",
      "text": "Translate the transcript into English.\n\n${output}"
    },
    {
      "id": "prompt-settings-invalid.form.text-error",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.text-error",
      "test_id": "prompt-settings-invalid-form-text-error"
    },
    {
      "id": "prompt-settings-invalid.form.text-hint",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.text-hint",
      "test_id": "prompt-settings-invalid-form-text-hint",
      "text": "${output} is replaced by the transcript."
    },
    {
      "id": "prompt-settings-invalid.form.text-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.text-label",
      "test_id": "prompt-settings-invalid-form-text-label",
      "text": "Prompt"
    },
    {
      "id": "prompt-settings-invalid.reload-error",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.reload-error",
      "test_id": "prompt-settings-invalid-reload-error"
    },
    {
      "action": "prompt-settings-invalid.reload-retry",
      "id": "prompt-settings-invalid.reload-retry",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.reload-retry",
      "test_id": "prompt-settings-invalid-reload-retry"
    },
    {
      "action": "prompt-settings-invalid.save",
      "id": "prompt-settings-invalid.save",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.save",
      "test_id": "prompt-settings-invalid-save",
      "text": "Save"
    },
    {
      "id": "prompt-settings-invalid.saved",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.saved",
      "test_id": "prompt-settings-invalid-saved"
    },
    {
      "children": [
        "prompt-settings-invalid.test.sample",
        "prompt-settings-invalid.test.run",
        "prompt-settings-invalid.test.error",
        "prompt-settings-invalid.test.result",
        "prompt-settings-invalid.test.retry",
        "prompt-settings-invalid.test.outdated"
      ],
      "id": "prompt-settings-invalid.test",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test",
      "test_id": "prompt-settings-invalid-test",
      "text": "Test"
    },
    {
      "children": [
        "prompt-settings-invalid.test.error.title"
      ],
      "id": "prompt-settings-invalid.test.error",
      "role": "alert",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.error",
      "test_id": "prompt-settings-invalid-test-error"
    },
    {
      "id": "prompt-settings-invalid.test.error.title",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.error.title",
      "test_id": "prompt-settings-invalid-test-error-title",
      "text": "Test failed: the provider returned an error"
    },
    {
      "id": "prompt-settings-invalid.test.meta",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.meta",
      "test_id": "prompt-settings-invalid-test-meta"
    },
    {
      "id": "prompt-settings-invalid.test.outdated",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.outdated",
      "test_id": "prompt-settings-invalid-test-outdated"
    },
    {
      "id": "prompt-settings-invalid.test.output",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.output",
      "test_id": "prompt-settings-invalid-test-output"
    },
    {
      "children": [
        "prompt-settings-invalid.test.output",
        "prompt-settings-invalid.test.meta"
      ],
      "id": "prompt-settings-invalid.test.result",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.result",
      "test_id": "prompt-settings-invalid-test-result"
    },
    {
      "action": "prompt-settings-invalid.test.retry",
      "id": "prompt-settings-invalid.test.retry",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.retry",
      "test_id": "prompt-settings-invalid-test-retry"
    },
    {
      "action": "prompt-settings-invalid.test.run",
      "id": "prompt-settings-invalid.test.run",
      "role": "button",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.run",
      "test_id": "prompt-settings-invalid-test-run",
      "text": "Run test"
    },
    {
      "action": "prompt-settings-invalid.test.sample",
      "binding": "prompt_test.sample",
      "id": "prompt-settings-invalid.test.sample",
      "role": "input",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.test.sample",
      "test_id": "prompt-settings-invalid-test-sample",
      "text": "Le point de demain est décalé à jeudi."
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "custom-ai-prompts",
    "id": "prompt-settings-invalid",
    "root_node": "prompt-settings-invalid",
    "states": [
      {
        "id": "default"
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
        "id": "recording"
      },
      {
        "id": "confirm-delete"
      },
      {
        "id": "fix-field-open"
      }
    ],
    "surface": "desktop",
    "title": "AI prompts — invalid and not ready",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
