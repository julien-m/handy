//! Non-creating portable/installed identities use the same OS directory provider as Tauri.
use super::{
    errors::{SettingsError, SettingsErrorCode},
    store::persistence_failed,
};
use std::path::{Component, Path, PathBuf};
/// Tauri bundle identifier (`tauri.conf.json` `identifier`); a unit test keeps them equal.
const APP_IDENTIFIER: &str = "com.pais.handy";
const STORE_FILENAME: &str = crate::settings::SETTINGS_STORE_PATH;

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
/// `persistence_failed` for relative or `..` paths; `unavailable` for aliases that cross a user
/// boundary (or any Windows alias) and for foreign owners.
pub(super) fn canonical_path(path: &Path) -> Result<PathBuf, SettingsError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(persistence_failed());
    }
    reject_aliases(path)?;
    let (ancestor, missing) = nearest_existing_ancestor(path)?;
    let mut result = validate_canonical_ancestor(&ancestor)?;
    for component in missing.into_iter().rev() {
        result.push(component);
    }
    Ok(result)
}
/// Closest existing ancestor and the missing components below it, deepest first.
fn nearest_existing_ancestor(
    path: &Path,
) -> Result<(PathBuf, Vec<std::ffi::OsString>), SettingsError> {
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
    Ok((ancestor, missing))
}
/// Canonicalize the existing ancestor and require the current user to own it (Windows also
/// requires a checked, reparse-refusing handle).
fn validate_canonical_ancestor(ancestor: &Path) -> Result<PathBuf, SettingsError> {
    let canonical = ancestor.canonicalize().map_err(|_| persistence_failed())?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| persistence_failed())?;
    validate_owner(&metadata)?;
    #[cfg(windows)]
    {
        let kind = if metadata.is_dir() {
            super::windows_store::ResourceKind::Directory
        } else {
            super::windows_store::ResourceKind::File
        };
        super::windows_store::open_checked(&canonical, kind, false)
            .map_err(|_| persistence_failed())?;
    }
    Ok(canonical)
}
/// Walk every existing component before canonicalization so no alias is resolved silently. The
/// walk stops at the first missing component, which creation will add. The final component (the
/// store file itself) must never be a link: atomic replacement would replace the link, not its target.
fn reject_aliases(path: &Path) -> Result<(), SettingsError> {
    walk_aliases(path, true, 0)
}
/// Bound on nested user-owned links followed while checking one chain; deeper chains (or loops)
/// are refused. 8 is far above any real dotfile or relocated-data layout.
const MAX_ALIAS_DEPTH: u8 = 8;
/// Check each existing component of `path`; `is_store` forbids a link as the last component.
fn walk_aliases(path: &Path, is_store: bool, depth: u8) -> Result<(), SettingsError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        let metadata = match std::fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            // Early exit: missing components are created later and cannot be aliases yet.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(persistence_failed()),
        };
        if metadata.file_type().is_symlink() {
            if is_store && current.as_path() == path {
                return Err(SettingsError::new(SettingsErrorCode::Unavailable));
            }
            check_alias(&current, &metadata, depth)?;
        }
    }
    Ok(())
}
/// Plan rule: refuse only aliases that cross a user boundary. Root-owned Unix links (macOS `/var`
/// -> `/private/var`) belong to the system and are followed; a link owned by the current user is
/// followed only when every link of its chain passes this same rule (one hop at a time, bounded)
/// and its resolved target is a directory owned by the current user. Windows symlinks and
/// junctions are always refused, because their ownership is ACL-based and is not checked here.
fn check_alias(link: &Path, metadata: &std::fs::Metadata, depth: u8) -> Result<(), SettingsError> {
    let refused = || SettingsError::new(SettingsErrorCode::Unavailable);
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let user = super::ownership::current_uid();
        match metadata.uid() {
            0 => Ok(()),
            owner if owner == user && depth < MAX_ALIAS_DEPTH => {
                let target = std::fs::read_link(link).map_err(|_| refused())?;
                let parent = link.parent().ok_or_else(refused)?;
                walk_aliases(&parent.join(target), false, depth + 1)?;
                std::fs::metadata(link)
                    .is_ok_and(|target| target.is_dir() && target.uid() == user)
                    .then_some(())
                    .ok_or_else(refused)
            }
            _ => Err(refused()),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (link, metadata, depth);
        Err(refused())
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
#[cfg(test)]
mod tests {
    #[test]
    fn app_identifier_matches_the_tauri_bundle_identifier() -> Result<(), Box<dyn std::error::Error>>
    {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json"))?;
        assert_eq!(config["identifier"], super::APP_IDENTIFIER);
        Ok(())
    }
}
