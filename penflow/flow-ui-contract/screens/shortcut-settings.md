<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: shortcut-settings

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
    }
  ],
  "nodes": [
    {
      "children": [
        "shortcut-settings.general"
      ],
      "id": "shortcut-settings",
      "role": "screen",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings",
      "test_id": "shortcut-settings-screen"
    },
    {
      "children": [
        "shortcut-settings.activation.title",
        "shortcut-settings.activation.select"
      ],
      "id": "shortcut-settings.activation",
      "role": "container",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.activation",
      "test_id": "shortcut-settings-activation"
    },
    {
      "action": "shortcut-settings.activation.select",
      "binding": "shortcuts.activation_mode",
      "id": "shortcut-settings.activation.select",
      "role": "select",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.activation.select",
      "test_id": "shortcut-settings-activation-select",
      "text": "Toggle"
    },
    {
      "id": "shortcut-settings.activation.title",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.activation.title",
      "test_id": "shortcut-settings-activation-title",
      "text": "Shortcut Behavior"
    },
    {
      "children": [
        "shortcut-settings.cancel.title",
        "shortcut-settings.cancel.list",
        "shortcut-settings.cancel.recording",
        "shortcut-settings.cancel.reset"
      ],
      "id": "shortcut-settings.cancel",
      "role": "container",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel",
      "test_id": "shortcut-settings-cancel"
    },
    {
      "children": [
        "shortcut-settings.cancel.shortcut"
      ],
      "id": "shortcut-settings.cancel.list",
      "repeat_model": {
        "collection": "shortcuts.cancel",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel.list",
      "test_id": "shortcut-settings-cancel-list"
    },
    {
      "action": "shortcut-settings.cancel.recording",
      "id": "shortcut-settings.cancel.recording",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel.recording",
      "states": [
        "recording"
      ],
      "test_id": "shortcut-settings-cancel-recording",
      "text": "Press keys…"
    },
    {
      "action": "shortcut-settings.cancel.reset",
      "id": "shortcut-settings.cancel.reset",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel.reset",
      "test_id": "shortcut-settings-cancel-reset"
    },
    {
      "action": "shortcut-settings.cancel.shortcut",
      "id": "shortcut-settings.cancel.shortcut",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel.shortcut",
      "test_id": "shortcut-settings-cancel-shortcut",
      "text": "Esc"
    },
    {
      "id": "shortcut-settings.cancel.title",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.cancel.title",
      "test_id": "shortcut-settings-cancel-title",
      "text": "Cancel Shortcut"
    },
    {
      "children": [
        "shortcut-settings.dictation.title",
        "shortcut-settings.dictation.list",
        "shortcut-settings.dictation.recording",
        "shortcut-settings.dictation.recording-discard",
        "shortcut-settings.dictation.add",
        "shortcut-settings.dictation.reset",
        "shortcut-settings.dictation.last",
        "shortcut-settings.dictation.conflicts-unavailable"
      ],
      "id": "shortcut-settings.dictation",
      "role": "container",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation",
      "test_id": "shortcut-settings-dictation"
    },
    {
      "action": "shortcut-settings.dictation.add",
      "id": "shortcut-settings.dictation.add",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.add",
      "test_id": "shortcut-settings-dictation-add",
      "text": "Add shortcut"
    },
    {
      "id": "shortcut-settings.dictation.conflicts-unavailable",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.conflicts-unavailable",
      "states": [
        "conflicts-unavailable"
      ],
      "test_id": "shortcut-settings-dictation-conflicts-unavailable",
      "text": "Shortcut conflicts could not be checked"
    },
    {
      "id": "shortcut-settings.dictation.last",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.last",
      "states": [
        "last-shortcut"
      ],
      "test_id": "shortcut-settings-dictation-last",
      "text": "Each action keeps at least one shortcut. Change or reset it instead."
    },
    {
      "children": [
        "shortcut-settings.dictation.shortcut",
        "shortcut-settings.dictation.shortcut-refused",
        "shortcut-settings.dictation.shortcut-conflict"
      ],
      "id": "shortcut-settings.dictation.list",
      "repeat_model": {
        "collection": "shortcuts.dictation",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.list",
      "test_id": "shortcut-settings-dictation-list"
    },
    {
      "action": "shortcut-settings.dictation.recording",
      "id": "shortcut-settings.dictation.recording",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.recording",
      "test_id": "shortcut-settings-dictation-recording",
      "text": "Press keys…"
    },
    {
      "action": "shortcut-settings.dictation.recording-discard",
      "id": "shortcut-settings.dictation.recording-discard",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.recording-discard",
      "states": [
        "recording"
      ],
      "test_id": "shortcut-settings-dictation-recording-discard"
    },
    {
      "action": "shortcut-settings.dictation.reset",
      "id": "shortcut-settings.dictation.reset",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.reset",
      "test_id": "shortcut-settings-dictation-reset",
      "text": "Reset"
    },
    {
      "children": [
        "shortcut-settings.dictation.shortcut.remove"
      ],
      "id": "shortcut-settings.dictation.shortcut",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.shortcut",
      "test_id": "shortcut-settings-dictation-shortcut",
      "text": "⌥ Space"
    },
    {
      "id": "shortcut-settings.dictation.shortcut-conflict",
      "role": "icon",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.shortcut-conflict",
      "states": [
        "conflict"
      ],
      "test_id": "shortcut-settings-dictation-shortcut-conflict"
    },
    {
      "id": "shortcut-settings.dictation.shortcut-refused",
      "role": "icon",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.shortcut-refused",
      "states": [
        "refused"
      ],
      "test_id": "shortcut-settings-dictation-shortcut-refused"
    },
    {
      "action": "shortcut-settings.dictation.shortcut.remove",
      "id": "shortcut-settings.dictation.shortcut.remove",
      "role": "button",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.shortcut.remove",
      "test_id": "shortcut-settings-dictation-shortcut-remove"
    },
    {
      "id": "shortcut-settings.dictation.title",
      "role": "text",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.dictation.title",
      "test_id": "shortcut-settings-dictation-title",
      "text": "Transcribe Shortcut"
    },
    {
      "children": [
        "shortcut-settings.dictation",
        "shortcut-settings.activation",
        "shortcut-settings.cancel"
      ],
      "id": "shortcut-settings.general",
      "role": "container",
      "screen_id": "shortcut-settings",
      "semantic_id": "shortcut-settings.general",
      "test_id": "shortcut-settings-general",
      "text": "General"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "multiple-dictation-shortcuts",
    "id": "shortcut-settings",
    "root_node": "shortcut-settings",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "activated"
      },
      {
        "id": "recording"
      },
      {
        "id": "reset"
      },
      {
        "id": "shortcut-removed"
      },
      {
        "id": "recording-discarded"
      }
    ],
    "surface": "desktop",
    "title": "Dictation shortcuts",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
