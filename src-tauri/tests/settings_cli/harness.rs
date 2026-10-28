//! Isolated `handy` binary for process tests: a hard link (or copy) beside a portable marker,
//! so the production locator resolves a private `Data/settings_store.json`, never the user store.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, OnceLock};

static SPAWN: Mutex<()> = Mutex::new(());
static WARM_UP: OnceLock<bool> = OnceLock::new();

pub type TestResult = Result<(), Box<dyn std::error::Error>>;
pub type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

/// One portable installation of the binary under test with its own store.
pub struct Portable {
    pub root: tempfile::TempDir,
    binary: PathBuf,
    legacy_socket: PathBuf,
}
/// Exit code, stdout and stderr of one invocation.
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}
impl Run {
    fn from(output: Output) -> Self {
        use std::os::unix::process::ExitStatusExt;
        Self {
            // A signal is reported as its negated number, so a crash never looks like an exit.
            code: output
                .status
                .code()
                .unwrap_or_else(|| -output.status.signal().unwrap_or(1)),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
    /// The single JSON envelope printed on stdout (exactly one line).
    pub fn envelope(&self) -> Fallible<Value> {
        let lines = self.stdout.lines().collect::<Vec<_>>();
        if lines.len() != 1 {
            return Err(format!("expected one stdout line, got {lines:?}").into());
        }
        Ok(serde_json::from_str(lines[0])?)
    }
}
impl Portable {
    /// Short temporary root under /tmp so private socket paths stay within platform limits.
    pub fn new() -> Fallible<Self> {
        let root = tempfile::Builder::new()
            .prefix("hs")
            .tempdir_in(short_temp())?;
        let directory = root.path().join("app");
        std::fs::create_dir_all(directory.join("Data"))?;
        std::fs::write(directory.join("portable"), "Handy Portable Mode\n")?;
        let source = Path::new(env!("CARGO_BIN_EXE_handy"));
        let binary = directory.join(source.file_name().ok_or("binary has no file name")?);
        if std::fs::hard_link(source, &binary).is_err() {
            std::fs::copy(source, &binary)?;
        }
        let legacy_socket = root.path().join("legacy.sock");
        Ok(Self {
            root,
            binary,
            legacy_socket,
        })
    }
    /// The store the binary resolves.
    pub fn store(&self) -> Fallible<PathBuf> {
        let data = self.root.path().join("app").join("Data").canonicalize()?;
        Ok(data.join("settings_store.json"))
    }
    /// Isolated pathname of the passive legacy-owner probe (debug builds only).
    pub fn legacy_socket(&self) -> &Path {
        &self.legacy_socket
    }
    /// Run the binary with `args`; the developer's own Handy never decides the outcome.
    pub fn run(&self, args: &[&str]) -> Fallible<Run> {
        self.run_input(args, None)
    }
    /// Run the binary with `args`, and `input` on its stdin (a pipe, never a terminal) when given.
    pub fn run_input(&self, args: &[&str], input: Option<&[u8]>) -> Fallible<Run> {
        let mut child = self.spawn(args, input.is_some())?;
        if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
            use std::io::Write;
            // Dropping the handle closes the pipe, so the child sees the end of its input.
            stdin.write_all(input)?;
        }
        Ok(Run::from(child.wait_with_output()?))
    }
    fn spawn(&self, args: &[&str], piped_stdin: bool) -> Fallible<std::process::Child> {
        // macOS occasionally SIGKILLed children of a freshly linked binary when the test
        // threads launched it simultaneously; one completed launch first (help reads no
        // store) and serialized spawns avoid it. The children still run concurrently.
        let _spawning = SPAWN.lock().map_err(|_| "spawn lock poisoned")?;
        WARM_UP.get_or_init(|| {
            Command::new(env!("CARGO_BIN_EXE_handy"))
                .args(["settings", "--help"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        });
        let stdin = if piped_stdin {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };
        Ok(Command::new(&self.binary)
            .args(args)
            .env("HANDY_SETTINGS_TEST_LEGACY_SOCKET", &self.legacy_socket)
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    }
    /// Raw store bytes, or None when the store does not exist.
    pub fn store_bytes(&self) -> Fallible<Option<Vec<u8>>> {
        match std::fs::read(self.store()?) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
fn short_temp() -> PathBuf {
    if cfg!(unix) {
        PathBuf::from("/tmp")
    } else {
        std::env::temp_dir()
    }
}
