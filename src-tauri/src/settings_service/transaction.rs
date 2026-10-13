//! Lock/reload/patch transactions preserve raw fields and independent startup ownership.
pub use super::atomic::{PersistObserver, PersistStage};
use super::{
    errors::SettingsError,
    ownership::{require_absent, FileLock, NativeOwnerProbe, OwnerProbe, StartupLease},
    protocol::{Revision, SettingsSnapshot},
    raw_merge::{apply_changed_fields, normalized_payload},
    store::{persistence_failed, Document, SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use std::path::PathBuf;

/// Locks and the fresh generation held for the whole transaction.
struct LockedDocument {
    path: PathBuf,
    transaction: FileLock,
    lease: FileLock,
    /// Parent (dev, ino) recorded right after the transaction lock was acquired.
    #[cfg(unix)]
    parent: (u64, u64),
    document: Document,
}
impl SettingsRepository {
    /// Commit a validated scalar against fresh bytes using native passive ownership checks.
    // @spec FR-003: Durable typed patch and revision — .specs/features/008-settings-cli/spec.md#fr-003
    pub fn commit_mutation(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        self.commit_with_probe(mutation, expected, &NativeOwnerProbe, &())
    }
    /// Isolated OS probe/fault ports test the actual repository and durable replacement.
    ///
    /// # Errors
    /// `revision_conflict` for a stale `expected`, `validation_failed` when the value is no longer
    /// valid against the locked snapshot, `unavailable` when an owner or alias is detected,
    /// `persistence_failed` when nothing was replaced, and `unavailable` with outcome `unknown`
    /// when replacement happened but durability could not be confirmed.
    pub fn commit_with_probe(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
        probe: &impl OwnerProbe,
        observer: &impl PersistObserver,
    ) -> Result<SettingsRead, SettingsError> {
        let mut locked = self.lock_and_load(probe, observer)?;
        check_revision(&locked.document, expected)?;
        let mutation = revalidate_mutation(&locked.document.read.snapshot.settings, mutation)?;
        let Some((settings, revision)) = apply_mutation(&mut locked.document, &mutation)? else {
            return Ok(locked.document.read);
        };
        self.persist_locked(&locked, probe, observer)?;
        Ok(SettingsRead {
            snapshot: SettingsSnapshot { settings, revision },
            warning: locked.document.read.warning,
        })
    }
    /// Startup normalizes explicitly under the transaction lock and retains a separate lease until shutdown.
    // @spec FR-004: Startup/offline ownership fence — .specs/features/008-settings-cli/spec.md#fr-004
    pub fn initialize(
        &self,
        probe: &impl OwnerProbe,
    ) -> Result<(SettingsRead, StartupLease), SettingsError> {
        let mut locked = self.lock_and_load(probe, &())?;
        check_revision(&locked.document, None)?;
        if let Some(revision) = normalize_document(&mut locked.document)? {
            self.persist_locked(&locked, probe, &())?;
            locked.document.read.snapshot.revision = revision;
        }
        Ok((locked.document.read, StartupLease { lock: locked.lease }))
    }
    /// Order matters: transaction lock, parent identity, nonblocking startup lease, passive owner
    /// probe, then the fresh read, so the read always belongs to the locked parent generation.
    fn lock_and_load(
        &self,
        probe: &impl OwnerProbe,
        observer: &impl PersistObserver,
    ) -> Result<LockedDocument, SettingsError> {
        let path = self.prepare_parent()?;
        let transaction = FileLock::acquire(path.with_file_name("settings.lock"), true)?;
        #[cfg(unix)]
        let parent = parent_identity(&path)?;
        let lease = FileLock::acquire(path.with_file_name("settings.instance.lock"), false)?;
        require_absent(probe)?;
        let document = self.load_at(&path)?;
        observer
            .observe(PersistStage::AfterLoad)
            .map_err(|_| persistence_failed())?;
        Ok(LockedDocument {
            path,
            transaction,
            lease,
            #[cfg(unix)]
            parent,
            document,
        })
    }
    /// Replace the generation only after every identity captured under the lock still holds.
    fn persist_locked(
        &self,
        locked: &LockedDocument,
        probe: &impl OwnerProbe,
        observer: &impl PersistObserver,
    ) -> Result<(), SettingsError> {
        super::atomic::persist(&locked.path, &locked.document.raw, observer, || {
            locked.transaction.validate()?;
            locked.lease.validate()?;
            if super::locator::canonical_path(&self.path)? != locked.path {
                return Err(persistence_failed());
            }
            #[cfg(unix)]
            verify_origin(locked)?;
            // Last passive probe occurs after temporary-file fsync and immediately before replacement.
            require_absent(probe)
        })
    }
    fn prepare_parent(&self) -> Result<PathBuf, SettingsError> {
        let path = super::locator::canonical_path(&self.path)?;
        let parent = path.parent().ok_or_else(persistence_failed)?;
        if !parent.exists() {
            // Creation belongs only to an explicit changed/startup operation; read never calls this function.
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                std::fs::DirBuilder::new()
                    .recursive(true)
                    .mode(0o700)
                    .create(parent)
                    .map_err(|_| persistence_failed())?;
            }
            #[cfg(windows)]
            super::windows_store::ensure_private_parent(parent)
                .map_err(|_| persistence_failed())?;
        }
        super::locator::canonical_path(&path)
    }
}
#[cfg(unix)]
fn parent_identity(path: &std::path::Path) -> Result<(u64, u64), SettingsError> {
    let parent = path.parent().ok_or_else(persistence_failed)?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| persistence_failed())?;
    Ok(super::locator::unix_identity(&metadata))
}
/// Detect (not prevent) a same-user rename-and-restore of the parent during the transaction: the
/// parent must be the directory locked earlier and the store pathname must still name the exact
/// file that was read (or still be absent). Windows pins the parent through the lock handle instead.
#[cfg(unix)]
fn verify_origin(locked: &LockedDocument) -> Result<(), SettingsError> {
    if parent_identity(&locked.path)? != locked.parent {
        return Err(persistence_failed());
    }
    let current = match std::fs::symlink_metadata(&locked.path) {
        Ok(metadata) => Some(super::locator::unix_identity(&metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err(persistence_failed()),
    };
    if current != locked.document.source {
        return Err(persistence_failed());
    }
    Ok(())
}
fn revalidate_mutation(
    settings: &AppSettings,
    mutation: SettingsMutation,
) -> Result<SettingsMutation, SettingsError> {
    let entry = super::descriptor::descriptor(mutation.key()).ok_or_else(persistence_failed)?;
    let candidate = serde_json::to_value(mutation.value()).map_err(|_| persistence_failed())?;
    // Membership can change after parsing; use the locked current prompts, not the caller's old context.
    let prompt_ids = settings
        .post_process_prompts
        .iter()
        .map(|prompt| prompt.id.as_str())
        .collect::<Vec<_>>();
    let context = super::validation::ValidationContext {
        prompt_ids: &prompt_ids,
        updates_allowed: !crate::settings::update_checks_forced_disabled(),
    };
    super::validation::validate_value(entry, candidate, &context)
}
/// Apply one scalar to the locked snapshot; `None` means a semantic no-op that must not be written.
fn apply_mutation(
    document: &mut Document,
    mutation: &SettingsMutation,
) -> Result<Option<(AppSettings, Revision)>, SettingsError> {
    let current =
        serde_json::to_value(&document.read.snapshot.settings).map_err(|_| persistence_failed())?;
    let mut candidate = current.clone();
    candidate[mutation.key()] =
        serde_json::to_value(mutation.value()).map_err(|_| persistence_failed())?;
    let settings: AppSettings =
        serde_json::from_value(candidate).map_err(|_| persistence_failed())?;
    let candidate = serde_json::to_value(&settings).map_err(|_| persistence_failed())?;
    if current == candidate {
        return Ok(None);
    }
    let revision = next_revision(document.read.snapshot.revision)?;
    apply_changed_fields(&mut document.raw, &current, &candidate, document.missing);
    document.raw["settings_revision"] = serde_json::json!(revision.0);
    Ok(Some((settings, revision)))
}
/// Merge startup normalization into the raw payload; `None` when the stored payload is already normal.
fn normalize_document(document: &mut Document) -> Result<Option<Revision>, SettingsError> {
    let normalized =
        serde_json::to_value(&document.read.snapshot.settings).map_err(|_| persistence_failed())?;
    let previous = document
        .raw
        .get("settings")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let preserved = normalized_payload(&previous, &normalized);
    if !document.missing && preserved == previous {
        return Ok(None);
    }
    let revision = next_revision(document.read.snapshot.revision)?;
    document.raw["settings"] = preserved;
    document.raw["settings_revision"] = serde_json::json!(revision.0);
    Ok(Some(revision))
}
fn check_revision(document: &Document, expected: Option<Revision>) -> Result<(), SettingsError> {
    if !document.writable {
        return Err(persistence_failed());
    }
    if expected.is_some_and(|revision| revision != document.read.snapshot.revision) {
        return Err(SettingsError::revision_conflict(
            document.read.snapshot.revision,
        ));
    }
    Ok(())
}
fn next_revision(current: Revision) -> Result<Revision, SettingsError> {
    current
        .0
        .checked_add(1)
        .map(Revision)
        .ok_or_else(persistence_failed)
}
