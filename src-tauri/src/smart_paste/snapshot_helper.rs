//! Read-only snapshot helper: the Handy executable re-launched with
//! [`HELPER_FLAG`] reads the pasteboard and streams it back (see
//! `snapshot_protocol`).
//!
//! Reading another application's promised data can block for as long as
//! that application takes. Running the read in a child process lets the
//! parent enforce the 200 ms / 50 MB bounds by killing and reaping it, keep
//! every type completed before the deadline, and never let a late reply
//! publish or restore anything. The helper never writes the pasteboard and
//! nothing it reads touches the disk or a log.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime};

use super::clock::Clock;
use super::macos_snapshot_stream::START_TRIGGER;
use super::snapshot::{
    OmittedTypes, PasteboardSnapshot, SkipCause, SnapshotBudget, SnapshotBuilder, SnapshotError,
};
use super::snapshot_protocol::{read_frame, read_header, Frame, Nonce, ProtocolError, NONCE_LEN};

/// Hidden first argument that turns the executable into the helper.
pub const HELPER_FLAG: &str = "--smart-paste-snapshot";

/// How to launch the helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperCommand {
    /// Executable to launch (the running Handy binary in production).
    pub executable: PathBuf,
    /// Named pasteboard to read; `None` reads the general pasteboard.
    pub pasteboard_name: Option<String>,
}

/// Owned helper process: killed and reaped on drop, then its reader joined.
struct HelperProcess {
    child: Child,
    reader: Option<JoinHandle<()>>,
}

