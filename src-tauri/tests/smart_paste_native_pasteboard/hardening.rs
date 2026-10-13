//! Native checks of the hardening round: which helper binary runs, how
//! restoration and materialization prepare the pasteboard, withheld
//! concealed content, the warm helper and inventory truncation.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use handy_app_lib::smart_paste::ids::{TransactionId, Transcript};
use handy_app_lib::smart_paste::pasteboard_port::{
    GuardedWrite, PasteboardPort, TranscriptMarking,
};
use handy_app_lib::smart_paste::pasteboard_types::{CONCEALED_TYPE, HANDY_PRIVACY_MARKERS};
use handy_app_lib::smart_paste::snapshot::SnapshotBudget;
use handy_app_lib::smart_paste::snapshot_protocol::{MAGIC, MAX_ITEMS, NONCE_LEN, VERSION};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::recorder::take_calls;
use super::scenario::{
    ensure, is_restored_exactly, publish, read_types, rich_fixture, saved, timed_capture,
    type_names, TRANSCRIPT,
};
use super::support::{external_item_count, read_all, seed, Harness, TypeBytes};

/// A helper without a start trigger exits on its own; five seconds fail closed.
const HELPER_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(5);
/// Lets an armed helper finish starting before its capture is timed.
const ARMED_START_ALLOWANCE: Duration = Duration::from_millis(500);
/// Items beyond the inventory limit in the truncation fixture.
const EXTRA_ITEMS: usize = 3;
/// Bound on waiting for a large write to reach other processes.
const LARGE_WRITE_COMMIT_TIMEOUT: Duration = Duration::from_secs(10);

fn helper_path() -> &'static str {
    env!("CARGO_BIN_EXE_handy")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Runs the helper without its start trigger: it must answer with the
