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
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime};

use super::clock::Clock;
use super::snapshot::{PasteboardSnapshot, SnapshotBudget, SnapshotBuilder, SnapshotError};
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
        .stdin(Stdio::null())
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

/// Captures a bounded snapshot through the helper process.
///
/// # Errors
/// See [`SnapshotError`]; on any error the caller must not publish.
pub fn capture_with_helper(
    command: &HelperCommand,
    budget: SnapshotBudget,
    clock: &dyn Clock,
) -> Result<PasteboardSnapshot, SnapshotError> {
    let deadline = clock.now() + budget.deadline;
    let nonce = new_nonce();
    let mut child = spawn_helper(command, &nonce, budget)?;
    let stdout = child.stdout.take().ok_or(SnapshotError::Spawn)?;
    let (reader, frames) = spawn_reader(stdout, nonce, budget.max_bytes);
    let _process = HelperProcess {
        child,
        reader: Some(reader),
    };
    collect_frames(&frames, clock, deadline, budget)
}

struct Collector {
    budget: SnapshotBudget,
    begin: Option<(i64, Vec<Vec<String>>, SnapshotBuilder)>,
}

impl Collector {
    fn apply(&mut self, frame: Frame) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        match frame {
            Frame::Begin {
                change_count,
                inventory,
            } if self.begin.is_none() => {
                let builder = SnapshotBuilder::new(self.budget, change_count, &inventory);
                self.begin = Some((change_count, inventory, builder));
                Ok(None)
            }
            Frame::Data {
                item,
                type_index,
                data,
            } => self.with_type(item, type_index, |builder, uti| {
                builder.accept(item, uti, data)
            }),
            Frame::Skipped {
                item,
                type_index,
                cause,
            } => self.with_type(item, type_index, |builder, uti| {
                builder.skip(item, uti, cause)
            }),
            Frame::End { change_count } if self.begin.is_some() => {
                self.finish_complete(change_count)
            }
            _ => Err(SnapshotError::Protocol(ProtocolError::Malformed)),
        }
    }

    /// Applies `update` to the inventoried type; frames before the
    /// inventory are malformed, unknown indexes are ignored.
    fn with_type(
        &mut self,
        item: usize,
        type_index: usize,
        update: impl FnOnce(&mut SnapshotBuilder, &str),
    ) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        let (_, inventory, builder) = self
            .begin
            .as_mut()
            .ok_or(SnapshotError::Protocol(ProtocolError::Malformed))?;
        if let Some(uti) = inventory.get(item).and_then(|types| types.get(type_index)) {
            update(builder, uti);
        }
        Ok(None)
    }

    fn finish_complete(
        &mut self,
        end_count: i64,
    ) -> Result<Option<PasteboardSnapshot>, SnapshotError> {
        let (begin_count, _, builder) = self.begin.take().ok_or(SnapshotError::NoInventory)?;
        if begin_count != end_count {
            return Err(SnapshotError::SourceChanged);
        }
        Ok(Some(builder.finish()))
    }

    fn finish_at_deadline(self) -> Result<PasteboardSnapshot, SnapshotError> {
        // Types not received by the deadline are recorded as timed out.
        self.begin
            .map(|(_, _, builder)| builder.finish())
            .ok_or(SnapshotError::NoInventory)
    }
}

fn collect_frames(
    frames: &Receiver<Result<Frame, ProtocolError>>,
    clock: &dyn Clock,
    deadline: Instant,
    budget: SnapshotBudget,
) -> Result<PasteboardSnapshot, SnapshotError> {
    let mut collector = Collector {
        budget,
        begin: None,
    };
    // Applies frames until the end frame (complete snapshot), a violation
    // or an early end of stream (rejected), or the deadline (partial).
    loop {
        let remaining = deadline.saturating_duration_since(clock.now());
        match frames.recv_timeout(remaining) {
            Ok(Ok(frame)) => {
                if let Some(snapshot) = collector.apply(frame)? {
                    return Ok(snapshot);
                }
            }
            Ok(Err(error)) => return Err(SnapshotError::Protocol(error)),
            Err(RecvTimeoutError::Timeout) => return collector.finish_at_deadline(),
            Err(RecvTimeoutError::Disconnected) => {
                return Err(SnapshotError::Protocol(ProtocolError::Io(
                    std::io::ErrorKind::UnexpectedEof,
                )))
            }
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
    match super::macos_snapshot_stream::write_snapshot_stream(&mut writer, &nonce, max_bytes, name)
    {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_round_trips_through_its_hex_argument() {
        let nonce = new_nonce();
        assert_eq!(parse_nonce(&hex(&nonce)), Some(nonce));
        assert_eq!(parse_nonce("zz"), None);
    }

    #[test]
    fn two_requests_get_different_nonces() {
        assert_ne!(new_nonce(), new_nonce());
    }
}