impl Drop for HelperProcess {
    fn drop(&mut self) {
        // The helper may already have exited; kill then reaps either way.
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Killing closed the pipe's only writer, so the reader ends promptly.
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn new_nonce() -> Nonce {
    let mut nonce = [0_u8; NONCE_LEN];
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    // RandomState is seeded from OS randomness; the nonce only correlates a
    // response with its request, it is not a cryptographic secret.
    for (index, chunk) in nonce.chunks_mut(8).enumerate() {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_usize(index);
        hasher.write_u128(nanos);
        chunk.copy_from_slice(&hasher.finish().to_le_bytes());
    }
    nonce
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Starts the helper without a shell: argv is the hidden flag, the nonce,
/// the byte budget and the optional pasteboard name. stdin carries the start
/// trigger; stderr is discarded (the helper never prints content there).
fn spawn_helper(
    command: &HelperCommand,
    nonce: &Nonce,
    budget: SnapshotBudget,
) -> Result<Child, SnapshotError> {
    let mut process = Command::new(&command.executable);
    process
        .arg(HELPER_FLAG)
        .arg(hex(nonce))
        .arg(budget.max_bytes.to_string())
        .args(command.pasteboard_name.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    process.spawn().map_err(|_| SnapshotError::Spawn)
}

type FrameResult = Result<Frame, ProtocolError>;

fn spawn_reader(
    stdout: ChildStdout,
    nonce: Nonce,
    max_bytes: usize,
) -> (JoinHandle<()>, Receiver<FrameResult>) {
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut stream = BufReader::new(stdout);
        match read_header(&mut stream, &nonce) {
            Ok(()) => forward_frames(&mut stream, &sender, max_bytes),
            Err(error) => {
                let _ = sender.send(Err(error));
            }
        }
    });
    (reader, receiver)
}

/// Forwards frames until end of stream or the first violation; the byte
/// budget is tracked here so no frame can exceed what is left of it.
fn forward_frames(
    stream: &mut impl std::io::Read,
    sender: &mpsc::Sender<FrameResult>,
    max_bytes: usize,
) {
    let mut remaining = max_bytes;
    loop {
        let frame = match read_frame(stream, remaining) {
            Ok(Some(frame)) => frame,
            Ok(None) => return,
            Err(error) => {
                let _ = sender.send(Err(error));
                return;
            }
        };
        if let Frame::Data { data, .. } = &frame {
            remaining -= data.len();
        }
        // A closed receiver means the parent stopped listening (deadline).
        if sender.send(Ok(frame)).is_err() {
            return;
        }
    }
}

/// A helper started ahead of time. It has (or will have) written its header
/// and waits for the start trigger before reading the pasteboard, so its
/// process startup is not paid inside the capture deadline. Dropping it
/// kills and reaps the process without any pasteboard read.
pub struct ArmedHelper {
    process: HelperProcess,
    stdin: Option<ChildStdin>,
    frames: Receiver<FrameResult>,
    budget: SnapshotBudget,
}

impl ArmedHelper {
    /// Starts a helper that waits for [`Self::capture`].
    ///
    /// # Errors
    /// [`SnapshotError::Spawn`] if the process or its pipes cannot be set up.
    pub fn arm(command: &HelperCommand, budget: SnapshotBudget) -> Result<Self, SnapshotError> {
        let nonce = new_nonce();
        let mut process = HelperProcess {
            child: spawn_helper(command, &nonce, budget)?,
            reader: None,
        };
        // On a missing pipe, `process` is dropped here: killed and reaped.
        let stdin = process.child.stdin.take().ok_or(SnapshotError::Spawn)?;
        let stdout = process.child.stdout.take().ok_or(SnapshotError::Spawn)?;
        let (reader, frames) = spawn_reader(stdout, nonce, budget.max_bytes);
        process.reader = Some(reader);
        Ok(Self {
            process,
            stdin: Some(stdin),
            frames,
            budget,
        })
    }

    /// Budget the helper was started with.
    pub fn budget(&self) -> SnapshotBudget {
        self.budget
    }

    /// Whether the process is still waiting (not exited or killed).
    pub fn is_waiting(&mut self) -> bool {
        matches!(self.process.child.try_wait(), Ok(None))
    }

    /// Triggers the read and collects the snapshot within the budget
    /// deadline, measured from now.
    ///
    /// # Errors
    /// See [`SnapshotError`]; on any error the caller must not publish.
    pub fn capture(self, clock: &dyn Clock) -> Result<PasteboardSnapshot, SnapshotError> {
        let deadline = clock.now() + self.budget.deadline;
        self.capture_until(clock, deadline)
    }

    fn capture_until(
        mut self,
        clock: &dyn Clock,
        deadline: Instant,
    ) -> Result<PasteboardSnapshot, SnapshotError> {
        let mut stdin = self.stdin.take().ok_or(SnapshotError::Spawn)?;
        stdin
            .write_all(&[START_TRIGGER])
            .and_then(|()| stdin.flush())
            .map_err(|_| SnapshotError::Spawn)?;
        collect_frames(&self.frames, clock, deadline, self.budget)
    }
}

/// Captures a bounded snapshot through a freshly started helper; the
/// deadline includes the helper startup.
///
/// # Errors
/// See [`SnapshotError`]; on any error the caller must not publish.
pub fn capture_with_helper(
    command: &HelperCommand,
    budget: SnapshotBudget,
    clock: &dyn Clock,
) -> Result<PasteboardSnapshot, SnapshotError> {
    let deadline = clock.now() + budget.deadline;
    ArmedHelper::arm(command, budget)?.capture_until(clock, deadline)
}

/// Source `changeCount` at the inventory and the snapshot being built.
type Inventoried = (i64, SnapshotBuilder);

struct Collector {
    budget: SnapshotBudget,
    begin: Option<Inventoried>,
}

impl Collector {
    fn apply(&mut self, frame: Frame) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        match frame {
            Frame::Begin {
                change_count,
                inventory,
            } if self.begin.is_none() => {
                let builder = SnapshotBuilder::new(self.budget, change_count, &inventory);
                self.begin = Some((change_count, builder));
                Ok(None)
            }
            Frame::Omitted { items, types } => self.with_builder(|builder| {
                builder.record_omitted(OmittedTypes { items, types });
            }),
            Frame::Data {
                item,
                type_index,
                data,
            } => self.with_builder(|builder| builder.accept(item, type_index, data)),
            Frame::Skipped {
                item,
                type_index,
                cause,
            } => self.with_builder(|builder| builder.skip(item, type_index, cause)),
            Frame::End { change_count } if self.begin.is_some() => {
                self.finish_complete(change_count)
            }
            // A second inventory or an end before the inventory.
            _ => Err(SnapshotError::Protocol(ProtocolError::Malformed)),
        }
    }

    /// Applies `update` once the inventory is known; frames before it are
    /// malformed. Unknown indexes are ignored by the builder.
    fn with_builder(
        &mut self,
        update: impl FnOnce(&mut SnapshotBuilder),
    ) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        let (_, builder) = self
            .begin
            .as_mut()
            .ok_or(SnapshotError::Protocol(ProtocolError::Malformed))?;
        update(builder);
        Ok(None)
    }

