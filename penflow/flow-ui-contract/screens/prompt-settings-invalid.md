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
      "destination": {
        "screen": "prompt-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings-invalid.form.fix",
      "kind": "navigation",
      "node": "prompt-settings-invalid.form.fix",
      "trigger": "click"
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
      "guard": "draft-valid",
      "id": "prompt-settings-invalid.save",
      "kind": "state_change",
      "node": "prompt-settings-invalid.save",
      "success_transition": "prompt-settings-invalid.save.saved",
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
        "prompt-settings-invalid.actions"
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
        "prompt-settings-invalid.save"
      ],
      "id": "prompt-settings-invalid.actions",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.actions",
      "test_id": "prompt-settings-invalid-actions"
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
        "prompt-settings-invalid.form.not-ready"
      ],
      "id": "prompt-settings-invalid.form",
      "role": "container",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form",
      "test_id": "prompt-settings-invalid-form"
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
      "id": "prompt-settings-invalid.form.provider-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.provider-label",
      "test_id": "prompt-settings-invalid-form-provider-label",
      "text": "Provider"
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
      "id": "prompt-settings-invalid.form.text-label",
      "role": "text",
      "screen_id": "prompt-settings-invalid",
      "semantic_id": "prompt-settings-invalid.form.text-label",
      "test_id": "prompt-settings-invalid-form-text-label",
      "text": "Prompt"
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
      "children": [
        "prompt-settings-invalid.test.sample",
        "prompt-settings-invalid.test.run",
        "prompt-settings-invalid.test.error"
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
      "text": "Test failed: provider returned HTTP 401"
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
        "id": "saved"
      },
      {
        "id": "tested"
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
