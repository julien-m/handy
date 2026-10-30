//! Bounded external command for feature 013 (spec AC-010): an addressing tool found in an
//! application's bundle is run with an argument vector (never a shell), standard input closed,
//! standard error discarded, the target variables of cmux removed from its environment, and a
//! polled wait after which it is killed. Its output is returned to the caller for identifiers and
//! names only and is never logged.
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Variables through which cmux picks a default target: removed so that only the explicit
/// `--workspace`/`--surface` arguments choose it (spec AC-007), even when Handy itself was
/// started from a cmux terminal.
const REMOVED_VARIABLES: [&str; 3] = ["CMUX_WORKSPACE_ID", "CMUX_SURFACE_ID", "CMUX_PANE_ID"];
/// Largest standard output kept: a `tree --json` of one workspace is a few kilobytes.
const MAX_OUTPUT_BYTES: u64 = 4 * 1024 * 1024;
/// Period of the exit polling: short against the 2-second bound, without a busy loop.
const POLL_PERIOD: Duration = Duration::from_millis(10);

/// What a finished command answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: String,
}

/// Why a command gave no answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliError {
    /// The program could not be started.
    Spawn,
    /// No exit within the bound: the process was killed.
    Timeout,
    /// Waiting for or reading the process failed.
    Io,
}

/// Runs one external command, bounded.
pub trait CliRunner: Send + Sync {
    /// Run `program` with `arguments`, waiting at most `timeout`.
    ///
    /// # Errors
    /// `Spawn`, `Timeout` (killed) or `Io`.
    fn run(
        &self,
        program: &Path,
        arguments: &[String],
        timeout: Duration,
    ) -> Result<CliOutput, CliError>;
}

/// The production runner over `std::process`.
pub struct ProcessRunner;

impl CliRunner for ProcessRunner {
    // @spec AC-010: No shell, closed stdin, bounded then killed — .specs/features/013-default-input-per-app/spec.md#ac-010
    fn run(
        &self,
        program: &Path,
        arguments: &[String],
        timeout: Duration,
    ) -> Result<CliOutput, CliError> {
        let mut command = Command::new(program);
        command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for variable in REMOVED_VARIABLES {
            command.env_remove(variable);
        }
        let mut child = command.spawn().map_err(|_| CliError::Spawn)?;
        // The output is drained on its own thread so a full pipe never blocks the child.
        let stdout = child.stdout.take().ok_or(CliError::Io)?;
        let reader = std::thread::spawn(move || {
            let mut text = String::new();
            let read = stdout.take(MAX_OUTPUT_BYTES).read_to_string(&mut text);
            read.map(|_| text)
        });
        let status = wait_bounded(&mut child, timeout)?;
        let stdout = reader
            .join()
            .map_err(|_| CliError::Io)?
            .map_err(|_| CliError::Io)?;
        Ok(CliOutput {
            success: status.success(),
            code: status.code(),
            stdout,
        })
    }
}

/// Exit status within `timeout`, or the child killed and reaped.
fn wait_bounded(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, CliError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|_| CliError::Io)? {
            return Ok(status);
        }
        // Past the bound: kill, reap, and report the timeout.
        if started.elapsed() >= timeout {
            // The child may have exited between the poll and the kill: both are fine.
            let _ = child.kill();
            let _ = child.wait();
            return Err(CliError::Timeout);
        }
        std::thread::sleep(POLL_PERIOD);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn a_command_answers_its_output_and_status() -> TestResult {
        let output = ProcessRunner
            .run(
                Path::new("/bin/echo"),
                &["; rm nothing".to_owned()],
                Duration::from_secs(2),
            )
            .map_err(|error| format!("echo: {error:?}"))?;
        assert!(output.success);
        // The argument reached the program verbatim: no shell interpreted it.
        assert_eq!(output.stdout.trim_end(), "; rm nothing");
        Ok(())
    }

    #[test]
    fn a_slow_command_is_killed_at_its_bound() -> TestResult {
        let started = Instant::now();
        let result = ProcessRunner.run(
            Path::new("/bin/sleep"),
            &["5".to_owned()],
            Duration::from_millis(100),
        );
        assert_eq!(result, Err(CliError::Timeout));
        assert!(started.elapsed() < Duration::from_secs(2));
        Ok(())
    }

    #[test]
    fn a_missing_program_is_a_spawn_error() -> TestResult {
        let result = ProcessRunner.run(
            Path::new("/nonexistent/handy-013-tool"),
            &[],
            Duration::from_millis(100),
        );
        assert_eq!(result, Err(CliError::Spawn));
        Ok(())
    }
}
