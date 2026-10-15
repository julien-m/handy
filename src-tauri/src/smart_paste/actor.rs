//! The single thread that owns the coordinator.
//!
//! Insertions, served reads (from the pasteboard provider on the main
//! thread), timers and shutdown all go through one channel, so clipboard
//! ownership, chords and input-source cleanup are serialized and the UI
//! thread never waits on smart paste.
//!
//! Ordering rules: reads are handled before a timer that they could move
//! (a read stamped before a deadline but delivered just after it is awaited
//! for [`READ_DELIVERY_GRACE`]); a due timer before queued requests; a
//! shutdown before queued insertions, which are then dropped unrun. An
//! insertion in progress always finishes first, so its outcome stays
//! truthful: [`shutdown_allowance`] bounds that wait.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::clock::Clock;
use super::coordinator::{SmartPasteCoordinator, SmartPastePorts};
use super::direct::TOTAL_DIRECT_BUDGET;
use super::ids::Transcript;
use super::key_events::CHORD_HOLD;
use super::keyboard::{LAYOUT_RESTORE_DELAY, LAYOUT_SETTLE_DELAY};
use super::outcome::PasteOutcome;
use super::pasteboard_port::{PasteboardError, ReadEvent, ReadSink};
use super::request::{InsertionRequest, ReadEvents};
use super::snapshot::SNAPSHOT_DEADLINE;
use super::timing::JUMP_NO_READ_DECISION;

// @spec FR-008: Serialized delivery of terminal outcomes — .specs/features/007-smart-paste/spec.md#fr-008

/// How long a read stamped before a deadline may still take to reach the
/// actor after that deadline. The provider stamps and sends back to back on
/// the main thread, so only a scheduler preemption delays delivery; 20 ms
/// covers that without visibly delaying a "not read" decision.
pub const READ_DELIVERY_GRACE: Duration = Duration::from_millis(20);

/// One main-thread write (settlement on exit) is bounded by the adapters'
/// one-second dispatch timeout.
const SETTLEMENT_ALLOWANCE: Duration = Duration::from_secs(1);

/// Longest a normal exit may wait in [`SmartPasteActor::shutdown`] with a
/// responsive main thread: an insertion already running finishes first
/// (direct insertion, snapshot, layout settle, `paste_delay`, chord hold,
/// layout restore, the longest no-read decision and the read grace), then
/// the pending transaction is settled with one main-thread write.
pub fn shutdown_allowance(paste_delay: Duration) -> Duration {
    TOTAL_DIRECT_BUDGET
        + SNAPSHOT_DEADLINE
        + LAYOUT_SETTLE_DELAY
        + paste_delay
        + CHORD_HOLD
        + LAYOUT_RESTORE_DELAY
        + JUMP_NO_READ_DECISION
        + READ_DELIVERY_GRACE
        + SETTLEMENT_ALLOWANCE
}

enum ActorMessage {
    Insert(Box<InsertionRequest>, Sender<PasteOutcome>),
    Read(ReadEvent),
    /// 010's deliberate Copy, serialized with insertions, reads, restoration and timers.
    DeliberateCopy(Transcript, Sender<Result<(), PasteboardError>>),
    Shutdown(Sender<()>),
}

/// The actor thread has stopped; no request can be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorStopped;

/// Why [`SmartPasteActor::shutdown`] did not complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownError {
    /// Called on the macOS main thread: the settlement write needs that
    /// thread, so it would wait for itself. Call from a worker while the
    /// main run loop keeps running (for example: prevent the exit, shut down
    /// from a worker, then exit).
    OnMainThread,
    /// The actor had already stopped.
    Stopped,
    /// The actor did not finish within the timeout; it keeps running and
    /// settles on its own.
    TimedOut,
}

/// Handle to the smart-paste actor thread. Dropping it without
/// [`Self::shutdown`] still asks the thread to settle and stop, without
/// waiting for it.
pub struct SmartPasteActor {
    sender: Sender<ActorMessage>,
    thread: Option<JoinHandle<()>>,
}

