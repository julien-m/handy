//! Wire protocol of the ChatGPT dictation stream, as used by the ChatGPT
//! desktop app (observed in version 26.1117, private and undocumented): a JSON
//! WebSocket where the client sends `session.start`, `audio.append` (base64
//! PCM16) and `session.close`, and the server answers with session, speech and
//! transcript events.

use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;

/// Desktop dictation endpoint; `dictation_surface=global` is the system-wide
/// dictation surface, `client_type=desktop` the desktop client family.
pub(super) const STREAM_URL: &str =
    "wss://chatgpt.com/backend-api/dictation/stream?dictation_surface=global&client_type=desktop";
/// WebSocket subprotocols the desktop app announces.
pub(super) const SUBPROTOCOLS: &str = "chatgpt-dictation, codex-desktop";
/// Handy's recorder delivers 16 kHz mono frames; the server accepts the rate
/// declared in `session.start` (verified 2026-11-23).
const SAMPLE_RATE_HZ: u32 = 16_000;
/// Values copied from the desktop app's `session.start`: 4 MiB server buffer,
/// 30 s per utterance, 5 min per session, and its server VAD tuning.
const MAX_BUFFER_SIZE_BYTES: u32 = 4_194_304;
const MAX_UTTERANCE_DURATION_MS: u32 = 30_000;
const SESSION_TTL_MS: u32 = 300_000;

/// Build the `session.start` message. `language` `None` lets the server detect it.
pub(super) fn session_start(session_id: &str, attempt_id: &str, language: Option<&str>) -> String {
    json!({
        "type": "session.start",
        "dictation_session_id": session_id,
        "attempt_id": attempt_id,
        "config": {
            "language": language,
            "input_audio_format": "pcm16",
            "sample_rate_hz": SAMPLE_RATE_HZ,
            "num_channels": 1,
            "max_buffer_size_bytes": MAX_BUFFER_SIZE_BYTES,
            "max_utterance_duration_ms": MAX_UTTERANCE_DURATION_MS,
            "session_ttl_ms": SESSION_TTL_MS,
            "provider_mode": "streaming_sse",
            "transcript_delivery_mode": "segment",
            "vad": {"type": "server_vad", "threshold": 0.5,
                    "prefix_padding_ms": 300, "silence_duration_ms": 500}
        }
    })
    .to_string()
}

/// Build an `audio.append` message from 16 kHz mono float samples.
pub(super) fn audio_append(samples: &[f32]) -> String {
    // Lossy by design: float samples are clamped to [-1, 1] and quantized to
    // the 16-bit PCM the stream expects.
    let pcm: Vec<u8> = samples
        .iter()
        .flat_map(|sample| ((sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16).to_le_bytes())
        .collect();
    let audio = base64::engine::general_purpose::STANDARD.encode(pcm);
    json!({"type": "audio.append", "audio": audio}).to_string()
}

/// The `session.close` message: ends input and asks for the final text.
pub(super) fn session_close() -> String {
    json!({"type": "session.close"}).to_string()
}

/// Server events Handy needs; any other event type parses as `Other`.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub(super) enum ServerEvent {
    #[serde(rename = "session.started")]
    SessionStarted { session: SessionInfo },
    #[serde(rename = "session.updated")]
    SessionUpdated { session: SessionInfo },
    #[serde(rename = "transcript.segment")]
    Segment {
        utterance_id: String,
        revision: u64,
        text: String,
    },
    #[serde(rename = "transcript.final")]
    Final {
        utterance_id: String,
        revision: u64,
        text: String,
    },
    #[serde(rename = "transcript.failed")]
    Failed { error: ServerError },
    #[serde(rename = "session.error")]
    Error { fatal: bool, error: ServerError },
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(super) struct SessionInfo {
    pub(super) session_id: String,
    pub(super) status: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub(super) struct ServerError {
    pub(super) code: String,
    pub(super) message: String,
}

/// Session status the server reports once the final text has been sent.
pub(super) const SESSION_CLOSED: &str = "closed";

/// Parse one server text frame; `None` when it is not a known event shape.
pub(super) fn parse_event(text: &str) -> Option<ServerEvent> {
    serde_json::from_str(text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_start_declares_rate_and_language() {
        let message: serde_json::Value =
            serde_json::from_str(&session_start("s", "a", Some("fr"))).unwrap();
        assert_eq!(message["type"], "session.start");
        assert_eq!(message["config"]["sample_rate_hz"], 16_000);
        assert_eq!(message["config"]["language"], "fr");
        let auto: serde_json::Value = serde_json::from_str(&session_start("s", "a", None)).unwrap();
        assert!(auto["config"]["language"].is_null());
    }

    #[test]
    fn audio_append_encodes_clamped_pcm16() {
        let message: serde_json::Value =
            serde_json::from_str(&audio_append(&[0.0, 2.0, -2.0])).unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(message["audio"].as_str().unwrap())
            .unwrap();
        let samples: Vec<i16> = bytes
            .chunks(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        assert_eq!(samples, vec![0, i16::MAX, -i16::MAX]);
    }

    #[test]
    fn parse_event_reads_segments_and_ignores_unknown_types() {
        assert_eq!(
            parse_event(
                r#"{"type":"transcript.segment","sequence_no":3,"utterance_id":"u","revision":2,"text":" Bonjour"}"#
            ),
            Some(ServerEvent::Segment {
                utterance_id: "u".into(),
                revision: 2,
                text: " Bonjour".into()
            })
        );
        assert_eq!(
            parse_event(r#"{"type":"audio.frame_processed","sequence_no":1}"#),
            Some(ServerEvent::Other)
        );
        assert_eq!(parse_event("garbage"), None);
    }
}
