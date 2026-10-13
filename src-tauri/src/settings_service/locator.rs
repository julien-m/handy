//! Non-creating portable/installed identities use the same OS directory provider as Tauri.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    store::persistence_failed,
};
use std::path::{Component, Path, PathBuf};
const APP_IDENTIFIER: &str = "com.pais.handy";
const STORE_FILENAME: &str = "settings_store.json";

/// Side-effect-free installed/portable store resolver; no marker upgrade or Data creation.
pub struct StoreLocator;
impl StoreLocator {
    /// Resolve current executable and OS user data without starting Tauri or changing the environment.
    // @spec FR-002: Pure store identity resolution — .specs/features/008-settings-cli/spec.md#fr-002
    pub fn current() -> Result<PathBuf, SettingsError> {
        let executable = std::env::current_exe().map_err(|_| persistence_failed())?;
        let installed = dirs::data_dir()
            .ok_or_else(persistence_failed)?
            .join(APP_IDENTIFIER);
        Self::resolve(&executable, &installed)
    }
    /// Explicit path inputs support installed/portable parity tests without modifying global configuration.
    pub fn resolve(executable: &Path, installed_data: &Path) -> Result<PathBuf, SettingsError> {
        let directory = executable.parent().ok_or_else(persistence_failed)?;
        let data = directory.join("Data");
        let marker = directory.join("portable");
        let portable = crate::portable::resolve_data_dir(&marker, &data)?;
        canonical_path(
            &portable
                .unwrap_or_else(|| installed_data.to_path_buf())
                .join(STORE_FILENAME),
        )
    }
}
/// Canonicalize the closest existing ancestor and append only normal missing components.
///
/// # Errors
/// `persistence_failed` for relative or `..` paths; `unavailable` for aliases or foreign owners.
pub(super) fn canonical_path(path: &Path) -> Result<PathBuf, SettingsError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(persistence_failed());
    }
    reject_aliases(path)?;
    let mut ancestor = path.to_path_buf();
    let mut missing = Vec::new();
    while !ancestor.exists() {
        missing.push(
            ancestor
                .file_name()
                .ok_or_else(persistence_failed)?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(persistence_failed)?
            .to_path_buf();
    }
    let canonical = ancestor.canonicalize().map_err(|_| persistence_failed())?;
    validate_owner(&std::fs::metadata(&canonical).map_err(|_| persistence_failed())?)?;
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(&canonical).map_err(|_| persistence_failed())?;
        let kind = if metadata.is_dir() {
            super::windows_store::ResourceKind::Directory
        } else {
            super::windows_store::ResourceKind::File
        };
        super::windows_store::open_checked(&canonical, kind, false)
            .map_err(|_| persistence_failed())?;
    }
    let mut result = canonical;
    for component in missing.into_iter().rev() {
        result.push(component);
    }
    Ok(result)
}
/// Walk every existing component before canonicalization so no store, parent or ancestor alias is
/// silently resolved. The walk stops at the first missing component, which creation will add.
fn reject_aliases(path: &Path) -> Result<(), SettingsError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        let metadata = match std::fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(persistence_failed()),
        };
        if metadata.file_type().is_symlink() && !is_system_alias(&metadata) {
            return Err(SettingsError::new(SettingsErrorCode::Unavailable));
        }
    }
    Ok(())
}
/// Root-owned Unix links such as macOS `/var` -> `/private/var` belong to the operating system and
/// only an administrator can change them, so they are followed. Any other symlink, and every
/// Windows symlink or junction (reported as a symlink by the standard library), is refused.
fn is_system_alias(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.uid() == 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}
/// Unix (device, inode) identity used to detect a substituted directory entry.
#[cfg(unix)]
pub(super) fn unix_identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}
/// Refuse foreign owner/type aliases; Windows native ACL implementation remains platform-specific.
pub(super) fn validate_owner(metadata: &std::fs::Metadata) -> Result<(), SettingsError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != super::ownership::current_uid() {
            return Err(SettingsError::new(SettingsErrorCode::Unavailable));
        }
    }
    if metadata.file_type().is_symlink() {
        return Err(SettingsError::new(SettingsErrorCode::Unavailable));
    }
    Ok(())
}
