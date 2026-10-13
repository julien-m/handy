//! Writer inventory closure: no code path writes `settings_store.json` except the settings
//! service, and the store plugin never opens it (so no cache autosave or exit flush can write a
//! stale copy). A source scan, because the forbidden calls are what must stay absent.
use std::path::{Path, PathBuf};
type TestResult = Result<(), Box<dyn std::error::Error>>;
/// One source line containing a searched pattern.
#[derive(Debug)]
struct Hit {
    /// Path relative to the scanned root, with `/` separators.
    file: String,
    /// `line number: trimmed text`, for the failure message.
    line: String,
}

/// Every file with one of `extensions` below `root`, recursively.
fn sources(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extensions.iter().any(|wanted| extension == *wanted))
            {
                found.push(path);
            }
        }
    }
    Ok(found)
}
/// Every line below `root` containing `needle`.
fn occurrences(
    root: &Path,
    extensions: &[&str],
    needle: &str,
) -> Result<Vec<Hit>, Box<dyn std::error::Error>> {
    let mut hits = Vec::new();
    for path in sources(root, extensions)? {
        let text = std::fs::read_to_string(&path)?;
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        for (index, line) in text.lines().enumerate() {
            if line.contains(needle) {
                hits.push(Hit {
                    file: relative.clone(),
                    line: format!("{}: {}", index + 1, line.trim()),
                });
            }
        }
    }
    Ok(hits)
}
fn backend() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}
#[test]
fn no_backend_code_writes_settings_outside_the_service() -> TestResult {
    for needle in [
        "write_settings(",
        "StoreExt",
        "store_path(SETTINGS_STORE_PATH",
        ".save()",
    ] {
        let hits = occurrences(&backend(), &["rs"], needle)?;
        assert!(
            hits.is_empty(),
            "direct settings writer `{needle}`: {hits:?}"
        );
    }
    Ok(())
}
#[test]
fn store_plugin_is_only_registered_and_settings_path_is_only_resolved_by_the_service() -> TestResult
{
    let plugin = occurrences(&backend(), &["rs"], "tauri_plugin_store")?;
    let registration = plugin
        .iter()
        .all(|hit| hit.file == "lib.rs" && hit.line.contains("tauri_plugin_store::Builder"));
    assert!(
        registration && plugin.len() == 1,
        "store plugin use: {plugin:?}"
    );
    let store_name = occurrences(&backend(), &["rs"], "SETTINGS_STORE_PATH")?;
    for hit in &store_name {
        assert!(
            hit.file == "settings.rs" || hit.file == "settings_service/locator.rs",
            "settings store path resolved outside the service: {store_name:?}"
        );
    }
    Ok(())
}
#[test]
fn only_the_settings_service_emits_settings_changed() -> TestResult {
    let hits = occurrences(&backend(), &["rs"], "\"settings-changed\"")?;
    assert!(
        hits.iter()
            .all(|hit| hit.file == "settings_service/tauri_adapter.rs"),
        "ad hoc settings-changed emitter: {hits:?}"
    );
    Ok(())
}
#[test]
fn frontend_never_opens_the_settings_store_directly() -> TestResult {
    let frontend = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let hits = occurrences(&frontend, &["ts", "tsx"], "plugin-store")?;
    assert!(hits.is_empty(), "frontend store plugin use: {hits:?}");
    Ok(())
}
