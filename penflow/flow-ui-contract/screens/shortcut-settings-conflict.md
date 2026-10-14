<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: shortcut-settings-conflict

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
      "id": "shortcut-settings-conflict.dictation.recording",
      "kind": "state_change",
      "node": "shortcut-settings-conflict.dictation.recording",
      "success_transition": "shortcut-settings-conflict.dictation.recording.activated",
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
    }
  ],
  "nodes": [
    {
      "children": [
        "shortcut-settings-conflict.general"
      ],
      "id": "shortcut-settings-conflict",
      "role": "screen",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict",
      "test_id": "shortcut-settings-conflict-screen"
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation.title",
        "shortcut-settings-conflict.dictation.list",
        "shortcut-settings-conflict.dictation.recording",
        "shortcut-settings-conflict.dictation.conflict",
        "shortcut-settings-conflict.dictation.legacy",
        "shortcut-settings-conflict.dictation.os"
      ],
      "id": "shortcut-settings-conflict.dictation",
      "role": "container",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation",
      "test_id": "shortcut-settings-conflict-dictation"
    },
    {
      "action": "shortcut-settings-conflict.dictation.change",
      "id": "shortcut-settings-conflict.dictation.change",
      "role": "button",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.change",
      "test_id": "shortcut-settings-conflict-dictation-change",
      "text": "Change"
    },
    {
      "id": "shortcut-settings-conflict.dictation.conflict",
      "role": "text",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.conflict",
      "test_id": "shortcut-settings-conflict-dictation-conflict",
      "text": "⇧ ⌥ Space is already used by Dictation with AI processing. Nothing was saved."
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation.legacy.title",
        "shortcut-settings-conflict.dictation.change"
      ],
      "id": "shortcut-settings-conflict.dictation.legacy",
      "role": "alert",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.legacy",
      "test_id": "shortcut-settings-conflict-dictation-legacy"
    },
    {
      "id": "shortcut-settings-conflict.dictation.legacy.title",
      "role": "text",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.legacy.title",
      "test_id": "shortcut-settings-conflict-dictation-legacy-title",
      "text": "⌃ ⌥ D overlaps with Cancel"
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation.shortcut"
      ],
      "id": "shortcut-settings-conflict.dictation.list",
      "repeat_model": {
        "collection": "shortcuts.dictation",
        "instance_id_template": "shortcut-{keys}",
        "item_key": "keys",
        "item_model": "shortcut_binding"
      },
      "role": "list",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.list",
      "test_id": "shortcut-settings-conflict-dictation-list"
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation.os.title"
      ],
      "id": "shortcut-settings-conflict.dictation.os",
      "role": "alert",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.os",
      "test_id": "shortcut-settings-conflict-dictation-os"
    },
    {
      "id": "shortcut-settings-conflict.dictation.os.title",
      "role": "text",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.os.title",
      "test_id": "shortcut-settings-conflict-dictation-os-title",
      "text": "macOS refused ⌘ ⇧ 5"
    },
    {
      "action": "shortcut-settings-conflict.dictation.recording",
      "id": "shortcut-settings-conflict.dictation.recording",
      "role": "button",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.recording",
      "test_id": "shortcut-settings-conflict-dictation-recording",
      "text": "⇧ ⌥ Space"
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation.shortcut.remove"
      ],
      "id": "shortcut-settings-conflict.dictation.shortcut",
      "role": "text",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.shortcut",
      "test_id": "shortcut-settings-conflict-dictation-shortcut",
      "text": "⌥ Space"
    },
    {
      "action": "shortcut-settings-conflict.dictation.shortcut.remove",
      "id": "shortcut-settings-conflict.dictation.shortcut.remove",
      "role": "button",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.shortcut.remove",
      "test_id": "shortcut-settings-conflict-dictation-shortcut-remove"
    },
    {
      "id": "shortcut-settings-conflict.dictation.title",
      "role": "text",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.dictation.title",
      "test_id": "shortcut-settings-conflict-dictation-title",
      "text": "Dictation"
    },
    {
      "children": [
        "shortcut-settings-conflict.dictation"
      ],
      "id": "shortcut-settings-conflict.general",
      "role": "container",
      "screen_id": "shortcut-settings-conflict",
      "semantic_id": "shortcut-settings-conflict.general",
      "test_id": "shortcut-settings-conflict-general",
      "text": "General"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "multiple-dictation-shortcuts",
    "id": "shortcut-settings-conflict",
    "root_node": "shortcut-settings-conflict",
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
        "id": "shortcut-removed"
      }
    ],
    "surface": "desktop",
    "title": "Dictation shortcuts — conflicts",
    "viewport": {
      "height": 570,
      "width": 680
    }
  }
}
```
