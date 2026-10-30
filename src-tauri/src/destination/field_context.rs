//! What the insertion's final preparation (015 smart capitalization, 017 smart spacing) may
//! read of a dictation's destination before its insertion: the application the text goes to,
//! and the input 011 remembered for it with its remembered selection. The remembered element is
//! read outside the manager lock (bounded by the caller's deadline) and put back while its
//! dictation is current; nothing read is kept or logged.
use super::context::DictationId;
use super::field::{FieldNode, RememberedField, SelectionRange};
use super::manager::DestinationManager;
use super::session::Selection;

/// Where the text of a dictation goes, as far as its context is concerned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsertionTarget {
    /// The application in front when the text is inserted (no application destination).
    Frontmost,
    /// A chosen or captured application instance.
    Application { pid: u32 },
    /// The dictation is no longer current (cancelled, superseded): nothing is read.
    Stale,
}

impl DestinationManager {
    /// The insertion target of dictation `dictation_id` at `generation`.
    pub fn insertion_target(&self, dictation_id: DictationId, generation: u64) -> InsertionTarget {
        let state = self.lock();
        let Some(session) = state
            .session
            .as_ref()
            .filter(|session| session.is(dictation_id, generation))
        else {
            return InsertionTarget::Stale;
        };
        match &session.selection {
            Selection::Application(destination) => InsertionTarget::Application {
                pid: destination.captured_pid,
            },
            Selection::FrontmostAtEnd => InsertionTarget::Frontmost,
        }
    }

    // @spec AC-007: Context of 011's remembered caret — .specs/features/015-smart-capitalization/spec.md#ac-007
    /// Run `read` on the input 011 remembered for this dictation, with its remembered
    /// selection (start and length, `None` without a readable offsets range); `None` when no
    /// input is remembered for it. `read` must bound its own accessibility queries.
    ///
    /// Order matters: the input is moved out under the lock, read without it (an accessibility
    /// round trip must never stall the manager), then put back only while its dictation is
    /// still current and no other input took its place; otherwise it is released here.
    pub fn with_remembered_input<R>(
        &self,
        (dictation_id, generation): (DictationId, u64),
        read: impl FnOnce(&mut dyn FieldNode, Option<(usize, usize)>) -> R,
    ) -> Option<R> {
        let mut field = self.take_remembered(dictation_id, generation)?;
        let range = field.range.as_ref().and_then(SelectionRange::as_offsets);
        let answer = read(field.element.as_mut(), range);
        self.put_back(field);
        Some(answer)
    }

    /// The input remembered for this dictation, moved out of the manager.
    fn take_remembered(
        &self,
        dictation_id: DictationId,
        generation: u64,
    ) -> Option<RememberedField> {
        let mut state = self.lock();
        let own = state.field.as_ref().is_some_and(|field| {
            field.dictation_id == dictation_id && field.destination_generation == generation
        });
        if own {
            state.field.take()
        } else {
            None
        }
    }

    /// Put `field` back while its dictation is current and no other input was remembered.
    fn put_back(&self, field: RememberedField) {
        let mut state = self.lock();
        // A badge choice, pin or unpin during the read changes the selection without a new
        // generation and found no input to release (011 AC-003): `serves` releases it here.
        let current = state.field.is_none()
            && state.generation == field.destination_generation
            && state.session.as_ref().is_some_and(|session| {
                session.is(field.dictation_id, field.destination_generation)
                    && field.serves(&session.selection)
            });
        if current {
            state.field = Some(field);
        }
    }
}
