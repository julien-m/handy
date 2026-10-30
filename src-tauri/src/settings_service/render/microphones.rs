//! Text rendering of `handy settings microphones list` (feature 001): the saved choices with
//! their availability, then the microphone in use.
use super::plain;
use serde_json::Value;

/// `N. name: status` per saved choice, then the microphone in use (feature 001).
pub(super) fn microphone_lines(data: &Value) -> String {
    let choices = data["availability"].as_array().cloned().unwrap_or_default();
    let mut text = String::from("Microphone priority:\n");
    for (index, choice) in choices.iter().enumerate() {
        let status = choice["status"].as_str().unwrap_or("unknown");
        text.push_str(&format!(
            "{}. {}: {status}\n",
            index + 1,
            plain(&choice["name"])
        ));
    }
    if choices.is_empty() {
        text.push_str("(none: system default)\n");
    }
    if !data["warning"].is_null() {
        text.push_str("Devices could not be listed: availability is unknown.\n");
    }
    text + &in_use_line(&data["effective_input"])
}
/// The microphone in use now, or used by the last recording, or "not used yet".
fn in_use_line(input: &Value) -> String {
    match input["scope"].as_str() {
        Some(scope @ ("current" | "last_recording")) => format!(
            "In use ({scope}): {} ({})\n",
            plain(&input["name"]),
            plain(&input["reason"])
        ),
        _ => "In use: not used yet\n".to_owned(),
    }
}
