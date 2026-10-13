//! The single thread that owns the coordinator.
//!
//! Insertions, served reads (from the pasteboard provider on the main
//! thread), timers and shutdown all go through one channel, so clipboard
//! ownership, chords and input-source cleanup are serialized and the UI
//! thread never waits on smart paste.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::clock::Clock;
use super::coordinator::{SmartPasteCoordinator, SmartPastePorts};
use super::outcome::PasteOutcome;
use super::pasteboard_port::{ReadEvent, ReadSink};
use super::request::{InsertionRequest, ReadEvents};

// @spec FR-008: Serialized delivery of terminal outcomes — .specs/features/007-smart-paste/spec.md#fr-008

enum ActorMessage {
    Insert(Box<InsertionRequest>, Sender<PasteOutcome>),
    Read(ReadEvent),
    Shutdown(Sender<()>),
}

/// The actor thread has stopped; no request can be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorStopped;

/// Handle to the smart-paste actor thread.
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

    /// Settles the pending transaction for a normal exit and stops the
    /// thread. Returns false if the actor did not finish within `timeout`.
    pub fn shutdown(&mut self, timeout: Duration) -> bool {
        let (reply, done) = mpsc::channel();
        if self.sender.send(ActorMessage::Shutdown(reply)).is_err() {
            return false;
        }
        let is_done = done.recv_timeout(timeout).is_ok();
        if is_done {
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
        is_done
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
}

impl ActorQueue {
    fn next(&mut self, deadline: Option<Instant>) -> Next {
        if let Some(message) = self.deferred.pop_front() {
            return Next::Message(message);
        }
        let received = match deadline {
            None => self
                .receiver
                .recv()
                .map_err(|_| RecvTimeoutError::Disconnected),
            Some(deadline) => {
                let remaining = deadline.saturating_duration_since(self.clock.now());
                self.receiver.recv_timeout(remaining)
            }
        };
        match received {
            Ok(message) => Next::Message(message),
            Err(RecvTimeoutError::Timeout) => Next::Deadline,
            Err(RecvTimeoutError::Disconnected) => Next::Disconnected,
        }
    }
}

impl ReadEvents for ActorQueue {
    fn next_read(&mut self, deadline: Instant) -> Option<ReadEvent> {
        // Commands and reads stamped after the deadline are kept, in order,
        // for after the current insertion.
        loop {
            let remaining = deadline.saturating_duration_since(self.clock.now());
            match self.receiver.recv_timeout(remaining) {
                Ok(ActorMessage::Read(event)) if event.at <= deadline => return Some(event),
                Ok(other) => self.deferred.push_back(other),
                Err(_) => return None,
            }
        }
    }
}

fn run(ports: SmartPastePorts, receiver: Receiver<ActorMessage>) {
    let clock = Arc::clone(&ports.clock);
    let mut coordinator = SmartPasteCoordinator::new(ports);
    let mut queue = ActorQueue {
        receiver,
        deferred: VecDeque::new(),
        clock,
    };
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
            Next::Message(ActorMessage::Shutdown(reply)) => {
                coordinator.shutdown();
                let _ = reply.send(());
                return;
            }
            Next::Deadline => coordinator.handle_deadline(),
            Next::Disconnected => {
                coordinator.shutdown();
                return;
            }
        }
    }
}
