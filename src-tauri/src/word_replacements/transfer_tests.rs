//! Staging of large imports: ordered chunks, hostile or stale tokens, expiry and capacity.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::{
    sha256_hex, StageError, StagingArea, MAX_STAGED_INPUT_BYTES, MAX_STAGED_TOTAL_BYTES, STAGE_TTL,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// An area on a manual clock, and the handle that moves that clock.
fn area() -> (StagingArea, Arc<Mutex<Instant>>) {
    let now = Arc::new(Mutex::new(Instant::now()));
    let clock = Arc::clone(&now);
    let area = StagingArea::new(Arc::new(move || {
        *clock.lock().unwrap_or_else(PoisonError::into_inner)
    }));
    (area, now)
}
fn advance(now: &Mutex<Instant>, by: Duration) {
    *now.lock().unwrap_or_else(PoisonError::into_inner) += by;
}

#[test]
fn ordered_chunks_are_taken_once_as_the_declared_input() -> TestResult {
    let (area, _) = area();
    let input = "recognized;replacement\ncloud code;Claude Code\n";
    let token = area
        .begin(input.len(), &sha256_hex(input))
        .map_err(|e| format!("{e:?}"))?;
    let (first, second) = input.split_at(10);
    assert_eq!(area.append(&token, 0, first), Ok(()));
    assert_eq!(area.append(&token, 1, second), Ok(()));
    assert_eq!(area.take(&token), Ok(input.to_owned()));
    assert_eq!(area.take(&token), Err(StageError::Unknown), "taken once");
    assert_eq!(area.reserved_bytes(), 0);
    Ok(())
}

#[test]
fn missing_duplicated_or_reordered_chunks_never_complete() -> TestResult {
    let (area, _) = area();
    let input = "abcdef";
    let token = area
        .begin(input.len(), &sha256_hex(input))
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(area.append(&token, 1, "def"), Err(StageError::Mismatch));
    assert_eq!(area.append(&token, 0, "abc"), Ok(()));
    assert_eq!(area.append(&token, 0, "abc"), Err(StageError::Mismatch));
    assert_eq!(area.take(&token), Err(StageError::Mismatch), "incomplete");
    let altered = area
        .begin(3, &sha256_hex("abc"))
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(area.append(&altered, 0, "abd"), Ok(()));
    assert_eq!(area.take(&altered), Err(StageError::Mismatch), "altered");
    Ok(())
}

#[test]
fn foreign_and_expired_tokens_are_unknown() -> TestResult {
    let (area, now) = area();
    assert_eq!(area.append("guessed", 0, "x"), Err(StageError::Unknown));
    let token = area
        .begin(1, &sha256_hex("x"))
        .map_err(|e| format!("{e:?}"))?;
    advance(&now, STAGE_TTL);
    assert_eq!(area.append(&token, 0, "x"), Err(StageError::Unknown));
    assert_eq!(area.reserved_bytes(), 0, "expired input dropped");
    Ok(())
}

#[test]
fn inputs_and_the_process_capacity_are_bounded() -> TestResult {
    let (area, _) = area();
    let too_large = area.begin(MAX_STAGED_INPUT_BYTES + 1, "00");
    assert_eq!(too_large, Err(StageError::TooLarge));
    let token = area
        .begin(2, &sha256_hex("ab"))
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(area.append(&token, 0, "abc"), Err(StageError::TooLarge));
    let fills = MAX_STAGED_TOTAL_BYTES / MAX_STAGED_INPUT_BYTES;
    for _ in 1..fills {
        area.begin(MAX_STAGED_INPUT_BYTES, "00")
            .map_err(|e| format!("{e:?}"))?;
    }
    let refused = area.begin(MAX_STAGED_INPUT_BYTES, "00");
    assert_eq!(refused, Err(StageError::Saturated));
    Ok(())
}
