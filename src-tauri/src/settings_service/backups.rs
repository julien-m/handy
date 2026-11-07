//! Private copies of committed settings generations (feature 023).
//!
//! Every generation this service writes is copied to `settings_backups/` next to the store. At
//! startup, a store whose revision went back, or whose payload changed without a new revision,
//! was rewritten by another program (typically an older Handy that drops the settings it does not
//! know): the latest copy is restored and the rewritten document is set aside, never deleted.
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Folder of the copies, next to `settings_store.json`.
const DIR: &str = "settings_backups";
/// Copies kept; the oldest are removed beyond this count.
const KEEP: usize = 20;
/// Prefix of a committed generation's copy; the revision follows, zero-padded so names sort.
const GENERATION: &str = "settings_store.r";
/// Prefix of a document set aside because it was replaced by a restore.
const SET_ASIDE: &str = "overwritten-";

/// Revision stored in a raw document; a missing or malformed counter counts as zero.
pub(super) fn revision_of(raw: &Value) -> u64 {
    raw.get("settings_revision")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

// @spec FR-001: Every committed generation is copied — .specs/features/023-settings-backups/spec.md#fr-001
/// Copy `raw`, just committed at `store`, and prune the oldest copies. Best effort: a failed copy
/// is logged and never fails the commit that is already durable.
pub(super) fn keep(store: &Path, raw: &Value) {
    let name = format!("{GENERATION}{:020}.json", revision_of(raw));
    if let Err(error) = write_private(store, &name, raw).and_then(|dir| prune(&dir)) {
        log::warn!("Could not keep a backup copy of the settings: {error}");
    }
}

/// Latest copy as `(revision, document)`, if any can be read.
pub(super) fn latest(store: &Path) -> Option<(u64, Value)> {
    let dir = backup_dir(store)?;
    let newest = generations(&dir).ok()?.pop()?;
    let bytes = std::fs::read(dir.join(newest)).ok()?;
    let raw = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .filter(Value::is_object)?;
    Some((revision_of(&raw), raw))
}

// @spec FR-002: A foreign rewrite is detected at startup — .specs/features/023-settings-backups/spec.md#fr-002
/// The document to restore when `current` was rewritten by another program since `backup`, the
/// latest copy, was committed: its revision went back, or its payload changed at the same
/// revision. The restored document takes the next revision after both.
pub(super) fn restoration(current: &Value, backup_revision: u64, backup: &Value) -> Option<Value> {
    let revision = revision_of(current);
    let rewritten = revision < backup_revision
        || (revision == backup_revision && current.get("settings") != backup.get("settings"));
    if !rewritten {
        return None;
    }
    let mut restored = backup.clone();
    restored["settings_revision"] = serde_json::json!(revision.max(backup_revision) + 1);
    Some(restored)
}

/// Keep the document a restore replaces, so nothing is ever lost.
pub(super) fn set_aside(store: &Path, raw: &Value) {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let name = format!("{SET_ASIDE}{seconds}.json");
    if let Err(error) = write_private(store, &name, raw) {
        log::warn!("Could not set aside the rewritten settings: {error}");
    }
}

fn backup_dir(store: &Path) -> Option<PathBuf> {
    Some(store.parent()?.join(DIR))
}

/// Committed copies, oldest first.
fn generations(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(GENERATION) && name.ends_with(".json"))
        .collect();
    names.sort();
    Ok(names)
}

fn prune(dir: &Path) -> std::io::Result<()> {
    let names = generations(dir)?;
    for name in names.iter().take(names.len().saturating_sub(KEEP)) {
        std::fs::remove_file(dir.join(name))?;
    }
    Ok(())
}

/// Write `raw` as `name` in the private backup folder (it can hold secrets such as API keys):
/// a private temporary file is written, synced, then renamed over the copy.
fn write_private(store: &Path, name: &str, raw: &Value) -> std::io::Result<PathBuf> {
    use std::io::Write;
    let dir = backup_dir(store).ok_or(std::io::ErrorKind::NotFound)?;
    create_private_dir(&dir)?;
    let temporary = dir.join(format!(".{name}.tmp"));
    let bytes = serde_json::to_vec_pretty(raw).map_err(std::io::Error::other)?;
    let mut file = create_private_file(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    std::fs::rename(&temporary, dir.join(name))?;
    Ok(dir)
}

fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
    }
    #[cfg(windows)]
    {
        // The store's parent is already private; the folder inherits its access rules.
        std::fs::create_dir_all(dir)
    }
}

fn create_private_file(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn store_in(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("settings_store.json")
    }

    fn doc(revision: u64, words: &[&str]) -> Value {
        json!({ "settings_revision": revision, "settings": { "custom_words": words } })
    }

    #[test]
    fn the_latest_copy_is_the_highest_revision() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = store_in(&dir);
        keep(&store, &doc(9, &["a"]));
        keep(&store, &doc(10, &["b"]));
        let latest = latest(&store).map(|(revision, raw)| (revision, raw["settings"].clone()));
        assert_eq!(latest, Some((10, json!({ "custom_words": ["b"] }))));
        Ok(())
    }

    #[test]
    fn only_the_newest_copies_are_kept() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = store_in(&dir);
        for revision in 0..(KEEP as u64 + 5) {
            keep(&store, &doc(revision, &[]));
        }
        let names = generations(&dir.path().join(DIR))?;
        assert_eq!(names.len(), KEEP);
        assert_eq!(
            latest(&store).map(|(revision, _)| revision),
            Some(KEEP as u64 + 4)
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn copies_are_private() -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir()?;
        let store = store_in(&dir);
        keep(&store, &doc(1, &[]));
        let folder = dir.path().join(DIR);
        let copy = folder.join(format!("{GENERATION}{:020}.json", 1));
        assert_eq!(
            std::fs::metadata(&folder)?.permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(std::fs::metadata(copy)?.permissions().mode() & 0o777, 0o600);
        Ok(())
    }

    #[test]
    fn a_rewrite_at_the_same_revision_is_restored() {
        let backup = doc(95, &["kept"]);
        let restored = restoration(&doc(95, &[]), 95, &backup);
        assert_eq!(
            restored.map(|raw| (revision_of(&raw), raw["settings"].clone())),
            Some((96, json!({ "custom_words": ["kept"] })))
        );
    }

    #[test]
    fn a_revision_that_went_back_is_restored() {
        let current = json!({ "settings": { "custom_words": [] } });
        let restored = restoration(&current, 7, &doc(7, &["kept"]));
        assert_eq!(restored.map(|raw| revision_of(&raw)), Some(8));
    }

    #[test]
    fn the_committed_generation_or_a_newer_one_is_left_alone() {
        let backup = doc(7, &["kept"]);
        assert_eq!(restoration(&doc(7, &["kept"]), 7, &backup), None);
        assert_eq!(restoration(&doc(8, &["new"]), 7, &backup), None);
    }
}
