<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: shortcut-settings-limit

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
    }
  ],
  "nodes": [
    {
      "children": [
        "shortcut-settings-limit.general"
      ],
      "id": "shortcut-settings-limit",
      "role": "screen",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit",
      "test_id": "shortcut-settings-limit-screen"
    },
    {
      "children": [
        "shortcut-settings-limit.dictation.title",
        "shortcut-settings-limit.dictation.list",
        "shortcut-settings-limit.dictation.add",
        "shortcut-settings-limit.dictation.limit"
      ],
      "id": "shortcut-settings-limit.dictation",
      "role": "container",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation",
      "test_id": "shortcut-settings-limit-dictation"
    },
    {
      "action": "shortcut-settings-limit.dictation.add",
      "id": "shortcut-settings-limit.dictation.add",
      "role": "button",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.add",
      "test_id": "shortcut-settings-limit-dictation-add",
      "text": "Add shortcut"
    },
    {
      "id": "shortcut-settings-limit.dictation.limit",
      "role": "text",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.limit",
      "test_id": "shortcut-settings-limit-dictation-limit",
      "text": "4 of 4 shortcuts. Remove one to add another."
    },
    {
      "children": [
        "shortcut-settings-limit.dictation.shortcut"
      ],
      "id": "shortcut-settings-limit.dictation.list",
      "repeat_model": {
        "collection": "shortcuts.dictation",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.list",
      "test_id": "shortcut-settings-limit-dictation-list"
    },
    {
      "children": [
        "shortcut-settings-limit.dictation.shortcut.remove"
      ],
      "id": "shortcut-settings-limit.dictation.shortcut",
      "role": "text",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.shortcut",
      "test_id": "shortcut-settings-limit-dictation-shortcut",
      "text": "⌥ Space"
    },
    {
      "action": "shortcut-settings-limit.dictation.shortcut.remove",
      "id": "shortcut-settings-limit.dictation.shortcut.remove",
      "role": "button",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.shortcut.remove",
      "test_id": "shortcut-settings-limit-dictation-shortcut-remove"
    },
    {
      "id": "shortcut-settings-limit.dictation.title",
      "role": "text",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.dictation.title",
      "test_id": "shortcut-settings-limit-dictation-title",
      "text": "Dictation"
    },
    {
      "children": [
        "shortcut-settings-limit.dictation"
      ],
      "id": "shortcut-settings-limit.general",
      "role": "container",
      "screen_id": "shortcut-settings-limit",
      "semantic_id": "shortcut-settings-limit.general",
      "test_id": "shortcut-settings-limit-general",
      "text": "General"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "multiple-dictation-shortcuts",
    "id": "shortcut-settings-limit",
    "root_node": "shortcut-settings-limit",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "recording"
      },
      {
        "id": "shortcut-removed"
      }
    ],
    "surface": "desktop",
    "title": "Dictation shortcuts — limit",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
