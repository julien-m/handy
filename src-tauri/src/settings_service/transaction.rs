//! Lock/reload/patch transactions preserve raw fields and independent startup ownership.
pub use super::atomic::{PersistObserver, PersistStage};
use super::{
    errors::{SettingsError, SettingsErrorCode, ValidationReason},
    ownership::{require_absent, FileLock, NativeOwnerProbe, OwnerProbe, StartupLease},
    protocol::{Revision, SettingsSnapshot},
    raw_merge::{apply_changed_fields, normalized_payload},
    store::{absent_document, persistence_failed, Document, SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
use crate::settings::AppSettings;
use std::path::{Path, PathBuf};

const TRANSACTION_LOCK: &str = "settings.lock";
const INSTANCE_LOCK: &str = "settings.instance.lock";

/// Who may write: an offline writer proves that no instance owns the store, while the running
/// instance proves that it still holds the startup lease acquired by `initialize`.
enum Authority<'a, P: OwnerProbe> {
    Offline(&'a P),
    Owner(&'a StartupLease),
}
/// Ownership evidence rechecked before the temp exists and again right before replacement.
enum Fence<'a, P: OwnerProbe> {
    /// Offline: the lease is held for this transaction only and the passive probe must stay absent.
    Offline { lease: FileLock, probe: &'a P },
    /// Running instance: its retained lease is revalidated. No passive probe runs, because the
    /// instance's own single-instance resources would look like an owner.
    Owner(&'a FileLock),
}
impl<P: OwnerProbe> Fence<'_, P> {
    fn check(&self) -> Result<(), SettingsError> {
        match self {
            Self::Offline { lease, probe } => {
                lease.validate()?;
                require_absent(*probe)
            }
            Self::Owner(lease) => lease.validate(),
        }
    }
}
/// Locks and the fresh generation held for the whole transaction.
struct LockedDocument<'a, P: OwnerProbe> {
    path: PathBuf,
    transaction: FileLock,
    fence: Fence<'a, P>,
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
    /// Offline commit with injectable OS probe/fault ports over the actual durable replacement.
    ///
    /// Side effects: an absent store is checked against defaults first, so a refused or no-op
    /// commit creates no directory, lock or store; a changed commit creates the private parent and
    /// the sibling lock files. Against an existing store, the lock files may be created (as reads do).
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
        self.commit(mutation, expected, Authority::Offline(probe), observer)
    }
    /// Commit as the running instance that holds `lease` from `initialize`, under the same
    /// transaction lock, fresh re-read, CAS and validation as offline writers.
    ///
    /// # Errors
    /// Same as `commit_with_probe`; `unavailable` when `lease` belongs to another store and
    /// `persistence_failed` when the lease pathname no longer names the held lock.
    pub fn commit_owned(
        &self,
        lease: &StartupLease,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        self.commit::<NativeOwnerProbe>(mutation, expected, Authority::Owner(lease), &())
    }
    /// Startup normalizes explicitly under the transaction lock and retains a separate lease until
    /// shutdown. An unreadable document or malformed revision is returned as a read-only recovery
    /// (defaults or salvage with `warning`) without writing, and the lease is still retained.
    // @spec FR-004: Startup/offline ownership fence — .specs/features/008-settings-cli/spec.md#fr-004
    pub fn initialize(
        &self,
        probe: &impl OwnerProbe,
    ) -> Result<(SettingsRead, StartupLease), SettingsError> {
        let path = self.prepare_parent()?;
        let mut locked = self.lock_and_load(path, Authority::Offline(probe), &())?;
        if locked.document.writable {
            if let Some(revision) = normalize_document(&mut locked.document)? {
                self.persist_locked(&locked, &())?;
                locked.document.read.snapshot.revision = revision;
            }
        }
        let Fence::Offline { lease, .. } = locked.fence else {
            return Err(persistence_failed());
        };
        Ok((locked.document.read, StartupLease { lock: lease }))
    }
    fn commit<P: OwnerProbe>(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
        authority: Authority<'_, P>,
        observer: &impl PersistObserver,
    ) -> Result<SettingsRead, SettingsError> {
        if let Some(read) = self.settle_absent_store(&mutation, expected, &authority)? {
            return Ok(read);
        }
        let path = self.prepare_parent()?;
        let mut locked = self.lock_and_load(path, authority, observer)?;
        check_revision(&locked.document, expected)?;
        let mutation = revalidate_mutation(&locked.document.read.snapshot.settings, &mutation)?;
        let Some((settings, revision)) = apply_mutation(&mut locked.document, &mutation)? else {
            return Ok(locked.document.read);
        };
        self.persist_locked(&locked, observer)?;
        Ok(SettingsRead {
            snapshot: SettingsSnapshot { settings, revision },
            warning: locked.document.read.warning,
        })
    }
    /// Absent store: decide refusals and no-ops against the defaults before creating anything.
    /// `None` means the commit changes something (or the store exists) and must run locked.
    fn settle_absent_store<P: OwnerProbe>(
        &self,
        mutation: &SettingsMutation,
        expected: Option<Revision>,
        authority: &Authority<'_, P>,
    ) -> Result<Option<SettingsRead>, SettingsError> {
        let path = super::locator::canonical_path(&self.path)?;
        if std::fs::symlink_metadata(&path).is_ok() {
            return Ok(None);
        }
        verify_absent_store_authority(&path, authority)?;
        let mut document = absent_document();
        check_revision(&document, expected)?;
        let mutation = revalidate_mutation(&document.read.snapshot.settings, mutation)?;
        Ok(match apply_mutation(&mut document, &mutation)? {
            None => Some(document.read),
            Some(_) => None,
        })
    }
    /// Order matters: transaction lock, parent identity, startup lease and ownership evidence,
    /// then the fresh read, so the read always belongs to the locked parent generation.
    fn lock_and_load<'a, P: OwnerProbe>(
        &self,
        path: PathBuf,
        authority: Authority<'a, P>,
        observer: &impl PersistObserver,
    ) -> Result<LockedDocument<'a, P>, SettingsError> {
        let transaction = FileLock::acquire(path.with_file_name(TRANSACTION_LOCK), true)?;
        #[cfg(unix)]
        let parent = parent_identity(&path)?;
        let fence = establish_fence(&path, authority)?;
        fence.check()?;
        let document = self.load_at(&path)?;
        observer
            .observe(PersistStage::AfterLoad)
            .map_err(|_| persistence_failed())?;
        Ok(LockedDocument {
            path,
            transaction,
            fence,
            #[cfg(unix)]
            parent,
            document,
        })
    }
    /// Replace the generation only while every identity captured under the lock still holds.
    fn persist_locked<P: OwnerProbe>(
        &self,
        locked: &LockedDocument<'_, P>,
        observer: &impl PersistObserver,
    ) -> Result<(), SettingsError> {
        super::atomic::persist(&locked.path, &locked.document.raw, observer, || {
            self.authorize_replace(locked)
        })
    }
    /// Runs before the temp exists and again after its fsync, immediately before replacement.
    fn authorize_replace<P: OwnerProbe>(
        &self,
        locked: &LockedDocument<'_, P>,
    ) -> Result<(), SettingsError> {
        locked.transaction.validate()?;
        if super::locator::canonical_path(&self.path)? != locked.path {
            return Err(persistence_failed());
        }
        #[cfg(unix)]
        verify_origin(locked)?;
        // The ownership evidence (lease and passive probe) is the last check before replacement.
        locked.fence.check()
    }
    /// Create the private parent for a changed commit or startup; reads never call this function.
    fn prepare_parent(&self) -> Result<PathBuf, SettingsError> {
        let path = super::locator::canonical_path(&self.path)?;
        let parent = path.parent().ok_or_else(persistence_failed)?;
        if !parent.exists() {
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
/// Same authority as a locked commit, without creating anything: the owner's lease must be this
/// store's valid instance lock; an offline writer needs a passive absence and, when the instance
/// lock already exists, must be able to take it (an instance running on a deleted store refuses).
fn verify_absent_store_authority<P: OwnerProbe>(
    path: &Path,
    authority: &Authority<'_, P>,
) -> Result<(), SettingsError> {
    let lease_path = path.with_file_name(INSTANCE_LOCK);
    match authority {
        Authority::Owner(lease) if lease.lock.path() == lease_path => lease.validate(),
        Authority::Owner(_) => Err(SettingsError::new(SettingsErrorCode::Unavailable)),
        Authority::Offline(probe) => {
            require_absent(*probe)?;
            if std::fs::symlink_metadata(&lease_path).is_ok() {
                // Taken and released at once; a lock file that exists is never created here.
                FileLock::acquire(lease_path, false)?;
            }
            Ok(())
        }
    }
}
/// Offline writers take the lease nonblocking (a running instance makes them `unavailable`); the
/// owner's retained lease must be this store's instance lock.
fn establish_fence<'a, P: OwnerProbe>(
    path: &Path,
    authority: Authority<'a, P>,
) -> Result<Fence<'a, P>, SettingsError> {
    let lease_path = path.with_file_name(INSTANCE_LOCK);
    match authority {
        Authority::Offline(probe) => Ok(Fence::Offline {
            lease: FileLock::acquire(lease_path, false)?,
            probe,
        }),
        Authority::Owner(lease) if lease.lock.path() == lease_path => Ok(Fence::Owner(&lease.lock)),
        Authority::Owner(_) => Err(SettingsError::new(SettingsErrorCode::Unavailable)),
    }
}
#[cfg(unix)]
fn parent_identity(path: &Path) -> Result<(u64, u64), SettingsError> {
    let parent = path.parent().ok_or_else(persistence_failed)?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| persistence_failed())?;
    Ok(super::locator::unix_identity(&metadata))
}
/// Detect (not prevent) a same-user rename-and-restore of the parent during the transaction: the
/// parent must be the directory locked earlier and the store pathname must still name the exact
/// file that was read (or still be absent). Windows pins the parent through the lock handle instead.
#[cfg(unix)]
fn verify_origin<P: OwnerProbe>(locked: &LockedDocument<'_, P>) -> Result<(), SettingsError> {
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
    mutation: &SettingsMutation,
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
///
/// The complete normalization is persisted with the change under one revision: writing only the
/// changed field would leave legacy migration markers stale, and the next read would replay the
/// migrations over the committed value (lost write or unrequested changes to other fields).
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
    super::validation::validate_dependencies(&document.read.snapshot.settings, &settings)?;
    let candidate = serde_json::to_value(&settings).map_err(|_| persistence_failed())?;
    require_stable(&candidate)?;
    if current == candidate {
        return Ok(None);
    }
    let revision = next_revision(document.read.snapshot.revision)?;
    let mut payload = normalized_payload(&stored_payload(document), &current);
    apply_changed_fields(&mut payload, &current, &candidate);
    document.raw["settings"] = payload;
    document.raw["settings_revision"] = serde_json::json!(revision.0);
    Ok(Some((settings, revision)))
}
/// The committed candidate must be a fixed point of read normalization; otherwise the next read
/// would rewrite the accepted value and the returned snapshot would differ from the reread.
///
/// # Errors
/// `validation_failed` naming the first field that normalization would change.
fn require_stable(candidate: &serde_json::Value) -> Result<(), SettingsError> {
    let (normalized, _) = super::normalization::normalize(candidate)?;
    let normalized = serde_json::to_value(&normalized).map_err(|_| persistence_failed())?;
    match super::descriptor::descriptors()
        .iter()
        .find(|entry| candidate.get(entry.key) != normalized.get(entry.key))
    {
        Some(entry) => Err(SettingsError::validation(
            entry.key,
            ValidationReason::CanonicalValue,
        )),
        None => Ok(()),
    }
}
/// Merge startup normalization into the raw payload; `None` when the stored payload is already normal.
fn normalize_document(document: &mut Document) -> Result<Option<Revision>, SettingsError> {
    let normalized =
        serde_json::to_value(&document.read.snapshot.settings).map_err(|_| persistence_failed())?;
    let previous = stored_payload(document);
    let preserved = normalized_payload(&previous, &normalized);
    if preserved == previous {
        return Ok(None);
    }
    let revision = next_revision(document.read.snapshot.revision)?;
    document.raw["settings"] = preserved;
    document.raw["settings_revision"] = serde_json::json!(revision.0);
    Ok(Some(revision))
}
/// Stored `settings` payload; an absent store or a store without payload starts from `{}`.
fn stored_payload(document: &Document) -> serde_json::Value {
    document
        .raw
        .get("settings")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}))
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
