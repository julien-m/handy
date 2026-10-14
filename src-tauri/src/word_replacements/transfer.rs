//! Bounded staging of an import too large for one message of the private settings channel
//! (1 MiB once escaped). The CLI declares the input's byte length and SHA-256, sends it in
//! ordered chunks under an unguessable token, then imports by token; the import takes the whole
//! input at once, so a missing, duplicated or reordered chunk never reaches the rules. Staged
//! text lives only in memory, is never a settings write, and is dropped when taken, when its
//! five-second deadline passes, or when the process exits.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

/// Deadline of a staged input: the settings channel's total deadline for one operation.
pub const STAGE_TTL: Duration = Duration::from_secs(5);
/// Bound of one staged input: the CLI's domain input bound (1 MiB).
pub const MAX_STAGED_INPUT_BYTES: usize = 1 << 20;
/// Bound of all staged inputs of a process: 32 concurrent maximal imports. Beyond it the
/// channel is saturated and staging is refused; nothing else is affected.
pub const MAX_STAGED_TOTAL_BYTES: usize = 32 << 20;

/// Why a staging step is refused; never carries staged text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageError {
    /// Unknown, expired or already taken token.
    Unknown,
    /// A chunk out of order, or an incomplete or altered input at import.
    Mismatch,
    /// The declared length is above `MAX_STAGED_INPUT_BYTES`, or chunks exceed it.
    TooLarge,
    /// All staging capacity is in use.
    Saturated,
}

struct Staged {
    text: String,
    next_index: u64,
    declared_len: usize,
    sha256: String,
    expires: Instant,
}

/// Monotonic time source, injected so expiry is testable.
pub type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

/// Staged inputs of this process by token.
pub struct StagingArea {
    staged: Mutex<HashMap<String, Staged>>,
    clock: Clock,
    sweeping: AtomicBool,
}

impl StagingArea {
    /// An empty area reading time from `clock`.
    pub fn new(clock: Clock) -> Self {
        Self {
            staged: Mutex::new(HashMap::new()),
            clock,
            sweeping: AtomicBool::new(false),
        }
    }

    /// Reserve a staged input of `declared_len` bytes with this SHA-256 (lowercase hex).
    ///
    /// # Errors
    /// `TooLarge` above the input bound; `Saturated` when the process bound would be passed.
    pub fn begin(&self, declared_len: usize, sha256: &str) -> Result<String, StageError> {
        if declared_len > MAX_STAGED_INPUT_BYTES {
            return Err(StageError::TooLarge);
        }
        let mut staged = self.live_entries();
        let reserved: usize = staged.values().map(|entry| entry.declared_len).sum();
        if reserved + declared_len > MAX_STAGED_TOTAL_BYTES {
            return Err(StageError::Saturated);
        }
        let token = uuid::Uuid::new_v4().simple().to_string();
        let entry = Staged {
            text: String::with_capacity(declared_len),
            next_index: 0,
            declared_len,
            sha256: sha256.to_ascii_lowercase(),
            expires: (self.clock)() + STAGE_TTL,
        };
        staged.insert(token.clone(), entry);
        Ok(token)
    }

    /// Append chunk `index` (0-based, in order) to the input of `token`.
    ///
    /// # Errors
    /// `Unknown` token, `Mismatch` for an out-of-order chunk, `TooLarge` past the declared length.
    pub fn append(&self, token: &str, index: u64, chunk: &str) -> Result<(), StageError> {
        let mut staged = self.live_entries();
        let entry = staged.get_mut(token).ok_or(StageError::Unknown)?;
        if index != entry.next_index {
            return Err(StageError::Mismatch);
        }
        if entry.text.len() + chunk.len() > entry.declared_len {
            return Err(StageError::TooLarge);
        }
        entry.text.push_str(chunk);
        entry.next_index += 1;
        Ok(())
    }

    /// Remove the input of `token` and return it when it is complete and unaltered.
    ///
    /// # Errors
    /// `Unknown` token; `Mismatch` when the length or SHA-256 differs from the declaration (the
    /// staged input is dropped either way).
    pub fn take(&self, token: &str) -> Result<String, StageError> {
        let entry = self
            .live_entries()
            .remove(token)
            .ok_or(StageError::Unknown)?;
        let digest = format!("{:x}", Sha256::digest(entry.text.as_bytes()));
        if entry.text.len() != entry.declared_len || digest != entry.sha256 {
            return Err(StageError::Mismatch);
        }
        Ok(entry.text)
    }

    /// Bytes reserved by staged inputs that have not expired.
    pub fn reserved_bytes(&self) -> usize {
        let staged = self.live_entries();
        staged.values().map(|entry| entry.declared_len).sum()
    }

    /// Staged inputs after dropping the expired ones.
    fn live_entries(&self) -> std::sync::MutexGuard<'_, HashMap<String, Staged>> {
        let mut staged = self.staged.lock().unwrap_or_else(PoisonError::into_inner);
        let now = (self.clock)();
        staged.retain(|_, entry| entry.expires > now);
        staged
    }
}

/// The staging area of this process (a running instance, or a CLI process routing offline).
pub fn staging() -> &'static StagingArea {
    static AREA: OnceLock<StagingArea> = OnceLock::new();
    AREA.get_or_init(|| StagingArea::new(Arc::new(Instant::now)))
}

/// Make sure abandoned inputs of the process area are dropped at their deadline even when no
/// other staging call comes: one background sweep runs while inputs are staged.
pub fn schedule_sweep() {
    let area = staging();
    if area.sweeping.swap(true, Ordering::AcqRel) {
        return;
    }
    std::thread::spawn(move || loop {
        std::thread::sleep(STAGE_TTL);
        if area.reserved_bytes() > 0 {
            continue;
        }
        area.sweeping.store(false, Ordering::Release);
        // Exit unless a new input was staged after the check above (it then owns no sweeper).
        if area.reserved_bytes() == 0 || area.sweeping.swap(true, Ordering::AcqRel) {
            return;
        }
    });
}

/// SHA-256 of `text` as lowercase hex, as `begin` expects it.
pub fn sha256_hex(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
