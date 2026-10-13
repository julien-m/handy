//! Domain input of the CLI process: a file path, or `-` for stdin, read as bounded UTF-8 text
//! before the operation is routed (a running instance never reads the caller's files or stdin).
use super::errors::{SettingsError, SettingsErrorCode, ValidationReason};
use std::io::{IsTerminal, Read};

/// Input bound: the private channel's frame limit, which every domain shares until its owning
/// feature defines another.
pub const MAX_INPUT_BYTES: usize = super::ipc::MAX_FRAME_BYTES;

/// Read the input named by `source`: `-` is the process's stdin, anything else a file path.
///
/// Side effects: consumes stdin for `-`; never prompts (a terminal stdin is refused instead of
/// waiting for typed input).
///
/// # Errors
/// `usage_error` for a terminal stdin or an unreadable file; `limit_exceeded` above
/// `MAX_INPUT_BYTES`; `validation_failed` (reason `invalid_encoding`, `details.offset`) for non-UTF-8.
// @spec FR-005: File and stdin domain input — .specs/features/008-settings-cli/spec.md#fr-005
pub fn read_source(source: &str) -> Result<String, SettingsError> {
    if source == "-" {
        let stdin = std::io::stdin();
        // Non-TTY fallback only: an interactive terminal would make the command wait for typing.
        if stdin.is_terminal() {
            return Err(SettingsError::new(SettingsErrorCode::UsageError));
        }
        return read_bounded(stdin.lock());
    }
    let file = std::fs::File::open(source)
        .map_err(|_| SettingsError::new(SettingsErrorCode::UsageError))?;
    read_bounded(file)
}
/// Read at most `MAX_INPUT_BYTES` (one more byte detects an oversized input) as UTF-8 text.
///
/// # Errors
/// Same as `read_source`, except the terminal refusal.
pub fn read_bounded(reader: impl Read) -> Result<String, SettingsError> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| SettingsError::new(SettingsErrorCode::UsageError))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(SettingsError::new(SettingsErrorCode::LimitExceeded));
    }
    String::from_utf8(bytes).map_err(|error| {
        let offset = error.utf8_error().valid_up_to() as u64;
        SettingsError::invalid_input(ValidationReason::InvalidEncoding, Some(offset))
    })
}