    fn finish_complete(
        &mut self,
        end_count: i64,
    ) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        let (begin_count, builder) = self.begin.take().ok_or(SnapshotError::NoInventory)?;
        if begin_count != end_count {
            return Err(SnapshotError::SourceChanged);
        }
        Ok(Some(builder.finish()))
    }

    /// Partial snapshot when the stream stops early: types not received by
    /// the deadline time out; after the helper stopped they are unreadable.
    /// Either way the completed types are kept, as at the deadline.
    fn finish_partial(self, missing: SkipCause) -> Result<PasteboardSnapshot, SnapshotError> {
        self.begin
            .map(|(_, builder)| builder.finish_with_missing_as(missing))
            .ok_or(SnapshotError::NoInventory)
    }
}

/// Whether `error` only means the stream ended early (helper stopped).
fn is_early_end(error: &ProtocolError) -> bool {
    *error == ProtocolError::Io(std::io::ErrorKind::UnexpectedEof)
}

fn collect_frames(
    frames: &Receiver<FrameResult>,
    clock: &dyn Clock,
    deadline: Instant,
    budget: SnapshotBudget,
) -> Result<PasteboardSnapshot, SnapshotError> {
    let mut collector = Collector {
        budget,
        begin: None,
    };
    // Applies frames until the end frame (complete snapshot), a violation
    // (rejected), or the deadline or an early end of the stream (partial).
    loop {
        let remaining = deadline.saturating_duration_since(clock.now());
        let frame = match frames.recv_timeout(remaining) {
            Ok(Ok(frame)) => frame,
            Ok(Err(error)) if !is_early_end(&error) => return Err(SnapshotError::Protocol(error)),
            Err(RecvTimeoutError::Timeout) => return collector.finish_partial(SkipCause::Timeout),
            Ok(Err(_)) | Err(RecvTimeoutError::Disconnected) => {
                return collector.finish_partial(SkipCause::Unreadable)
            }
        };
        if let Some(snapshot) = collector.apply(frame)? {
            return Ok(snapshot);
        }
    }
}

/// Whether this process was launched as the snapshot helper. Checked in
/// `main` before CLI parsing, Tauri or single-instance handling.
pub fn is_helper_invocation() -> bool {
    std::env::args_os()
        .nth(1)
        .is_some_and(|argument| argument == HELPER_FLAG)
}

fn parse_nonce(text: &str) -> Option<Nonce> {
    let mut nonce = [0_u8; NONCE_LEN];
    if text.len() != NONCE_LEN * 2 {
        return None;
    }
    for (index, byte) in nonce.iter_mut().enumerate() {
        *byte = u8::from_str_radix(text.get(index * 2..index * 2 + 2)?, 16).ok()?;
    }
    Some(nonce)
}

/// Runs the helper: streams a snapshot to stdout and returns the exit code
/// (0 success, 2 bad arguments, 1 write failure).
pub fn run_helper() -> i32 {
    let arguments: Vec<String> = std::env::args().skip(2).collect();
    let parsed = arguments
        .first()
        .and_then(|nonce| parse_nonce(nonce))
        .zip(arguments.get(1).and_then(|max| max.parse::<usize>().ok()));
    let Some((nonce, max_bytes)) = parsed else {
        return 2;
    };
    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let name = arguments.get(2).map(String::as_str);
    let written = super::macos_snapshot_stream::write_snapshot_stream(
        &mut writer,
        super::macos_snapshot_stream::wait_for_trigger,
        &nonce,
        max_bytes,
        name,
    );
    match written {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
#[path = "snapshot_helper_tests.rs"]
mod tests;
