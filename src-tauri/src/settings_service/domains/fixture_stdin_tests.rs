//! `-` (stdin) input through the test domain, in a real child process: the test binary runs
//! itself with only `stdin_child` selected and the bytes on a pipe (or a terminal) as stdin;
//! the child reads them with `read_source("-")`, runs a `words` import offline and prints the
//! reply envelope on one marked line.
use super::{call, offline};
use crate::settings_service::{input::read_source, operations::Reply};
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};
type TestResult = Result<(), Box<dyn std::error::Error>>;

// Set only for the child process, so running ignored tests by hand never reads a terminal.
const CHILD_ENV: &str = "HANDY_SETTINGS_STDIN_CHILD";
const REPLY_MARKER: &str = "STDIN-CHILD-REPLY ";

#[test]
#[ignore = "child process of the stdin tests below"]
fn stdin_child() -> TestResult {
    if std::env::var_os(CHILD_ENV).is_none() {
        return Ok(());
    }
    let directory = tempfile::tempdir()?;
    let reply = match read_source("-") {
        Ok(input) => call(
            &offline(&directory)?,
            "import",
            &["merge"],
            Some(&input),
            false,
        ),
        Err(error) => Reply::failure(error),
    };
    println!("{REPLY_MARKER}{}", reply.envelope());
    Ok(())
}

/// Run `stdin_child` in a new process of this test binary with `stdin`; its reply envelope.
fn child_reply(stdin: Stdio, bytes: Option<&[u8]>) -> Result<Value, Box<dyn std::error::Error>> {
    let path = module_path!().split_once("::").map_or("", |(_, path)| path);
    let mut child = Command::new(std::env::current_exe()?)
        .args([&format!("{path}::stdin_child"), "--exact", "--ignored"])
        .args(["--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, "1")
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    if let (Some(bytes), Some(mut input)) = (bytes, child.stdin.take()) {
        input.write_all(bytes)?;
    }
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let line = stdout
        .lines()
        // libtest prints the test name first on the same line.
        .find_map(|line| line.split_once(REPLY_MARKER).map(|(_, reply)| reply))
        .ok_or("the child printed no reply")?;
    Ok(serde_json::from_str(line)?)
}

#[test]
fn piped_stdin_feeds_the_domain_input() -> TestResult {
    let envelope = child_reply(Stdio::piped(), Some(b"handy\ncli\n\nhandy\n"))?;
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(envelope["data"]["custom_words"], json!(["handy", "cli"]));
    Ok(())
}

#[test]
fn non_utf8_stdin_is_refused_with_its_offset_before_any_change() -> TestResult {
    let envelope = child_reply(Stdio::piped(), Some(b"ok\n\xffno"))?;
    assert_eq!(
        (
            &envelope["error"]["code"],
            &envelope["error"]["details"]["offset"]
        ),
        (&json!("validation_failed"), &json!(3))
    );
    Ok(())
}

/// A terminal stdin is refused (`usage_error`) instead of waiting for typed input.
#[cfg(unix)]
#[test]
fn terminal_stdin_is_a_usage_error_without_waiting() -> TestResult {
    let envelope = child_reply(Stdio::from(terminal()?), None)?;
    assert_eq!(envelope["error"]["code"], json!("usage_error"));
    assert!(envelope["data"].is_null(), "nothing was imported");
    Ok(())
}

/// The secondary side of a new pseudo-terminal (posix_openpt, grantpt, unlockpt, ptsname: the
/// portable POSIX sequence on macOS and Linux). The primary side stays open in this process
/// for the life of the test binary, so the secondary remains a live terminal.
#[cfg(unix)]
fn terminal() -> std::io::Result<std::fs::File> {
    // SAFETY: posix_openpt returns a new descriptor or -1, checked below.
    let primary = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if primary < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the primary descriptor is valid; ptsname's static buffer is copied at once.
    let name = unsafe {
        if libc::grantpt(primary) != 0 || libc::unlockpt(primary) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let name = libc::ptsname(primary);
        if name.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        std::ffi::CStr::from_ptr(name)
            .to_string_lossy()
            .into_owned()
    };
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name)
}
