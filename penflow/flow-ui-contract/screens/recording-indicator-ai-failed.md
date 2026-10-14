<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-ai-failed

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [],
  "nodes": [
    {
      "children": [
        "recording-indicator-ai-failed.notice"
      ],
      "id": "recording-indicator-ai-failed",
      "role": "screen",
      "screen_id": "recording-indicator-ai-failed",
      "semantic_id": "recording-indicator-ai-failed",
      "test_id": "recording-indicator-ai-failed-screen"
    },
    {
      "id": "recording-indicator-ai-failed.notice",
      "role": "alert",
      "screen_id": "recording-indicator-ai-failed",
      "semantic_id": "recording-indicator-ai-failed.notice",
      "test_id": "recording-indicator-ai-failed-notice",
      "text": "The AI step failed. The original text was used for insertion."
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "custom-ai-prompts",
    "id": "recording-indicator-ai-failed",
    "root_node": "recording-indicator-ai-failed",
    "states": [
      {
        "id": "default"
      }
    ],
    "static": true,
    "static_reason": "Transient failure notice shown for at least 4 seconds on the indicator; it has no controls.",
    "surface": "desktop",
    "title": "Recording indicator — AI step failed",
    "viewport": {
      "height": 200,
      "width": 560
    }
  }
}
```
