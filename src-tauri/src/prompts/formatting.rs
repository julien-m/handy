//! Request formatting and reply cleaning of the AI step, moved unchanged from `actions.rs`
//! (005 Step 4): the structured-output schema and system prompt, the legacy `${output}`
//! substitution, and the cleaning of a reply (leading think block, invisible characters).

/// Field name of the structured output JSON schema.
pub const TRANSCRIPTION_FIELD: &str = "transcription";

/// Strip invisible Unicode characters that some LLMs may insert.
pub fn strip_invisible_chars(s: &str) -> String {
    s.replace(['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'], "")
}

/// Strip a leading `<think>...</think>` block. Some endpoints can't disable reasoning, and some
/// local servers put the reasoning text into `content` instead of a separate field — without
/// this the user would get the model's chain of thought pasted along with the result.
pub fn strip_think_block(s: &str) -> &str {
    if let Some(rest) = s.trim_start().strip_prefix("<think>") {
        if let Some(end) = rest.find("</think>") {
            return rest[end + "</think>".len()..].trim_start();
        }
    }
    s
}

/// A reply as inserted: think block and invisible characters removed.
pub fn clean_reply(content: &str) -> String {
    strip_invisible_chars(strip_think_block(content))
}

/// Build a system prompt from the user's prompt template. Removes the `${output}` placeholder
/// since the transcription is sent as the user message.
pub fn build_system_prompt(prompt_template: &str) -> String {
    prompt_template.replace("${output}", "").trim().to_string()
}

/// Legacy request: the prompt with `${output}` replaced by the transcript (a prompt without the
/// placeholder is sent as is, as before).
pub fn legacy_prompt(prompt_template: &str, transcription: &str) -> String {
    prompt_template.replace("${output}", transcription)
}

/// JSON schema asking for the processed transcript in `TRANSCRIPTION_FIELD`.
pub fn transcription_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            (TRANSCRIPTION_FIELD): {
                "type": "string",
                "description": "The cleaned and processed transcription text"
            }
        },
        "required": [TRANSCRIPTION_FIELD],
        "additionalProperties": false
    })
}

/// Text of a structured reply: its `TRANSCRIPTION_FIELD`, or (backward compatibility with
/// endpoints that ignore the schema, kept from before 005) the whole reply when it is not JSON or
/// lacks the field.
pub fn structured_text(content: &str) -> String {
    let content = strip_think_block(content);
    let field = serde_json::from_str::<serde_json::Value>(content)
        .ok()
        .and_then(|json| json.get(TRANSCRIPTION_FIELD)?.as_str().map(str::to_owned));
    match field {
        Some(text) => strip_invisible_chars(&text),
        None => {
            log::debug!("Structured reply without the transcription field; using it as is");
            strip_invisible_chars(content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_think_block_is_stripped() {
        assert_eq!(
            strip_think_block("<think>pondering...</think>Cleaned text."),
            "Cleaned text."
        );
        assert_eq!(
            strip_think_block("  \n<think>multi\nline</think>\n  Cleaned text."),
            "Cleaned text."
        );
    }
    #[test]
    fn content_without_think_block_is_unchanged() {
        assert_eq!(strip_think_block("Cleaned text."), "Cleaned text.");
        assert_eq!(
            strip_think_block("Mentions <think> mid-sentence."),
            "Mentions <think> mid-sentence."
        );
        // Unclosed block: leave untouched rather than guess
        assert_eq!(
            strip_think_block("<think>never closed"),
            "<think>never closed"
        );
    }
    #[test]
    fn structured_replies_use_the_field_or_the_whole_non_json_reply() {
        assert_eq!(
            structured_text(r#"{"transcription":"Hi​ there"}"#),
            "Hi there"
        );
        assert_eq!(structured_text("plain reply"), "plain reply");
        assert_eq!(structured_text(r#"{"other":"x"}"#), r#"{"other":"x"}"#);
    }
    #[test]
    fn prompts_without_placeholder_are_sent_as_before() {
        assert_eq!(legacy_prompt("Fix: ${output}", "hello"), "Fix: hello");
        assert_eq!(legacy_prompt("No placeholder", "hello"), "No placeholder");
        assert_eq!(build_system_prompt("Fix: ${output}"), "Fix:");
    }
}
