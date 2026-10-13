//! Writer inventory closure: no code path writes `settings_store.json` except the settings
//! service, and the store plugin is gone (no cache, autosave or exit flush can write a stale copy,
//! and no webview can reach it). A source scan, because the forbidden calls must stay absent.
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
fn store_plugin_is_gone_and_settings_path_is_only_resolved_by_the_service() -> TestResult {
    let plugin = occurrences(&backend(), &["rs"], "tauri_plugin_store")?;
    assert!(plugin.is_empty(), "store plugin use: {plugin:?}");
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))?;
    assert!(
        !manifest.contains("tauri-plugin-store"),
        "store plugin dependency"
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
/// The file name itself (a direct `fs::write(dir.join("settings_store.json"))`) is allowed only
/// in its constant, inside the service, in tests and in comments.
#[test]
fn settings_file_name_is_only_spelled_by_the_service() -> TestResult {
    let hits = occurrences(&backend(), &["rs"], "settings_store.json")?;
    let outside: Vec<&Hit> = hits
        .iter()
        .filter(|hit| {
            let code = hit.line.split_once(": ").map_or("", |(_, code)| code);
            !(hit.file == "settings.rs"
                || hit.file.starts_with("settings_service/")
                || hit.file.ends_with("_tests.rs")
                || code.starts_with("//"))
        })
        .collect();
    assert!(
        outside.is_empty(),
        "settings file named outside the service: {outside:?}"
    );
    Ok(())
}
/// Neither webview can reach the store plugin: no capability grants it.
#[test]
fn no_webview_capability_grants_the_store_plugin() -> TestResult {
    let capabilities = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
    let hits = occurrences(&capabilities, &["json", "toml"], "\"store:")?;
    assert!(hits.is_empty(), "store plugin capability: {hits:?}");
    Ok(())
}
/// Startup order (Step 4 audit A, M1): the service is installed right after `build()`, before
/// the macOS activation policy reads `start_hidden`, and nowhere else.
#[test]
fn settings_service_is_installed_before_the_first_startup_reader() -> TestResult {
    let lib = std::fs::read_to_string(backend().join("lib.rs"))?;
    let install = lib
        .find("install_settings_service(&app, headless_mode);")
        .ok_or("service not installed after build()")?;
    let policy = lib
        .find("apply_startup_activation_policy(&mut app, headless_mode);")
        .ok_or("activation policy call not found")?;
    assert!(
        install < policy,
        "a startup reader runs before the service is installed"
    );
    let owned = lib.matches("tauri_adapter::install_owned(").count();
    let offline = lib.matches("tauri_adapter::install_offline(").count();
    assert_eq!((owned, offline), (1, 1), "service installed more than once");
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
    for needle in ["plugin-store", "plugin:store"] {
        let hits = occurrences(&frontend, &["ts", "tsx", "js", "jsx"], needle)?;
        assert!(hits.is_empty(), "frontend store plugin use: {hits:?}");
    }
    let package = std::fs::read_to_string(frontend.join("../package.json"))?;
    assert!(
        !package.contains("plugin-store"),
        "store plugin JS dependency"
    );
    Ok(())
}
