//! Source scan of the Enter emitters (009 Step 5, decision `claim_enter_scan_step`): outside
//! tests, only 009's authority calls 006's `claim_enter`; the key emitters are exactly the legacy
//! paths of `dev` (driven by `auto_submit`, which the session path sets to false or bypasses) and
//! the authority's own key press; smart paste's session method no longer submits Enter itself.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn sources() -> Vec<PathBuf> {
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory).expect("readable source directory");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if is_production_source(&path) {
                files.push(path);
            }
        }
    }
    files
}
/// Rust sources that are not test modules (`*_tests.rs`, `insertion_cases.rs`).
fn is_production_source(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    name.ends_with(".rs") && !name.ends_with("_tests.rs") && name != "insertion_cases.rs"
}
/// Production code of `text`: everything before its inline `#[cfg(test)]` module.
fn production_code(text: &str) -> &str {
    text.find("#[cfg(test)]\nmod tests")
        .map_or(text, |index| &text[..index])
}
fn files_calling(needles: &[&str]) -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    sources()
        .into_iter()
        .filter(|path| {
            let text = std::fs::read_to_string(path).expect("readable source");
            let code = production_code(&text);
            needles.iter().any(|needle| code.contains(needle))
        })
        .filter_map(|path| {
            let relative = path.strip_prefix(&root).ok()?.to_str()?.replace('\\', "/");
            Some(relative)
        })
        .collect()
}

#[test]
fn only_the_automatic_enter_authority_claims_the_enter_press() {
    let claimers = files_calling(&[".claim_enter(", "::claim_enter("]);
    let expected = ["automatic_enter/transaction.rs", "destination/manager.rs"]
        .map(str::to_owned)
        .into_iter()
        .collect::<BTreeSet<_>>();
    // `destination/manager.rs` only forwards to the session's own claim (its definition).
    assert_eq!(claimers, expected);
}
#[test]
fn the_key_emitters_are_the_legacy_paths_and_the_authority_only() {
    let emitters = files_calling(&["send_return_key(", "send_auto_submit(", "submit_enter("]);
    let expected = [
        "automatic_enter/app.rs",
        "clipboard.rs",
        "paste_tx/macos.rs",
        "paste_tx/windows.rs",
        "smart_paste/app.rs",
    ]
    .map(str::to_owned)
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(emitters, expected);
}
#[test]
fn smart_paste_session_method_hands_its_outcome_to_the_authority() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/smart_paste/app.rs");
    let text = std::fs::read_to_string(path).expect("readable source");
    let start = text.find("pub fn session_method(").expect("session_method");
    let end = start + text[start..].find("\n}\n").expect("end of session_method");
    let body = &text[start..end];
    assert!(body.contains("enter.complete("));
    assert!(!body.contains("submit_enter(") && !body.contains("claim_enter("));
}
#[test]
fn the_session_path_sends_no_legacy_enter() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/clipboard.rs");
    let text = std::fs::read_to_string(path).expect("readable source");
    // The classic 50 ms block runs only on the legacy route; the reliable paste gets `false`
    // and the authority's receipt sink on the authority route.
    assert!(text.contains("legacy_enter && should_send_auto_submit("));
    assert!(text.contains("Self::Authority(receipts) => (false, Some(receipts.clone()))"));
}
