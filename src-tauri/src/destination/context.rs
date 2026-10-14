//! Typed contracts handed by the destination manager to insertion methods (007), to the Enter
//! policy (009) and to the recovery bubble (010): application identity, the per-insertion
//! `DestinationContext`, the single insert-or-recover `RecoveryDisposition` and the optional
//! pre-insertion check of 010.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Immutable identity of one accepted dictation (the same value 007 puts in its `PasteOutcome`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(transparent)]
pub struct DictationId(pub u64);

/// Stable application identity: bundle identifier on macOS; application user model ID on
/// Windows when available, otherwise the canonical executable path.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, specta::Type)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ApplicationIdentity {
    BundleId(String),
    Aumid(String),
    ExecutablePath(PathBuf),
}
impl ApplicationIdentity {
    /// macOS bundle identifier, for 007's per-application strategies.
    pub fn bundle_id(&self) -> Option<&str> {
        match self {
            Self::BundleId(bundle_id) => Some(bundle_id),
            Self::Aumid(_) | Self::ExecutablePath(_) => None,
        }
    }
}

/// The running instance an insertion targets: its process and stable identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedTarget {
    pub pid: u32,
    pub identity: ApplicationIdentity,
}

/// Where the destination of a dictation comes from (009 adds `Pinned`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DestinationSource {
    /// Frontmost application when the recording actually started.
    Captured,
    /// Chosen explicitly from the badge menu during the dictation.
    Badge,
    /// No destination: the application frontmost at insertion, as before this feature.
    FrontmostAtEnd,
}

// @spec FR-005: Destination context for insertion methods — .specs/features/006-destination-application/spec.md#fr-005
/// The only value handed to an insertion method and to the recovery check, built at the
/// pre-dispatch guard after `paste_delay_ms`, immediately before the first dispatch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DestinationContext {
    pub dictation_id: DictationId,
    pub destination_generation: u64,
    pub source: DestinationSource,
    /// `None` when no application can be named (foreground unreadable or Handy itself).
    pub target: Option<ResolvedTarget>,
}

/// The single per-dictation decision "insert or recover", and the only Enter veto.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDisposition {
    Pending,
    Insert,
    Recover,
    Cancelled,
}
impl RecoveryDisposition {
    /// Business rule: transitions are monotonic. `Pending` may become anything, `Insert` may
    /// only become `Recover` or `Cancelled`, and `Recover`/`Cancelled` are final; setting the
    /// current value again is a no-op.
    pub fn may_become(self, next: Self) -> bool {
        use RecoveryDisposition::{Cancelled, Insert, Pending, Recover};
        self == next
            || matches!(
                (self, next),
                (Pending, Insert | Recover | Cancelled) | (Insert, Recover | Cancelled)
            )
    }
}

/// Why 010 asks to recover instead of inserting (010 owns and extends this classification).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReason {
    /// No text field is focused in the target (010 spec reason "No text field is focused").
    NoTextField,
}

/// Answer of the optional 010 pre-insertion check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreInsertionVerdict {
    Insert,
    Recover(RecoveryReason),
}

/// Pre-insertion check registered by 010; without 010 every verdict is `Insert`.
pub trait RecoveryPort: Send + Sync {
    /// Decide, from the context about to be dispatched, whether to insert or recover.
    fn before_insertion(&self, context: &DestinationContext) -> PreInsertionVerdict;
}

/// The recovery port of a build without 010.
pub struct NoRecoveryConsumer;
impl RecoveryPort for NoRecoveryConsumer {
    fn before_insertion(&self, context: &DestinationContext) -> PreInsertionVerdict {
        // `context` is unused: without 010 there is no field classification to apply.
        let _ = context;
        PreInsertionVerdict::Insert
    }
}
