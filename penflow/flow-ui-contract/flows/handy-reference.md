<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Flow: handy-reference

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
    {
      "destination": {
        "screen": "microphone-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "microphone-settings-empty.add",
      "kind": "navigation",
      "node": "microphone-settings-empty.add",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "clamshell"
      },
      "event": "change",
      "id": "microphone-settings-empty.clamshell.select",
      "kind": "state_change",
      "node": "microphone-settings-empty.clamshell.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-empty.refresh",
      "kind": "state_change",
      "node": "microphone-settings-empty.refresh",
      "success_transition": "microphone-settings-empty.refresh.refreshing",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "new_choice"
      },
      "event": "change",
      "id": "microphone-settings-error.add",
      "kind": "state_change",
      "node": "microphone-settings-error.add",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.move-down",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.move-down",
      "success_transition": "microphone-settings-error.choice.move-down.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.move-up",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.move-up",
      "success_transition": "microphone-settings-error.choice.move-up.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.choice.remove",
      "kind": "state_change",
      "node": "microphone-settings-error.choice.remove",
      "success_transition": "microphone-settings-error.choice.remove.removed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-dirty",
      "id": "microphone-settings-error.discard",
      "kind": "state_change",
      "node": "microphone-settings-error.discard",
      "success_transition": "microphone-settings-error.discard.discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.reload",
      "kind": "state_change",
      "node": "microphone-settings-error.reload",
      "success_transition": "microphone-settings-error.reload.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings-error.retry",
      "kind": "state_change",
      "node": "microphone-settings-error.retry",
      "success_transition": "microphone-settings-error.retry.refreshing",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "settings-conflict-cleared",
      "id": "microphone-settings-error.save",
      "kind": "state_change",
      "node": "microphone-settings-error.save",
      "success_transition": "microphone-settings-error.save.saved",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "new_choice"
      },
      "event": "change",
      "id": "microphone-settings.add",
      "kind": "state_change",
      "node": "microphone-settings.add",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.move-down",
      "kind": "state_change",
      "node": "microphone-settings.choice.move-down",
      "success_transition": "microphone-settings.choice.move-down.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.move-up",
      "kind": "state_change",
      "node": "microphone-settings.choice.move-up",
      "success_transition": "microphone-settings.choice.move-up.reordered",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.choice.remove",
      "kind": "state_change",
      "node": "microphone-settings.choice.remove",
      "success_transition": "microphone-settings.choice.remove.removed",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "microphone_priority",
        "field": "clamshell"
      },
      "event": "change",
      "id": "microphone-settings.clamshell.select",
      "kind": "state_change",
      "node": "microphone-settings.clamshell.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "draft-dirty",
      "id": "microphone-settings.discard",
      "kind": "state_change",
      "node": "microphone-settings.discard",
      "success_transition": "microphone-settings.discard.discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "microphone-settings.refresh",
      "kind": "state_change",
      "node": "microphone-settings.refresh",
      "success_transition": "microphone-settings.refresh.refreshing",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "draft-valid",
      "id": "microphone-settings.save",
      "kind": "state_change",
      "node": "microphone-settings.save",
      "success_transition": "microphone-settings.save.saved",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.always",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.always",
      "success_transition": "output-settings-enter-options.auto-enter.always.mode-chosen",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_key"
      },
      "event": "change",
      "id": "output-settings-enter-options.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "id": "output-settings-enter-options.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.mode",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.never",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.never",
      "success_transition": "output-settings-enter-options.auto-enter.never.mode-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-enter-options.auto-enter.pin-only",
      "kind": "state_change",
      "node": "output-settings-enter-options.auto-enter.pin-only",
      "success_transition": "output-settings-enter-options.auto-enter.pin-only.mode-chosen",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings-paste-methods.clipboard.select",
      "kind": "state_change",
      "node": "output-settings-paste-methods.clipboard.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.clipboard",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.clipboard",
      "success_transition": "output-settings-paste-methods.paste-method.clipboard.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.clipboard-ctrl-shift-v",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.clipboard-ctrl-shift-v",
      "success_transition": "output-settings-paste-methods.paste-method.clipboard-ctrl-shift-v.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.clipboard-shift-insert",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.clipboard-shift-insert",
      "success_transition": "output-settings-paste-methods.paste-method.clipboard-shift-insert.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.direct",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.direct",
      "success_transition": "output-settings-paste-methods.paste-method.direct.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.none",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.none",
      "success_transition": "output-settings-paste-methods.paste-method.none.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.script",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.script",
      "success_transition": "output-settings-paste-methods.paste-method.script.method-chosen",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings-paste-methods.paste-method.select",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.paste-method.smart",
      "kind": "state_change",
      "node": "output-settings-paste-methods.paste-method.smart",
      "success_transition": "output-settings-paste-methods.paste-method.smart.method-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-paste-methods.permission.open",
      "kind": "state_change",
      "node": "output-settings-paste-methods.permission.open",
      "success_transition": "output-settings-paste-methods.permission.open.permission-requested",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_key"
      },
      "event": "change",
      "id": "output-settings-unavailable.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings-unavailable.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "id": "output-settings-unavailable.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings-unavailable.auto-enter.mode",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings-unavailable.clipboard.select",
      "kind": "state_change",
      "node": "output-settings-unavailable.clipboard.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "no_field_bubble"
      },
      "event": "change",
      "id": "output-settings-unavailable.no-field-bubble.toggle",
      "kind": "state_change",
      "node": "output-settings-unavailable.no-field-bubble.toggle",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings-unavailable.paste-method.select",
      "kind": "state_change",
      "node": "output-settings-unavailable.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "output-settings-unavailable.return-app.info",
      "kind": "state_change",
      "node": "output-settings-unavailable.return-app.info",
      "success_transition": "output-settings-unavailable.return-app.info.details-shown",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "return_to_app"
      },
      "event": "change",
      "id": "output-settings-unavailable.return-app.toggle",
      "kind": "state_change",
      "node": "output-settings-unavailable.return-app.toggle",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_key"
      },
      "event": "change",
      "guard": "automatic-enter-available",
      "id": "output-settings.auto-enter.key",
      "kind": "state_change",
      "node": "output-settings.auto-enter.key",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "auto_enter_mode"
      },
      "event": "change",
      "guard": "automatic-enter-available",
      "id": "output-settings.auto-enter.mode",
      "kind": "state_change",
      "node": "output-settings.auto-enter.mode",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "clipboard_handling"
      },
      "event": "change",
      "id": "output-settings.clipboard.select",
      "kind": "state_change",
      "node": "output-settings.clipboard.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "history",
        "field": "limit"
      },
      "event": "change",
      "id": "output-settings.history-limit.input",
      "kind": "state_change",
      "node": "output-settings.history-limit.input",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "no_field_bubble"
      },
      "event": "change",
      "id": "output-settings.no-field-bubble.toggle",
      "kind": "state_change",
      "node": "output-settings.no-field-bubble.toggle",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "paste_method"
      },
      "event": "change",
      "id": "output-settings.paste-method.select",
      "kind": "state_change",
      "node": "output-settings.paste-method.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "destination-return-available",
      "id": "output-settings.return-app.info",
      "kind": "state_change",
      "node": "output-settings.return-app.info",
      "success_transition": "output-settings.return-app.info.details-shown",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "output",
        "field": "return_to_app"
      },
      "event": "change",
      "guard": "destination-return-available",
      "id": "output-settings.return-app.toggle",
      "kind": "state_change",
      "node": "output-settings.return-app.toggle",
      "trigger": "change"
    },
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
    },
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
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "post_processing",
        "field": "default_prompt"
      },
      "event": "change",
      "id": "prompt-settings.default.select",
      "kind": "state_change",
      "node": "prompt-settings.default.select",
      "trigger": "change"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "post_processing",
        "field": "enabled"
      },
      "event": "change",
      "id": "prompt-settings.enabled.toggle",
      "kind": "state_change",
      "node": "prompt-settings.enabled.toggle",
      "trigger": "change"
    },
    {
      "destination": {
        "screen": "prompt-settings-invalid",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings.new",
      "kind": "navigation",
      "node": "prompt-settings.new",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "prompt-settings-editor",
        "state": "default"
      },
      "event": "navigate",
      "id": "prompt-settings.prompt.edit",
      "kind": "navigation",
      "node": "prompt-settings.prompt.edit",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "prompt-settings.reload-retry",
      "kind": "state_change",
      "node": "prompt-settings.reload-retry",
      "success_transition": "prompt-settings.reload-retry.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.destination",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.destination",
      "success_transition": "recording-indicator-destination-menu.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.destination.remove",
      "success_transition": "recording-indicator-destination-menu.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.app",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.app",
      "success_transition": "recording-indicator-destination-menu.menu.app.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.frontmost",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.frontmost",
      "success_transition": "recording-indicator-destination-menu.menu.frontmost.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.frontmost-at-end",
      "success_transition": "recording-indicator-destination-menu.menu.frontmost-at-end.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.menu.pin",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.menu.pin",
      "success_transition": "recording-indicator-destination-menu.menu.pin.pinned",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-destination-menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-destination-menu.prompt",
      "success_transition": "recording-indicator-destination-menu.prompt.prompt-menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-no-destination.destination",
      "kind": "state_change",
      "node": "recording-indicator-no-destination.destination",
      "success_transition": "recording-indicator-no-destination.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-no-destination.prompt",
      "kind": "state_change",
      "node": "recording-indicator-no-destination.prompt",
      "success_transition": "recording-indicator-no-destination.prompt.prompt-menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.destination",
      "kind": "state_change",
      "node": "recording-indicator-pinned.destination",
      "success_transition": "recording-indicator-pinned.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-pinned.destination.remove",
      "success_transition": "recording-indicator-pinned.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.app",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.app",
      "success_transition": "recording-indicator-pinned.menu.app.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.frontmost",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.frontmost",
      "success_transition": "recording-indicator-pinned.menu.frontmost.destination-changed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "recording-indicator-pinned.menu.frontmost-at-end",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.frontmost-at-end",
      "success_transition": "recording-indicator-pinned.menu.frontmost-at-end.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.menu.unpin",
      "kind": "state_change",
      "node": "recording-indicator-pinned.menu.unpin",
      "success_transition": "recording-indicator-pinned.menu.unpin.unpinned",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-pinned.prompt",
      "kind": "state_change",
      "node": "recording-indicator-pinned.prompt",
      "success_transition": "recording-indicator-pinned.prompt.prompt-menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.destination",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.destination",
      "success_transition": "recording-indicator-prompt-menu.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.destination.remove",
      "success_transition": "recording-indicator-prompt-menu.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.menu.none",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.menu.none",
      "success_transition": "recording-indicator-prompt-menu.menu.none.prompt-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.menu.prompt",
      "success_transition": "recording-indicator-prompt-menu.menu.prompt.prompt-chosen",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator-prompt-menu.prompt",
      "kind": "state_change",
      "node": "recording-indicator-prompt-menu.prompt",
      "success_transition": "recording-indicator-prompt-menu.prompt.prompt-menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.destination",
      "kind": "state_change",
      "node": "recording-indicator.destination",
      "success_transition": "recording-indicator.destination.menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.destination.remove",
      "kind": "state_change",
      "node": "recording-indicator.destination.remove",
      "success_transition": "recording-indicator.destination.remove.destination-cleared",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "dictation-in-progress",
      "id": "recording-indicator.prompt",
      "kind": "state_change",
      "node": "recording-indicator.prompt",
      "success_transition": "recording-indicator.prompt.prompt-menu-open",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-explanation.close",
      "kind": "state_change",
      "node": "recovery-bubble-explanation.close",
      "success_transition": "recovery-bubble-explanation.close.closed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-explanation.copy",
      "kind": "state_change",
      "node": "recovery-bubble-explanation.copy",
      "success_transition": "recovery-bubble-explanation.copy.copied",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-explanation.drag-handle",
      "kind": "state_change",
      "node": "recovery-bubble-explanation.drag-handle",
      "success_transition": "recovery-bubble-explanation.drag-handle.dragging",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-explanation.why",
      "kind": "state_change",
      "node": "recovery-bubble-explanation.why",
      "success_transition": "recovery-bubble-explanation.why.explained",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-insertion-failed.close",
      "kind": "state_change",
      "node": "recovery-bubble-insertion-failed.close",
      "success_transition": "recovery-bubble-insertion-failed.close.closed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-insertion-failed.copy",
      "kind": "state_change",
      "node": "recovery-bubble-insertion-failed.copy",
      "success_transition": "recovery-bubble-insertion-failed.copy.copied",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "recovery-bubble-insertion-failed.drag-handle",
      "kind": "state_change",
      "node": "recovery-bubble-insertion-failed.drag-handle",
      "success_transition": "recovery-bubble-insertion-failed.drag-handle.dragging",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-insertion-failed.why",
      "kind": "state_change",
      "node": "recovery-bubble-insertion-failed.why",
      "success_transition": "recovery-bubble-insertion-failed.why.explained",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-no-field.close",
      "kind": "state_change",
      "node": "recovery-bubble-no-field.close",
      "success_transition": "recovery-bubble-no-field.close.closed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-no-field.copy",
      "kind": "state_change",
      "node": "recovery-bubble-no-field.copy",
      "success_transition": "recovery-bubble-no-field.copy.copied",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-no-field.drag-handle",
      "kind": "state_change",
      "node": "recovery-bubble-no-field.drag-handle",
      "success_transition": "recovery-bubble-no-field.drag-handle.dragging",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-no-field.why",
      "kind": "state_change",
      "node": "recovery-bubble-no-field.why",
      "success_transition": "recovery-bubble-no-field.why.explained",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble.close",
      "kind": "state_change",
      "node": "recovery-bubble.close",
      "success_transition": "recovery-bubble.close.closed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble.copy",
      "kind": "state_change",
      "node": "recovery-bubble.copy",
      "success_transition": "recovery-bubble.copy.copied",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble.drag-handle",
      "kind": "state_change",
      "node": "recovery-bubble.drag-handle",
      "success_transition": "recovery-bubble.drag-handle.dragging",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble.why",
      "kind": "state_change",
      "node": "recovery-bubble.why",
      "success_transition": "recovery-bubble.why.explained",
      "trigger": "click"
    },
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
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.empty.add",
      "kind": "navigation",
      "node": "replacement-settings-empty.empty.add",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.empty.paste",
      "kind": "navigation",
      "node": "replacement-settings-empty.empty.paste",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-empty.export",
      "kind": "state_change",
      "node": "replacement-settings-empty.export",
      "success_transition": "replacement-settings-empty.export.exporting",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-import",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.import",
      "kind": "navigation",
      "node": "replacement-settings-empty.import",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.view-list",
      "kind": "navigation",
      "node": "replacement-settings-empty.view-list",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings-bulk",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-empty.view-text",
      "kind": "navigation",
      "node": "replacement-settings-empty.view-text",
      "trigger": "click"
    },
    {
      "destination": {
        "screen": "replacement-settings",
        "state": "default"
      },
      "event": "navigate",
      "id": "replacement-settings-import.dialog.apply",
      "kind": "navigation",
      "node": "replacement-settings-import.dialog.apply",
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
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "replacement-settings-import.unreadable.dismiss",
      "kind": "state_change",
      "node": "replacement-settings-import.unreadable.dismiss",
      "success_transition": "replacement-settings-import.unreadable.dismiss.refusal-dismissed",
      "trigger": "click"
    },
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
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "history",
        "field": "limit"
      },
      "event": "change",
      "id": "settings-write-errors.history-limit.input",
      "kind": "state_change",
      "node": "settings-write-errors.history-limit.input",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "settings-write-errors.reload",
      "kind": "state_change",
      "node": "settings-write-errors.reload",
      "success_transition": "settings-write-errors.reload.reloaded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-conflict.dictation.change",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.change",
      "success_transition": "shortcut-settings-conflict.dictation.change.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-conflict.dictation.os.retry",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.os.retry",
      "success_transition": "shortcut-settings-conflict.dictation.os.retry.retried",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-conflict.dictation.recording",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.recording",
      "success_transition": "shortcut-settings-conflict.dictation.recording.activated",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-conflict.dictation.recording-discard",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.recording-discard",
      "success_transition": "shortcut-settings-conflict.dictation.recording-discard.recording-discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-conflict.dictation.shortcut.remove",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.shortcut.remove",
      "success_transition": "shortcut-settings-conflict.dictation.shortcut.remove.shortcut-removed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-limit.dictation.add",
      "kind": "state_change",
      "node": "shortcut-settings-limit.dictation.add",
      "success_transition": "shortcut-settings-limit.dictation.add.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings-limit.dictation.shortcut.remove",
      "kind": "state_change",
      "node": "shortcut-settings-limit.dictation.shortcut.remove",
      "success_transition": "shortcut-settings-limit.dictation.shortcut.remove.shortcut-removed",
      "trigger": "click"
    },
    {
      "binding_ref": {
        "access": "write",
        "entity": "shortcuts",
        "field": "activation_mode"
      },
      "event": "change",
      "id": "shortcut-settings.activation.select",
      "kind": "state_change",
      "node": "shortcut-settings.activation.select",
      "trigger": "change"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.add",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.add",
      "success_transition": "shortcut-settings.ai-dictation.add.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.reset",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.reset",
      "success_transition": "shortcut-settings.ai-dictation.reset.reset",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.ai-dictation.shortcut.remove",
      "kind": "state_change",
      "node": "shortcut-settings.ai-dictation.shortcut.remove",
      "success_transition": "shortcut-settings.ai-dictation.shortcut.remove.shortcut-removed",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.cancel.recording",
      "kind": "state_change",
      "node": "shortcut-settings.cancel.recording",
      "success_transition": "shortcut-settings.cancel.recording.activated",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.cancel.reset",
      "kind": "state_change",
      "node": "shortcut-settings.cancel.reset",
      "success_transition": "shortcut-settings.cancel.reset.reset",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.cancel.shortcut",
      "kind": "state_change",
      "node": "shortcut-settings.cancel.shortcut",
      "success_transition": "shortcut-settings.cancel.shortcut.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "recorder-has-keys",
      "id": "shortcut-settings.dictation.add",
      "kind": "state_change",
      "node": "shortcut-settings.dictation.add",
      "success_transition": "shortcut-settings.dictation.add.recording",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.dictation.recording",
      "kind": "state_change",
      "node": "shortcut-settings.dictation.recording",
      "success_transition": "shortcut-settings.dictation.recording.activated",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.dictation.recording-discard",
      "kind": "state_change",
      "node": "shortcut-settings.dictation.recording-discard",
      "success_transition": "shortcut-settings.dictation.recording-discard.recording-discarded",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.dictation.reset",
      "kind": "state_change",
      "node": "shortcut-settings.dictation.reset",
      "success_transition": "shortcut-settings.dictation.reset.reset",
      "trigger": "click"
    },
    {
      "event": "submit",
      "guard": "control-enabled",
      "id": "shortcut-settings.dictation.shortcut.remove",
      "kind": "state_change",
      "node": "shortcut-settings.dictation.shortcut.remove",
      "success_transition": "shortcut-settings.dictation.shortcut.remove.shortcut-removed",
      "trigger": "click"
    },
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
    },
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
    },
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
    },
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
  "flow": {
    "entry_screen": "handy-reference",
    "id": "handy-reference"
  },
  "relations": [],
  "screens": [
    {
      "flow_id": "handy-reference",
      "id": "handy-reference",
      "root_node": "handy-reference",
      "states": [
        {
          "id": "default"
        }
      ],
      "static": true,
      "static_reason": "Design reference only: the current Handy General settings window recomposed from the source code, and the reusable pen.dev components every feature screen is built from. It is not a feature screen and adds no behavior.",
      "surface": "desktop",
      "title": "Handy reference — existing General settings and component library",
      "viewport": {
        "height": 570,
        "width": 680
      }
    }
  ],
  "transitions": []
}
```
