<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recovery-bubble-insertion-failed

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [
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
      "guard": "recovery-bubble-visible",
      "id": "recovery-bubble-insertion-failed.why",
      "kind": "state_change",
      "node": "recovery-bubble-insertion-failed.why",
      "success_transition": "recovery-bubble-insertion-failed.why.explained",
      "trigger": "click"
    }
  ],
  "nodes": [
    {
      "children": [
        "recovery-bubble-insertion-failed.bubble"
      ],
      "id": "recovery-bubble-insertion-failed",
      "role": "screen",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed",
      "test_id": "recovery-bubble-insertion-failed-screen"
    },
    {
      "children": [
        "recovery-bubble-insertion-failed.reason",
        "recovery-bubble-insertion-failed.close",
        "recovery-bubble-insertion-failed.preview",
        "recovery-bubble-insertion-failed.explanation",
        "recovery-bubble-insertion-failed.copy",
        "recovery-bubble-insertion-failed.why"
      ],
      "id": "recovery-bubble-insertion-failed.bubble",
      "role": "container",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.bubble",
      "test_id": "recovery-bubble-insertion-failed-bubble"
    },
    {
      "action": "recovery-bubble-insertion-failed.close",
      "id": "recovery-bubble-insertion-failed.close",
      "role": "button",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.close",
      "test_id": "recovery-bubble-insertion-failed-close"
    },
    {
      "action": "recovery-bubble-insertion-failed.copy",
      "id": "recovery-bubble-insertion-failed.copy",
      "role": "button",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.copy",
      "test_id": "recovery-bubble-insertion-failed-copy",
      "text": "Copy"
    },
    {
      "children": [
        "recovery-bubble-insertion-failed.explanation.text"
      ],
      "id": "recovery-bubble-insertion-failed.explanation",
      "role": "container",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.explanation",
      "test_id": "recovery-bubble-insertion-failed-explanation"
    },
    {
      "id": "recovery-bubble-insertion-failed.explanation.text",
      "role": "text",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.explanation.text",
      "test_id": "recovery-bubble-insertion-failed-explanation-text",
      "text": "Some text may have reached the application. The full transcript is kept in History."
    },
    {
      "id": "recovery-bubble-insertion-failed.preview",
      "role": "text",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.preview",
      "test_id": "recovery-bubble-insertion-failed-preview",
      "text": "Hi Marie, the release moves to Thursday. I'll send the checklist tonight so we can review the…"
    },
    {
      "id": "recovery-bubble-insertion-failed.reason",
      "role": "text",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.reason",
      "test_id": "recovery-bubble-insertion-failed-reason",
      "text": "Insertion failed"
    },
    {
      "action": "recovery-bubble-insertion-failed.why",
      "id": "recovery-bubble-insertion-failed.why",
      "role": "button",
      "screen_id": "recovery-bubble-insertion-failed",
      "semantic_id": "recovery-bubble-insertion-failed.why",
      "test_id": "recovery-bubble-insertion-failed-why",
      "text": "Hide details"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "recovery-bubble",
    "id": "recovery-bubble-insertion-failed",
    "root_node": "recovery-bubble-insertion-failed",
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
        "id": "explained"
      }
    ],
    "surface": "desktop",
    "title": "Recovery bubble — insertion failed (Windows, no drag)",
    "viewport": {
      "height": 300,
      "width": 560
    }
  }
}
```
