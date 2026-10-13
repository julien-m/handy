//! Fresh preserved raw documents and typed non-mutating settings views.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    protocol::{Revision, SettingsSnapshot},
};
use crate::settings::get_default_settings;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Repository identity stays stable while each JSON generation is replaced atomically.
pub struct SettingsRepository {
    pub(crate) path: PathBuf,
}
/// Recovery warning contains no raw document, field name or parse diagnostics.
pub struct SettingsRead {
    pub snapshot: SettingsSnapshot,
    pub warning: bool,
}
/// Raw documents remain internal: only transactions may replace their generations.
pub(super) struct Document {
    pub raw: Value,
    pub read: SettingsRead,
    pub writable: bool,
    pub missing: bool,
    /// Unix (dev, ino) of the store file that was read; `None` when the store was absent.
    #[cfg(unix)]
    pub source: Option<(u64, u64)>,
}
impl SettingsRepository {
    /// Bind an isolated path; construction performs no filesystem operations.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    /// Read fresh bytes and normalize only in memory.
    ///
    /// Side effects: never creates the data parent or the store and never rewrites the store. When
    /// the parent already exists, the read takes the transaction lock and may therefore create the
    /// sibling `settings.lock` file, which is intentionally retained after the read.
    ///
    /// # Errors
    /// `unavailable` for foreign-owned or aliased paths, `persistence_failed` for unreadable files.
    // @spec FR-002: Non-creating fresh reads — .specs/features/008-settings-cli/spec.md#fr-002
    pub fn read_snapshot(&self) -> Result<SettingsRead, SettingsError> {
        let path = super::locator::canonical_path(&self.path)?;
        if !path.parent().is_some_and(std::path::Path::exists) {
            return Ok(fallback(true)?.read);
        }
        let _lock =
            super::ownership::FileLock::acquire(path.with_file_name("settings.lock"), true)?;
        Ok(self.load_at(&path)?.read)
    }
    /// Load the current generation; a missing store yields defaults that remain writable.
    pub(super) fn load_at(&self, path: &std::path::Path) -> Result<Document, SettingsError> {
        #[cfg(windows)]
        let _parent =
            super::windows_store::pin_parent(path.parent().ok_or_else(persistence_failed)?)
                .map_err(|_| persistence_failed())?;
        match open_store(path) {
            Ok(file) => read_existing(file),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => fallback(true),
            Err(_) => Err(persistence_failed()),
        }
    }
}
/// Open without following a final symlink; Windows uses its checked, reparse-refusing handle.
fn open_store(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
    }
    #[cfg(windows)]
    {
        super::windows_store::open_checked(path, super::windows_store::ResourceKind::File, false)
    }
}
fn read_existing(mut file: std::fs::File) -> Result<Document, SettingsError> {
    let metadata = file.metadata().map_err(|_| persistence_failed())?;
    super::locator::validate_owner(&metadata)?;
    if !metadata.is_file() {
        return Err(persistence_failed());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(persistence_failed());
        }
    }
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes).map_err(|_| persistence_failed())?;
    let loaded = match serde_json::from_slice::<Value>(&bytes) {
        Ok(value) if value.is_object() => document(value)?,
        _ => fallback(false)?,
    };
    #[cfg(unix)]
    let loaded = Document {
        source: Some(super::locator::unix_identity(&metadata)),
        ..loaded
    };
    Ok(loaded)
}
fn document(raw: Value) -> Result<Document, SettingsError> {
    // A missing legacy revision starts at zero; an existing malformed counter can never authorize writes.
    let valid_revision = raw.get("settings_revision").map(Value::as_u64);
    let writable =
        !matches!(valid_revision, Some(None)) && raw.get("settings").is_none_or(Value::is_object);
    let revision = Revision(valid_revision.flatten().unwrap_or(0));
    let payload = raw.get("settings").cloned().unwrap_or_else(|| json!({}));
    let (settings, recovered) = super::normalization::normalize(&payload)?;
    let warning = recovered || !writable;
    Ok(Document {
        raw,
        read: SettingsRead {
            snapshot: SettingsSnapshot { settings, revision },
            warning,
        },
        writable,
        missing: false,
        #[cfg(unix)]
        source: None,
    })
}
fn fallback(missing: bool) -> Result<Document, SettingsError> {
    Ok(Document {
        raw: json!({}),
        read: SettingsRead {
            snapshot: SettingsSnapshot {
                settings: get_default_settings(),
                revision: Revision(0),
            },
            warning: !missing,
        },
        writable: missing,
        missing,
        #[cfg(unix)]
        source: None,
    })
}
pub(super) fn persistence_failed() -> SettingsError {
    SettingsError::new(SettingsErrorCode::PersistenceFailed)
}
