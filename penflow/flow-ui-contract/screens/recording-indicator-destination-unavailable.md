<!-- Generated from contract.json; do not edit. -->
<!-- @spec(FR-011) -->

# Screen: recording-indicator-destination-unavailable

Generator: `penflow.flow-docs/1`

```json
{
  "actions": [],
  "nodes": [
    {
      "children": [
        "recording-indicator-destination-unavailable.notice"
      ],
      "id": "recording-indicator-destination-unavailable",
      "role": "screen",
      "screen_id": "recording-indicator-destination-unavailable",
      "semantic_id": "recording-indicator-destination-unavailable",
      "test_id": "recording-indicator-destination-unavailable-screen"
    },
    {
      "id": "recording-indicator-destination-unavailable.notice",
      "role": "alert",
      "screen_id": "recording-indicator-destination-unavailable",
      "semantic_id": "recording-indicator-destination-unavailable.notice",
      "test_id": "recording-indicator-destination-unavailable-notice",
      "text": "Notes is closed. Nothing was inserted; the transcript is kept in History."
    }
  ],
  "relations": [],
  "screen": {
    "flow_id": "destination-application",
    "id": "recording-indicator-destination-unavailable",
    "root_node": "recording-indicator-destination-unavailable",
    "states": [
      {
        "id": "default"
      }
    ],
    "static": true,
    "static_reason": "Transient reason notice shown for at least 4 seconds in place of the indicator; it has no controls.",
    "surface": "desktop",
    "title": "Recording indicator — destination unavailable",
    "viewport": {
      "height": 200,
      "width": 560
    }
  }
}
```
