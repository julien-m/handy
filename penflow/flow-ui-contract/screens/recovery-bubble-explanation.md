<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recovery-bubble-explanation

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
    }
  ],
  "nodes": [
    {
      "children": [
        "recovery-bubble-explanation.bubble"
      ],
      "id": "recovery-bubble-explanation",
      "role": "screen",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation",
      "test_id": "recovery-bubble-explanation-screen"
    },
    {
      "children": [
        "recovery-bubble-explanation.reason",
        "recovery-bubble-explanation.close",
        "recovery-bubble-explanation.preview",
        "recovery-bubble-explanation.explanation",
        "recovery-bubble-explanation.drag-handle",
        "recovery-bubble-explanation.copy",
        "recovery-bubble-explanation.why"
      ],
      "id": "recovery-bubble-explanation.bubble",
      "role": "container",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.bubble",
      "test_id": "recovery-bubble-explanation-bubble"
    },
    {
      "action": "recovery-bubble-explanation.close",
      "id": "recovery-bubble-explanation.close",
      "role": "button",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.close",
      "test_id": "recovery-bubble-explanation-close"
    },
    {
      "action": "recovery-bubble-explanation.copy",
      "id": "recovery-bubble-explanation.copy",
      "role": "button",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.copy",
      "test_id": "recovery-bubble-explanation-copy",
      "text": "Copy"
    },
    {
      "action": "recovery-bubble-explanation.drag-handle",
      "id": "recovery-bubble-explanation.drag-handle",
      "role": "button",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.drag-handle",
      "test_id": "recovery-bubble-explanation-drag-handle",
      "text": "Drag into a field"
    },
    {
      "children": [
        "recovery-bubble-explanation.explanation.text"
      ],
      "id": "recovery-bubble-explanation.explanation",
      "role": "container",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.explanation",
      "test_id": "recovery-bubble-explanation-explanation"
    },
    {
      "id": "recovery-bubble-explanation.explanation.text",
      "role": "text",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.explanation.text",
      "test_id": "recovery-bubble-explanation-explanation-text",
      "text": "Handy didn't insert anything and sent no Enter. The transcript is kept in History. Open ChatGPT, then drag or copy the text."
    },
    {
      "id": "recovery-bubble-explanation.preview",
      "role": "text",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.preview",
      "test_id": "recovery-bubble-explanation-preview",
      "text": "Hi Marie, the release moves to Thursday. I'll send the checklist tonight so we can review the…"
    },
    {
      "id": "recovery-bubble-explanation.reason",
      "role": "text",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.reason",
      "test_id": "recovery-bubble-explanation-reason",
      "text": "ChatGPT is pinned but not open"
    },
    {
      "action": "recovery-bubble-explanation.why",
      "id": "recovery-bubble-explanation.why",
      "role": "button",
      "screen_id": "recovery-bubble-explanation",
      "semantic_id": "recovery-bubble-explanation.why",
      "test_id": "recovery-bubble-explanation-why",
      "text": "Hide details"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recovery-bubble",
    "id": "recovery-bubble-explanation",
    "root_node": "recovery-bubble-explanation",
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
    "title": "Recovery bubble — pinned app closed, details",
    "viewport": {
      "height": 330,
      "width": 560
    }
  }
}
```
