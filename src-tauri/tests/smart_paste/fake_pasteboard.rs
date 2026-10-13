//! Scripted pasteboard: a change count, typed content and a log of writes,
//! applying the same ownership guard as the production adapter.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use handy_app_lib::smart_paste::clock::ManualClock;
use handy_app_lib::smart_paste::ids::{TransactionId, Transcript};
use handy_app_lib::smart_paste::pasteboard_port::{GuardedWrite, PasteboardError, PasteboardPort};
use handy_app_lib::smart_paste::snapshot::{
    PasteboardSnapshot, SnapshotBudget, SnapshotBuilder, PLAIN_TEXT_TYPE,
};

/// Raw items: type identifier and bytes per item.
pub type Items = Vec<Vec<(String, Vec<u8>)>>;

/// What the scripted pasteboard holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Items(Items),
    Lazy(TransactionId),
    Plain(String),
}

/// Scripted pasteboard state; every write is logged by kind.
#[derive(Debug)]
pub struct PasteboardState {
    pub change_count: i64,
    pub content: Content,
    pub capture_error: Option<PasteboardError>,
    pub capture_cost: Duration,
    pub publish_error: Option<PasteboardError>,
    pub unsavable: bool,
    pub captures: usize,
    pub writes: Vec<&'static str>,
    pub released: Vec<TransactionId>,
}

/// Scripted pasteboard shared between the coordinator and the test.
#[derive(Debug, Clone)]
pub struct FakePasteboard {
    clock: ManualClock,
    state: Arc<Mutex<PasteboardState>>,
}

impl FakePasteboard {
    /// Creates a pasteboard holding `items`.
    pub fn new(clock: &ManualClock, items: Items) -> Self {
        let state = PasteboardState {
            change_count: 100,
            content: Content::Items(items),
            capture_error: None,
            capture_cost: Duration::ZERO,
            publish_error: None,
            unsavable: false,
            captures: 0,
            writes: Vec::new(),
            released: Vec::new(),
        };
        Self {
            clock: clock.clone(),
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Locks the scripted state.
    pub fn state(&self) -> MutexGuard<'_, PasteboardState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Another application (or the user) replaces the content.
    pub fn external_write(&self, content: Content) {
        let mut state = self.state();
        state.change_count += 1;
        state.content = content;
    }

    /// Transaction of the item currently published, if any.
    pub fn published(&self) -> Option<TransactionId> {
        match self.state().content {
            Content::Lazy(transaction) => Some(transaction),
            _ => None,
        }
    }

    fn snapshot_of(state: &PasteboardState) -> Result<PasteboardSnapshot, PasteboardError> {
        let items = match &state.content {
            Content::Items(items) => items.clone(),
            Content::Plain(text) => vec![vec![(
                PLAIN_TEXT_TYPE.to_string(),
                text.as_bytes().to_vec(),
            )]],
            // Capturing Handy's own lazy item would self-trigger its provider.
            Content::Lazy(_) => return Err(PasteboardError::WriteFailed),
        };
        let inventory: Vec<Vec<String>> = items
            .iter()
            .map(|item| item.iter().map(|(uti, _)| uti.clone()).collect())
            .collect();
        let mut builder =
            SnapshotBuilder::new(SnapshotBudget::default(), state.change_count, &inventory);
        for (index, item) in items.into_iter().enumerate().filter(|_| !state.unsavable) {
            item.into_iter()
                .for_each(|(uti, data)| builder.accept(index, &uti, data));
        }
        Ok(builder.finish())
    }
}

impl PasteboardPort for FakePasteboard {
    fn change_count(&mut self) -> Result<i64, PasteboardError> {
        Ok(self.state().change_count)
    }

    fn capture_snapshot(
        &mut self,
        _budget: SnapshotBudget,
    ) -> Result<PasteboardSnapshot, PasteboardError> {
        let mut state = self.state();
        state.captures += 1;
        self.clock.advance(state.capture_cost);
        match state.capture_error {
            Some(error) => Err(error),
            None => Self::snapshot_of(&state),
        }
    }

    fn publish_lazy_transcript(
        &mut self,
        transaction: TransactionId,
        _transcript: &Transcript,
        expected_source_count: i64,
    ) -> Result<i64, PasteboardError> {
        let mut state = self.state();
        if let Some(error) = state.publish_error {
            return Err(error);
        }
        if state.change_count != expected_source_count {
            return Err(PasteboardError::SourceChanged);
        }
        state.change_count += 1;
        state.content = Content::Lazy(transaction);
        state.writes.push("publish");
        Ok(state.change_count)
    }

    fn restore_snapshot(
        &mut self,
        snapshot: Arc<PasteboardSnapshot>,
        publication_count: i64,
    ) -> Result<GuardedWrite, PasteboardError> {
        let mut state = self.state();
        if state.change_count != publication_count {
            return Ok(GuardedWrite::OwnershipChanged);
        }
        let items = snapshot
            .items()
            .iter()
            .map(|item| {
                item.types
                    .iter()
                    .map(|t| (t.uti.clone(), t.data.to_vec()))
                    .collect()
            })
            .collect();
        state.change_count += 1;
        state.content = Content::Items(items);
        state.writes.push("restore");
        Ok(GuardedWrite::Written)
    }

    fn materialize_transcript(
        &mut self,
        transcript: &Transcript,
        publication_count: i64,
    ) -> Result<GuardedWrite, PasteboardError> {
        let mut state = self.state();
        if state.change_count != publication_count {
            return Ok(GuardedWrite::OwnershipChanged);
        }
        state.change_count += 1;
        state.content = Content::Plain(transcript.as_str().to_string());
        state.writes.push("materialize");
        Ok(GuardedWrite::Written)
    }

    fn write_plain_text(&mut self, transcript: &Transcript) -> Result<(), PasteboardError> {
        let mut state = self.state();
        state.change_count += 1;
        state.content = Content::Plain(transcript.as_str().to_string());
        state.writes.push("plain");
        Ok(())
    }

    fn release_transaction(&mut self, transaction: TransactionId) {
        self.state().released.push(transaction);
    }
}