impl SmartPasteActor {
    /// Starts the actor. `build_ports` receives the read sink that published
    /// items must report to and runs on the actor thread.
    ///
    /// # Errors
    /// Returns the OS error if the thread cannot be spawned.
    pub fn spawn(
        build_ports: impl FnOnce(ReadSink) -> SmartPastePorts + Send + 'static,
    ) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let read_sender = sender.clone();
        let sink: ReadSink = Arc::new(move |event| {
            // A stopped actor no longer settles anything; the read is moot.
            let _ = read_sender.send(ActorMessage::Read(event));
        });
        let thread = std::thread::Builder::new()
            .name("smart-paste".into())
            .spawn(move || run(build_ports(sink), receiver))?;
        Ok(Self {
            sender,
            thread: Some(thread),
        })
    }

    /// Queues an insertion; the terminal outcome arrives on the receiver.
    /// An insertion still queued when the actor shuts down is dropped and
    /// its receiver disconnects without an outcome.
    ///
    /// # Errors
    /// [`ActorStopped`] once the actor has shut down.
    pub fn insert(
        &self,
        request: InsertionRequest,
    ) -> Result<Receiver<PasteOutcome>, ActorStopped> {
        let (reply, outcome) = mpsc::channel();
        self.sender
            .send(ActorMessage::Insert(Box::new(request), reply))
            .map_err(|_| ActorStopped)?;
        Ok(outcome)
    }

    // @spec AC-005: Deliberate Copy dominates restoration — .specs/features/010-recovery-bubble/spec.md#ac-005
    /// Queues 010's deliberate Copy of `transcript` as unmarked plain text, behind an
    /// insertion in progress. A successful write changes the `changeCount`, so a pending
    /// restoration abandons through its own guard. The result arrives on the receiver; a
    /// Copy still queued at shutdown is dropped and its receiver disconnects.
    ///
    /// # Errors
    /// [`ActorStopped`] once the actor has shut down.
    pub fn deliberate_copy(
        &self,
        transcript: Transcript,
    ) -> Result<Receiver<Result<(), PasteboardError>>, ActorStopped> {
        let (reply, written) = mpsc::channel();
        self.sender
            .send(ActorMessage::DeliberateCopy(transcript, reply))
            .map_err(|_| ActorStopped)?;
        Ok(written)
    }

    /// Settles the pending transaction for a normal exit and stops the
    /// thread, waiting up to `timeout` (see [`shutdown_allowance`]). Must not
    /// be called on the macOS main thread.
    ///
    /// # Errors
    /// See [`ShutdownError`].
    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), ShutdownError> {
        #[cfg(target_os = "macos")]
        if super::main_thread::is_main_thread() {
            return Err(ShutdownError::OnMainThread);
        }
        let (reply, done) = mpsc::channel();
        self.sender
            .send(ActorMessage::Shutdown(reply))
            .map_err(|_| ShutdownError::Stopped)?;
        done.recv_timeout(timeout).map_err(|error| match error {
            RecvTimeoutError::Timeout => ShutdownError::TimedOut,
            RecvTimeoutError::Disconnected => ShutdownError::Stopped,
        })?;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        Ok(())
    }
}

impl Drop for SmartPasteActor {
    fn drop(&mut self) {
        // Not joined: dropping may happen on the main thread, which the
        // settlement write needs. The thread settles and exits on its own.
        if self.thread.take().is_some() {
            let (reply, _) = mpsc::channel();
            let _ = self.sender.send(ActorMessage::Shutdown(reply));
        }
    }
}

enum Next {
    Message(ActorMessage),
    Deadline,
    Disconnected,
}

struct ActorQueue {
    receiver: Receiver<ActorMessage>,
    deferred: VecDeque<ActorMessage>,
    clock: Arc<dyn Clock>,
    read_grace: Duration,
}

impl ActorQueue {
    fn new(receiver: Receiver<ActorMessage>, clock: Arc<dyn Clock>) -> Self {
        Self {
            receiver,
            deferred: VecDeque::new(),
            clock,
            read_grace: READ_DELIVERY_GRACE,
        }
    }

    /// Next message or timer for the idle actor, in the module's order.
    fn next(&mut self, deadline: Option<Instant>) -> Next {
        while let Ok(message) = self.receiver.try_recv() {
            self.deferred.push_back(message);
        }
        if let Some(read) = self.take_read_by(None) {
            return Next::Message(ActorMessage::Read(read));
        }
        if let Some(due) = deadline.filter(|due| self.clock.now() >= *due) {
            return self.at_deadline(due);
        }
        if let Some(message) = self.take_shutdown_or_first() {
            return Next::Message(message);
        }
        self.wait(deadline)
    }