/// current protocol header and nonce, then exit without reading.
fn helper_header(name: &str) -> Result<Vec<u8>, String> {
    let nonce = "ab".repeat(NONCE_LEN);
    let mut child = Command::new(helper_path())
        .args(["--smart-paste-snapshot", &nonce, "1000", name])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + HELPER_PREFLIGHT_TIMEOUT;
    while child.try_wait().map_err(|e| e.to_string())?.is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("helper did not exit without its trigger".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut output = Vec::new();
    let stdout = child.stdout.as_mut().ok_or("helper stdout missing")?;
    stdout.read_to_end(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}

/// Identifies the helper binary used by every capture of this run and
/// checks it speaks the current protocol before any scenario runs.
pub fn helper_identity(harness: &mut Harness) -> Result<serde_json::Value, String> {
    let binary = std::fs::read(helper_path()).map_err(|error| error.to_string())?;
    let header = helper_header(&harness.name)?;
    let mut expected = MAGIC.to_vec();
    expected.push(VERSION);
    expected.extend([0xab_u8; NONCE_LEN]);
    ensure(
        header == expected,
        "helper binary does not answer with the current protocol header",
    )?;
    Ok(json!({
        "check": "helper_identity",
        "helper_path": helper_path(),
        "helper_sha256": sha256_hex(&binary),
        "protocol_version": VERSION,
        "header_without_trigger_only": true,
    }))
}

/// Restoration prepares the pasteboard as current-host-only content.
pub fn restore_is_current_host_only(harness: &mut Harness) -> Result<serde_json::Value, String> {
    seed(&harness.name, rich_fixture())?;
    let (snapshot, _) = timed_capture(harness, SnapshotBudget::default())?;
    let saved_types = saved(&snapshot);
    let _ = take_calls(&harness.name);
    let publication = publish(harness, 11)?;
    let publish_calls = take_calls(&harness.name);
    let restored = harness
        .pasteboard
        .restore_snapshot(Arc::new(snapshot), publication)
        .map_err(|error| error.to_string())?;
    let restore_calls = take_calls(&harness.name);
    let restored_types = read_all(&harness.name)?;
    let is_restored_exactly = is_restored_exactly(&saved_types, &restored_types);
    harness.pasteboard.release_transaction(TransactionId(11));
    ensure(
        restored == GuardedWrite::Written && is_restored_exactly,
        &format!(
            "host-only restore failed: {restored:?}, saved {:?}, restored {:?}",
            type_names(&saved_types),
            type_names(&restored_types)
        ),
    )?;
    ensure(
        publish_calls == ["prepare(1)"] && restore_calls == ["prepare(1)"],
        "publication or restoration did not use current-host-only",
    )?;
    Ok(json!({
        "check": "restore_is_current_host_only",
        "publish_calls": publish_calls,
        "restore_calls": restore_calls,
        "restored_exactly": is_restored_exactly,
    }))
}

fn materialize(
    harness: &mut Harness,
    transaction: u64,
    marking: TranscriptMarking,
) -> Result<(Vec<String>, Vec<String>), String> {
    let publication = publish(harness, transaction)?;
    let _ = take_calls(&harness.name);
    let transcript = Transcript::new(TRANSCRIPT);
    let written = harness
        .pasteboard
        .materialize_transcript(&transcript, publication, marking)
        .map_err(|error| error.to_string())?;
    let calls = take_calls(&harness.name);
    let types = read_types(&harness.name)?;
    harness
        .pasteboard
        .release_transaction(TransactionId(transaction));
    ensure(written == GuardedWrite::Written, "materialize failed")?;
    Ok((calls, types))
}

/// Outside copy mode the eager transcript stays current-host-only and
/// marked; in copy mode it is ordinary plain text.
pub fn materialize_markings(harness: &mut Harness) -> Result<serde_json::Value, String> {
    let (private_calls, private_types) = materialize(harness, 12, TranscriptMarking::Private)?;
    let (plain_calls, plain_types) = materialize(harness, 13, TranscriptMarking::Plain)?;
    let has_markers = |types: &[String]| {
        HANDY_PRIVACY_MARKERS
            .iter()
            .all(|marker| types.iter().any(|uti| uti == marker))
    };
    ensure(
        private_calls == ["prepare(1)"] && has_markers(&private_types),
        "private materialization is not current-host-only and marked",
    )?;
    ensure(
        plain_calls == ["clear"]
            && !plain_types
                .iter()
                .any(|uti| uti.starts_with("org.nspasteboard")),
        "copy-mode materialization is not ordinary plain text",
    )?;
    Ok(json!({
        "check": "materialize_markings",
        "private": {"calls": private_calls, "types": private_types},
        "plain": {"calls": plain_calls, "types": plain_types},
    }))
}

/// A synthetic password-manager copy is detected through the real helper
/// and its snapshot keeps no bytes.
pub fn concealed_capture_is_withheld(harness: &mut Harness) -> Result<serde_json::Value, String> {
    let secret: Vec<TypeBytes> = vec![
        (
            "public.utf8-plain-text".into(),
            b"synthetic-secret".to_vec(),
        ),
        (CONCEALED_TYPE.into(), Vec::new()),
    ];
    seed(&harness.name, vec![secret])?;
    let (snapshot, _) = timed_capture(harness, SnapshotBudget::default())?;
    let withheld = snapshot.withhold_if_privacy_marked();
    ensure(
        withheld.is_restoration_withheld() && withheld.saved_bytes() == 0,
        "concealed content was not withheld",
    )?;
    Ok(json!({"check": "concealed_capture_is_withheld", "withheld": true, "saved_bytes": 0}))
}

/// The helper started ahead of the capture keeps its startup out of the
/// capture time; a fresh helper pays it.
pub fn armed_capture_is_warm(harness: &mut Harness) -> Result<serde_json::Value, String> {
    let fixture: Vec<TypeBytes> = vec![("com.handy.test.binary".into(), (0..=63).collect())];
    seed(&harness.name, vec![fixture.clone()])?;
    let budget = SnapshotBudget::default();
    harness.pasteboard.prepare_capture(budget);
    std::thread::sleep(ARMED_START_ALLOWANCE);
    let (armed, armed_ms) = timed_capture(harness, budget)?;
    ensure(
        saved(&armed) == vec![fixture],
        "armed capture lost the fixture",
    )?;
    Ok(json!({
        "check": "armed_capture_is_warm",
        "armed_capture_ms": armed_ms,
        "armed_capture_within_200_ms": armed_ms <= 200.0,
    }))
}

/// Waits until another process sees all `items` of a large write.
fn wait_until_visible(name: &str, items: usize) -> Result<(), String> {
    let deadline = Instant::now() + LARGE_WRITE_COMMIT_TIMEOUT;
    while external_item_count(name)? < items {
        if Instant::now() >= deadline {
            return Err("the large fixture never became visible to other processes".into());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}

/// More items than the protocol limit are truncated and counted, not
/// rejected.
pub fn oversized_inventory_is_truncated(
    harness: &mut Harness,
) -> Result<serde_json::Value, String> {
    let item = |index: usize| vec![("com.handy.test.item".to_string(), vec![(index % 251) as u8])];
    let seeded = MAX_ITEMS + EXTRA_ITEMS;
    seed(&harness.name, (0..seeded).map(item).collect())?;
    wait_until_visible(&harness.name, seeded)?;
    // A long deadline isolates the truncation from the time budget.
    let budget = SnapshotBudget {
        deadline: Duration::from_secs(20),
        ..SnapshotBudget::default()
    };
    let (snapshot, capture_ms) = timed_capture(harness, budget)?;
    let observed = (snapshot.omitted().items, snapshot.items().len());
    ensure(
        observed == (EXTRA_ITEMS, MAX_ITEMS),
        &format!("inventory was not truncated to the item limit: {observed:?}"),
    )?;
    Ok(json!({
        "check": "oversized_inventory_is_truncated",
        "seeded_items": MAX_ITEMS + EXTRA_ITEMS,
        "inventoried_items": snapshot.items().len(),
        "omitted_items": snapshot.omitted().items,
        "saved_types": snapshot.saved_type_count(),
        "capture_ms": capture_ms,
    }))
}
