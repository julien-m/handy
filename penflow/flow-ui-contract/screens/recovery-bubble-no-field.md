<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recovery-bubble-no-field

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
    }
  ],
  "nodes": [
    {
      "children": [
        "recovery-bubble-no-field.bubble"
      ],
      "id": "recovery-bubble-no-field",
      "role": "screen",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field",
      "test_id": "recovery-bubble-no-field-screen"
    },
    {
      "children": [
        "recovery-bubble-no-field.reason",
        "recovery-bubble-no-field.close",
        "recovery-bubble-no-field.preview",
        "recovery-bubble-no-field.drag-handle",
        "recovery-bubble-no-field.copy",
        "recovery-bubble-no-field.why"
      ],
      "id": "recovery-bubble-no-field.bubble",
      "role": "container",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.bubble",
      "test_id": "recovery-bubble-no-field-bubble"
    },
    {
      "action": "recovery-bubble-no-field.close",
      "id": "recovery-bubble-no-field.close",
      "role": "button",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.close",
      "test_id": "recovery-bubble-no-field-close"
    },
    {
      "action": "recovery-bubble-no-field.copy",
      "id": "recovery-bubble-no-field.copy",
      "role": "button",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.copy",
      "test_id": "recovery-bubble-no-field-copy",
      "text": "Copy"
    },
    {
      "action": "recovery-bubble-no-field.drag-handle",
      "id": "recovery-bubble-no-field.drag-handle",
      "role": "button",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.drag-handle",
      "test_id": "recovery-bubble-no-field-drag-handle",
      "text": "Drag into a field"
    },
    {
      "id": "recovery-bubble-no-field.preview",
      "role": "text",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.preview",
      "test_id": "recovery-bubble-no-field-preview",
      "text": "Hi Marie, the release moves to Thursday. I'll send the checklist tonight so we can review the…"
    },
    {
      "id": "recovery-bubble-no-field.reason",
      "role": "text",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.reason",
      "test_id": "recovery-bubble-no-field-reason",
      "text": "No text field is focused"
    },
    {
      "action": "recovery-bubble-no-field.why",
      "id": "recovery-bubble-no-field.why",
      "role": "button",
      "screen_id": "recovery-bubble-no-field",
      "semantic_id": "recovery-bubble-no-field.why",
      "test_id": "recovery-bubble-no-field-why",
      "text": "Why?"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recovery-bubble",
    "id": "recovery-bubble-no-field",
    "root_node": "recovery-bubble-no-field",
    "states": [
      {
        "id": "default"
      },
      {
        "id": "closed"
      },
      {
        "id": "copied"
      },
      {
        "id": "dragging"
      },
      {
        "id": "explained"
      }
    ],
    "surface": "desktop",
    "title": "Recovery bubble — no text field",
    "viewport": {
      "height": 260,
      "width": 560
    }
  }
}
```