    fn wait(&mut self, deadline: Option<Instant>) -> Next {
        let Some(due) = deadline else {
            return self
                .receiver
                .recv()
                .map_or(Next::Disconnected, Next::Message);
        };
        let remaining = due.saturating_duration_since(self.clock.now());
        match self.receiver.recv_timeout(remaining) {
            Ok(message) => Next::Message(message),
            Err(RecvTimeoutError::Timeout) => self.at_deadline(due),
            // Unreachable while a published item's read sink holds a sender.
            Err(RecvTimeoutError::Disconnected) => Next::Disconnected,
        }
    }

    fn at_deadline(&mut self, due: Instant) -> Next {
        match self.late_read_by(due) {
            Some(read) => Next::Message(ActorMessage::Read(read)),
            None => Next::Deadline,
        }
    }

    /// Removes the first deferred read stamped at or before `limit` (any
    /// read when `limit` is `None`).
    fn take_read_by(&mut self, limit: Option<Instant>) -> Option<ReadEvent> {
        let position = self.deferred.iter().position(|message| {
            matches!(message, ActorMessage::Read(event) if limit.is_none_or(|limit| event.at <= limit))
        })?;
        match self.deferred.remove(position) {
            Some(ActorMessage::Read(event)) => Some(event),
            _ => None,
        }
    }

    fn take_shutdown_or_first(&mut self) -> Option<ActorMessage> {
        let shutdown = self
            .deferred
            .iter()
            .position(|message| matches!(message, ActorMessage::Shutdown(_)));
        self.deferred.remove(shutdown.unwrap_or(0))
    }

    /// After the timer at `deadline` fired: a read stamped by then may still
    /// be in flight, so it is awaited for the grace period before the timer
    /// is applied. Other messages are kept for later, in order.
    fn late_read_by(&mut self, deadline: Instant) -> Option<ReadEvent> {
        if let Some(read) = self.take_read_by(Some(deadline)) {
            return Some(read);
        }
        let grace_end = self.clock.now() + self.read_grace;
        loop {
            let remaining = grace_end.saturating_duration_since(self.clock.now());
            match self.receiver.recv_timeout(remaining) {
                Ok(ActorMessage::Read(event)) if event.at <= deadline => return Some(event),
                Ok(other) => self.deferred.push_back(other),
                Err(_) => return None,
            }
        }
    }
}

impl ReadEvents for ActorQueue {
    fn next_read(&mut self, deadline: Instant) -> Option<ReadEvent> {
        // Commands and reads stamped after the deadline are kept, in order,
        // for after the current insertion.
        if let Some(read) = self.take_read_by(Some(deadline)) {
            return Some(read);
        }
        loop {
            let remaining = deadline.saturating_duration_since(self.clock.now());
            match self.receiver.recv_timeout(remaining) {
                Ok(ActorMessage::Read(event)) if event.at <= deadline => return Some(event),
                Ok(other) => self.deferred.push_back(other),
                Err(RecvTimeoutError::Timeout) => return self.late_read_by(deadline),
                Err(RecvTimeoutError::Disconnected) => return None,
            }
        }
    }
}

fn run(ports: SmartPastePorts, receiver: Receiver<ActorMessage>) {
    let mut queue = ActorQueue::new(receiver, Arc::clone(&ports.clock));
    let mut coordinator = SmartPasteCoordinator::new(ports);
    // Serves one message or timer at a time until shutdown (or until every
    // sender is gone, which settles like a normal exit).
    loop {
        match queue.next(coordinator.next_deadline()) {
            Next::Message(ActorMessage::Insert(request, reply)) => {
                let outcome = coordinator.insert(&request, &mut queue);
                // The requester may have given up waiting; the outcome stands.
                let _ = reply.send(outcome);
            }
            Next::Message(ActorMessage::Read(event)) => coordinator.handle_read(event),
            Next::Message(ActorMessage::DeliberateCopy(transcript, reply)) => {
                // The requester may have given up waiting; the write stands.
                let _ = reply.send(coordinator.deliberate_copy(&transcript));
            }
            Next::Message(ActorMessage::Shutdown(reply)) => {
                coordinator.shutdown();
                let _ = reply.send(());
                return;
            }
            Next::Deadline => coordinator.handle_deadline(),
            Next::Disconnected => return coordinator.shutdown(),
        }
    }
}

#[cfg(test)]
#[path = "actor_tests.rs"]
mod tests;
