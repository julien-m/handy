//! The "no actor reply" diagnostic of `run_insertion` (N12): logged only
//! when the actor really gave no outcome, never for a delivered reply.
use super::*;
use crate::destination::workspace::Foreground;
use std::cell::RefCell;
use std::sync::mpsc;
use std::sync::OnceLock;

thread_local! {
    // Records of the current test thread only: tests run in parallel and
    // `run_insertion` logs on its caller's thread.
    static CAPTURED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

struct ThreadCapture;
impl log::Log for ThreadCapture {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        CAPTURED.with(|captured| captured.borrow_mut().push(record.args().to_string()));
    }
    fn flush(&self) {}
}

/// `run`'s result and the smart-paste messages logged on this thread
/// while it executes.
fn with_smart_paste_logs<R>(run: impl FnOnce() -> R) -> (R, Vec<String>) {
    static LOGGER: ThreadCapture = ThreadCapture;
    static INSTALLED: OnceLock<bool> = OnceLock::new();
    let is_installed = *INSTALLED.get_or_init(|| {
        let is_set = log::set_logger(&LOGGER).is_ok();
        log::set_max_level(log::LevelFilter::Debug);
        is_set
    });
    assert!(is_installed, "another test logger is installed");
    CAPTURED.with(|captured| captured.borrow_mut().clear());
    let result = run();
    let logs = CAPTURED
        .with(|captured| captured.take())
        .into_iter()
        .filter(|message| message.starts_with("[smart-paste]"))
        .collect();
    (result, logs)
}

/// Actor answering `reply`, dropping the reply channel, or stopped.
enum Actor {
    Replies(PasteOutcomeKind),
    DropsReply,
    Stopped,
}
impl InsertionSubmitter for Actor {
    fn submit(&self, request: InsertionRequest) -> Result<Receiver<PasteOutcome>, ActorStopped> {
        let (sender, receiver) = mpsc::channel();
        match self {
            Self::Stopped => return Err(ActorStopped),
            Self::DropsReply => {}
            Self::Replies(kind) => {
                let reply = PasteOutcome {
                    dictation_id: request.dictation_id,
                    transaction_id: None,
                    destination_generation: request.destination.generation,
                    path: InsertionPath::ClipboardPaste,
                    kind: *kind,
                };
                assert!(sender.send(reply).is_ok(), "receiver alive");
            }
        }
        Ok(receiver)
    }
}

struct Always;
impl RequestCheck for Always {
    fn is_current(&self, _destination: &DestinationContext) -> bool {
        true
    }
}

fn insert_with(actor: &Actor) -> (PasteOutcomeKind, Vec<String>) {
    with_smart_paste_logs(|| {
        let checks = Arc::new(DestinationChecks::default());
        let destination =
            DestinationContext::from_foreground(&Foreground::Unreadable, DestinationGeneration(4));
        let request = InsertionRequest {
            dictation_id: DictationId(3),
            transcript: Transcript::new("bonjour".to_string()),
            destination,
            clipboard_policy: ClipboardPolicy::DontModify,
            paste_delay: Duration::ZERO,
            verifies_paste: false,
        };
        run_insertion(actor, &checks, request, Arc::new(Always)).kind
    })
}

#[test]
fn a_delivered_reply_is_never_logged_as_missing() {
    let (kind, logs) = insert_with(&Actor::Replies(PasteOutcomeKind::ReadObserved));
    assert_eq!(kind, PasteOutcomeKind::ReadObserved);
    assert_eq!(logs, Vec::<String>::new());
}

#[test]
fn a_dropped_reply_is_logged_once_as_missing() {
    let (kind, logs) = insert_with(&Actor::DropsReply);
    assert_eq!(kind, PasteOutcomeKind::Uncertain);
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert!(logs[0].contains("dictation 3: no actor reply"), "{logs:?}");
}

#[test]
fn a_stopped_actor_is_logged_as_a_refusal_not_a_missing_reply() {
    let (kind, logs) = insert_with(&Actor::Stopped);
    assert_eq!(kind, PasteOutcomeKind::InsertionFailed);
    assert_eq!(logs.len(), 1, "{logs:?}");
    assert!(logs[0].contains("insertion refused"), "{logs:?}");
}
