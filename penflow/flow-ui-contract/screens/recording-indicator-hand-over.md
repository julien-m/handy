<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-hand-over

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [],
  "nodes": [
    {
      "children": [
        "recording-indicator-hand-over.notice"
      ],
      "id": "recording-indicator-hand-over",
      "role": "screen",
      "screen_id": "recording-indicator-hand-over",
      "semantic_id": "recording-indicator-hand-over",
      "test_id": "recording-indicator-hand-over-screen"
    },
    {
      "id": "recording-indicator-hand-over.notice",
      "role": "alert",
      "screen_id": "recording-indicator-hand-over",
      "semantic_id": "recording-indicator-hand-over.notice",
      "test_id": "recording-indicator-hand-over-notice",
      "text": "Text placed on the clipboard for the remote screen"
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "smart-paste",
    "id": "recording-indicator-hand-over",
    "root_node": "recording-indicator-hand-over",
    "states": [
      {
        "id": "default"
      }
    ],
    "static": true,
    "static_reason": "Transient notice shown for at least 4 seconds on the indicator when the text goes to a remote screen; it has no controls.",
    "surface": "desktop",
    "title": "Recording indicator — hand-over to a remote screen",
    "viewport": {
      "height": 200,
      "width": 560
    }
  }
}
```
