<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recovery-bubble

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
    }
  ],
  "nodes": [
    {
      "children": [
        "recovery-bubble.bubble"
      ],
      "id": "recovery-bubble",
      "role": "screen",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble",
      "test_id": "recovery-bubble-screen"
    },
    {
      "children": [
        "recovery-bubble.reason",
        "recovery-bubble.close",
        "recovery-bubble.preview",
        "recovery-bubble.drag-handle",
        "recovery-bubble.copy",
        "recovery-bubble.why"
      ],
      "id": "recovery-bubble.bubble",
      "role": "container",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.bubble",
      "test_id": "recovery-bubble-bubble"
    },
    {
      "action": "recovery-bubble.close",
      "id": "recovery-bubble.close",
      "role": "button",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.close",
      "test_id": "recovery-bubble-close"
    },
    {
      "action": "recovery-bubble.copy",
      "id": "recovery-bubble.copy",
      "role": "button",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.copy",
      "test_id": "recovery-bubble-copy",
      "text": "Copy"
    },
    {
      "action": "recovery-bubble.drag-handle",
      "id": "recovery-bubble.drag-handle",
      "role": "button",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.drag-handle",
      "test_id": "recovery-bubble-drag-handle",
      "text": "Drag into a field"
    },
    {
      "id": "recovery-bubble.preview",
      "role": "text",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.preview",
      "test_id": "recovery-bubble-preview",
      "text": "Hi Marie, the release moves to Thursday. I'll send the checklist tonight so we can review the…"
    },
    {
      "id": "recovery-bubble.reason",
      "role": "text",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.reason",
      "test_id": "recovery-bubble-reason",
      "text": "Notes is no longer open"
    },
    {
      "action": "recovery-bubble.why",
      "id": "recovery-bubble.why",
      "role": "button",
      "screen_id": "recovery-bubble",
      "semantic_id": "recovery-bubble.why",
      "test_id": "recovery-bubble-why",
      "text": "Why?"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recovery-bubble",
    "id": "recovery-bubble",
    "root_node": "recovery-bubble",
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
    "title": "Recovery bubble",
    "viewport": {
      "height": 260,
      "width": 560
    }
  }
}
```
