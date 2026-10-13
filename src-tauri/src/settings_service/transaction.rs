//! Lock/reload/patch transactions preserve raw fields and independent startup ownership.
pub use super::atomic::{PersistObserver, PersistStage};
pub(crate) use super::edit::SettingsEdit;
use super::{
    edit::{apply_edit, check_revision, normalize_document},
    errors::{SettingsError, SettingsErrorCode},
    ownership::{require_absent, FileLock, NativeOwnerProbe, OwnerProbe, StartupLease},
    protocol::{Revision, SettingsSnapshot},
    store::{absent_document, persistence_failed, Document, SettingsRead, SettingsRepository},
    validation::SettingsMutation,
};
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
///
/// Field order is drop order: the lease (`fence`) is released before the transaction lock, so a
/// waiter that gets the transaction lock never finds an offline writer's lease still held.
struct LockedDocument<'a, P: OwnerProbe> {
    path: PathBuf,
    fence: Fence<'a, P>,
    transaction: FileLock,
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
        let edit = SettingsEdit::Scalar(mutation);
        self.commit(&edit, expected, Authority::Offline(probe), observer)
    }
    /// Offline commit of any typed edit (a process that is not the running instance, such as a
    /// headless run); same locking, CAS, validation and errors as `commit_with_probe`.
    ///
    /// # Errors
    /// Same as `commit_with_probe`, plus any refusal returned by an `Apply` edit.
    pub(crate) fn commit_edit_with_probe(
        &self,
        edit: &SettingsEdit<'_>,
        expected: Option<Revision>,
        probe: &impl OwnerProbe,
    ) -> Result<SettingsRead, SettingsError> {
        self.commit(edit, expected, Authority::Offline(probe), &())
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
        self.commit_owned_edit(lease, &SettingsEdit::Scalar(mutation), expected)
    }
    /// Lease-holder commit of any typed edit; the edit runs on the freshly locked settings.
    ///
    /// # Errors
    /// Same as `commit_owned`, plus any refusal returned by an `Apply` edit.
    pub(crate) fn commit_owned_edit(
        &self,
        lease: &StartupLease,
        edit: &SettingsEdit<'_>,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        self.commit::<NativeOwnerProbe>(edit, expected, Authority::Owner(lease), &())
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
        edit: &SettingsEdit<'_>,
        expected: Option<Revision>,
        authority: Authority<'_, P>,
        observer: &impl PersistObserver,
    ) -> Result<SettingsRead, SettingsError> {
        if let Some(read) = self.settle_absent_store(edit, expected, &authority)? {
            return Ok(read);
        }
        let path = self.prepare_parent()?;
        let mut locked = self.lock_and_load(path, authority, observer)?;
        check_revision(&locked.document, expected)?;
        let Some((settings, revision)) = apply_edit(&mut locked.document, edit)? else {
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
        edit: &SettingsEdit<'_>,
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
        Ok(match apply_edit(&mut document, edit)? {
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
            fence,
            transaction,
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
