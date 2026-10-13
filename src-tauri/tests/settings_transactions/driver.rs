//! Test-only process driver links the exact production repository and coordinates stdin barriers.
use handy_app_lib::settings_service::{
    ownership::LegacyPathProbe,
    store::SettingsRepository,
    validation::{validate_cli, ValidationContext},
};
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;
const DEADLINE: Duration = Duration::from_secs(10);
pub(super) struct Driver {
    child: Child,
    input: ChildStdin,
    lines: Receiver<String>,
}
impl Driver {
    pub(super) fn start(
        path: &std::path::Path,
        mode: &str,
        key: &str,
        value: &str,
    ) -> TestResult<Self> {
        // Contract: re-run this test binary with `--exact --ignored` so only the
        // `repository_subprocess_fixture` entry point runs, in its own process.
        // Protocol on stdin/stdout lines: parent sends the JSON config, child prints READY;
        // parent sends GO, child commits and prints `RESULT <json>` (startup mode then waits
        // for RELEASE before dropping its lease), or prints CRASH_READY at the injected stage.
        // A normal run must exit 0 (`finish`). `kill_at_barrier` kills the child at
        // CRASH_READY; on Unix the status must report SIGKILL, so no Rust unwind or Drop ran.
        let mut child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "driver::repository_subprocess_fixture",
                "--ignored",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().ok_or("child stdin")?;
        let stdout = child.stdout.take().ok_or("child stdout")?;
        let (sender, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        if sender.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let mut driver = Self {
            child,
            input,
            lines,
        };
        driver.send(&json!({"path":path,"mode":mode,"key":key,"value":value}).to_string())?;
        driver.receive("READY")?;
        Ok(driver)
    }
    pub(super) fn send(&mut self, line: &str) -> TestResult<()> {
        writeln!(self.input, "{line}")?;
        self.input.flush()?;
        Ok(())
    }
    fn receive(&self, prefix: &str) -> TestResult<String> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            let line = self
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))?;
            if line.starts_with(prefix) {
                eprintln!("NESTED {} {line}", self.child.id());
                return Ok(line);
            }
        }
    }
    pub(super) fn kill_at_barrier(mut self) -> TestResult<()> {
        self.child.kill()?;
        let status = self.child.wait()?;
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(libc::SIGKILL));
        }
        eprintln!("NESTED_KILL {} {:?}", self.child.id(), status.code());
        assert!(!status.success());
        Ok(())
    }
    pub(super) fn wait_crash_barrier(&self) -> TestResult<()> {
        self.receive("CRASH_READY")?;
        Ok(())
    }
    pub(super) fn outcome(&self) -> TestResult<Value> {
        Ok(serde_json::from_str(
            self.receive("RESULT ")?
                .strip_prefix("RESULT ")
                .ok_or("result")?,
        )?)
    }
    pub(super) fn finish(mut self) -> TestResult<()> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Some(status) = self.child.try_wait()? {
                println!(
                    "NESTED_EXIT {} {}",
                    self.child.id(),
                    status.code().unwrap_or(-1)
                );
                assert!(status.success());
                return Ok(());
            }
            if Instant::now() >= deadline {
                self.child.kill()?;
                return Err("child deadline".into());
            }
            std::thread::yield_now();
        }
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        // Fixture cleanup owns only this spawned test process; never a user application process.
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
#[test]
#[ignore = "test-only subprocess entry point; executed by process/crash contracts with stdin barriers"]
fn repository_subprocess_fixture() -> TestResult<()> {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut line = String::new();
    input.read_line(&mut line)?;
    let config: Value = serde_json::from_str(&line)?;
    let path = std::path::PathBuf::from(config["path"].as_str().ok_or("path")?);
    let probe = LegacyPathProbe::new(path.with_file_name("absent-legacy.sock"));
    let repository = SettingsRepository::new(path);
    println!("READY");
    std::io::stdout().flush()?;
    line.clear();
    input.read_line(&mut line)?;
    assert_eq!(line.trim(), "GO");
    if config["mode"] == "startup" {
        let (read, lease) = repository.initialize(&probe)?;
        println!(
            "RESULT {}",
            json!({"ok":true,"revision":read.snapshot.revision.0})
        );
        std::io::stdout().flush()?;
        line.clear();
        input.read_line(&mut line)?;
        assert_eq!(line.trim(), "RELEASE");
        drop(lease);
    } else {
        drop(input);
        write_fixture(&repository, &probe, &config)?;
    }
    Ok(())
}

struct CrashPause(handy_app_lib::settings_service::transaction::PersistStage);
impl handy_app_lib::settings_service::transaction::PersistObserver for CrashPause {
    fn observe(
        &self,
        stage: handy_app_lib::settings_service::transaction::PersistStage,
    ) -> std::io::Result<()> {
        if stage == self.0 {
            println!("CRASH_READY");
            std::io::stdout().flush()?;
            // The parent terminates this owned fixture here; no Rust error/unwind/Drop simulates the crash.
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line)?;
            return Err(std::io::Error::other("crash barrier unexpectedly released"));
        }
        Ok(())
    }
}
fn write_fixture(
    repository: &SettingsRepository,
    probe: &LegacyPathProbe,
    config: &Value,
) -> TestResult<()> {
    use handy_app_lib::settings_service::transaction::PersistStage;
    let mutation = validate_cli(
        config["key"].as_str().ok_or("key")?,
        config["value"].as_str().ok_or("value")?,
        &ValidationContext {
            prompt_ids: &[],
            updates_allowed: true,
        },
    )?;
    let result = match config["mode"].as_str() {
        Some("crash_before") => repository.commit_with_probe(
            mutation,
            None,
            probe,
            &CrashPause(PersistStage::BeforeReplace),
        ),
        Some("crash_after") => repository.commit_with_probe(
            mutation,
            None,
            probe,
            &CrashPause(PersistStage::AfterReplace),
        ),
        _ => repository.commit_with_probe(mutation, None, probe, &()),
    };
    let outcome = match result {
        Ok(read) => json!({"ok":true,"revision":read.snapshot.revision.0}),
        Err(error) => json!({"ok":false,"error":error}),
    };
    println!("RESULT {outcome}");
    std::io::stdout().flush()?;
    Ok(())
}
