//! Running-process settings authority: one repository, its write authority and the published
//! snapshot that every in-process reader sees between commits.
use super::{
    edit::{SettingsEdit, SettingsEditFn},
    errors::{Outcome, SettingsError, SettingsErrorCode},
    ownership::{OwnerProbe, OwnerState, StartupLease},
    protocol::{Revision, SettingsSnapshot},
    sequence::WriteSequence,
    store::{SettingsRead, SettingsRepository},
    transaction::PersistObserver,
    validation::{validate_ui, SettingsMutation, ValidationContext},
};
use crate::settings::AppSettings;
use std::sync::{Mutex, PoisonError, RwLock};

/// Startup probe of the GUI instance. The service is installed right after Tauri's `build()`, which
/// initializes the single-instance plugin: it has either exited this process (another Handy, old
/// or new, is alive) or claimed the singleton resources for it, so the passive legacy probe could
/// only observe this process itself.
pub struct ClaimedSingleInstance;
impl OwnerProbe for ClaimedSingleInstance {
    fn inspect(&self) -> OwnerState {
        OwnerState::Absent
    }
}

/// Who commits for this process.
enum Authority {
    /// The running instance: the startup lease retained until shutdown.
    Owner(StartupLease),
    /// A process that is not the instance (headless run): an offline writer whose probe must
    /// observe no owner before every commit.
    Offline(Box<dyn OwnerProbe + Send + Sync>),
    /// The store could not be owned (link, hard link, unreadable file, unsafe lock permissions):
    /// the process runs on its best read and refuses every write with this code.
    ReadOnly(SettingsErrorCode),
}
/// Borrowed dynamic probe usable where the repository expects a sized probe.
struct ProbeRef<'a>(&'a (dyn OwnerProbe + Send + Sync));
impl OwnerProbe for ProbeRef<'_> {
    fn inspect(&self) -> OwnerState {
        self.0.inspect()
    }
}

/// Borrowed dynamic observer usable where the repository expects a sized observer.
struct ObserverRef<'a>(&'a (dyn PersistObserver + Send + Sync));
impl PersistObserver for ObserverRef<'_> {
    fn observe(&self, stage: super::atomic::PersistStage) -> std::io::Result<()> {
        self.0.observe(stage)
    }
}

/// Live settings of one process. Commits are serialized in-process, then published.
// @spec FR-003: All writers and live adapters — .specs/features/008-settings-cli/spec.md#fr-003
pub struct LiveSettings {
    repository: SettingsRepository,
    authority: Authority,
    published: RwLock<SettingsSnapshot>,
    commits: Mutex<()>,
    sequence: WriteSequence,
    /// Durable-stage observer of owned commits: inert in production, a fault port in tests.
    observer: Box<dyn PersistObserver + Send + Sync>,
    recovered: bool,
}

/// Durable result of one live commit; `changed` is empty for a semantic no-op.
pub struct LiveCommit {
    /// Published settings just before the commit (the locked state: this process is the writer).
    pub(crate) before: AppSettings,
    /// Committed settings, as every later read returns them.
    pub(crate) after: AppSettings,
    /// Revision of `after`; unchanged for a no-op.
    pub revision: Revision,
    /// Descriptor keys whose value changed, in descriptor order.
    pub changed: Vec<&'static str>,
}

impl LiveSettings {
    /// Start as the running instance: normalize once under the lock and retain the lease.
    ///
    /// # Errors
    /// `unavailable` when another instance holds the lease or `probe` sees an owner;
    /// `persistence_failed` when the store or its locks cannot be used.
    pub fn start_owned(
        repository: SettingsRepository,
        probe: &impl OwnerProbe,
    ) -> Result<Self, SettingsError> {
        let (read, lease) = repository.initialize(probe)?;
        Ok(Self::with(repository, Authority::Owner(lease), read))
    }
    /// Start as a process that is not the instance: read fresh bytes without writing; later
    /// commits are offline writes refused while `probe` or the lease shows an owner.
    ///
    /// # Errors
    /// Same as `SettingsRepository::read_snapshot`.
    pub fn start_offline(
        repository: SettingsRepository,
        probe: Box<dyn OwnerProbe + Send + Sync>,
    ) -> Result<Self, SettingsError> {
        let read = repository.read_snapshot()?;
        Ok(Self::with(repository, Authority::Offline(probe), read))
    }
    fn with(repository: SettingsRepository, authority: Authority, read: SettingsRead) -> Self {
        Self {
            repository,
            authority,
            recovered: read.warning,
            published: RwLock::new(read.snapshot),
            commits: Mutex::new(()),
            sequence: WriteSequence::default(),
            observer: Box::new(()),
        }
    }
    #[cfg(test)]
    /// Inject a durable-stage fault port into this instance's owned commits.
    pub(crate) fn with_observer(
        mut self,
        observer: Box<dyn PersistObserver + Send + Sync>,
    ) -> Self {
        self.observer = observer;
        self
    }
    /// Start as the running instance, or read-only when the store cannot be owned: a startup
    /// failure must never stop the app, only its writes.
    ///
    /// # Errors
    /// `unavailable` only when another instance holds this store's lease (a real conflict).
    pub fn start_owned_or_read_only(
        repository: SettingsRepository,
        probe: &impl OwnerProbe,
    ) -> Result<Self, SettingsError> {
        let path = repository.path.clone();
        match Self::start_owned(repository, probe) {
            Ok(live) => Ok(live),
            Err(error) if super::route::instance_running(&path).unwrap_or(false) => Err(error),
            Err(error) => Ok(Self::start_read_only(path, error.code())),
        }
    }
    /// Run on the best available read of `path` and refuse every write with `refusal`.
    pub fn start_read_only(path: std::path::PathBuf, refusal: SettingsErrorCode) -> Self {
        log::warn!("Settings store cannot be owned ({refusal:?}); settings are read-only");
        let read = super::store::read_best_effort(&path);
        Self::with(
            SettingsRepository::new(path),
            Authority::ReadOnly(refusal),
            read,
        )
    }
    /// Published snapshot (settings and revision) of the last commit or startup read.
    pub fn snapshot(&self) -> SettingsSnapshot {
        self.published
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// Published settings; runtime-only overrides are applied by callers, never stored here.
    pub(crate) fn settings(&self) -> AppSettings {
        self.snapshot().settings
    }
    /// True when startup read a salvaged or unreadable store (writes may then be refused).
    pub fn recovered(&self) -> bool {
        self.recovered
    }
    /// True when this process could not own the store and refuses every write.
    pub fn read_only(&self) -> bool {
        matches!(self.authority, Authority::ReadOnly(_))
    }
    /// Write order of this process (see `sequence`).
    pub(crate) fn sequence(&self) -> &WriteSequence {
        &self.sequence
    }
    /// Changes between `before` and the published settings, as one commit description.
    ///
    /// # Errors
    /// `internal_error` if the settings cannot be compared.
    pub(crate) fn changes_since(&self, before: &AppSettings) -> Result<LiveCommit, SettingsError> {
        let current = self.snapshot();
        Ok(LiveCommit {
            changed: changed_keys(before, &current.settings)?,
            before: before.clone(),
            after: current.settings,
            revision: current.revision,
        })
    }
    /// Validate a settings-UI value with the validator shared with the CLI, against the
    /// published settings (prompt membership) and the build policy.
    ///
    /// # Errors
    /// `not_found` or `validation_failed`, exactly as the CLI reports them.
    pub fn validate_ui_value(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<SettingsMutation, SettingsError> {
        let settings = self.settings();
        let prompt_ids = settings
            .post_process_prompts
            .iter()
            .map(|prompt| prompt.id.as_str())
            .collect::<Vec<_>>();
        let context = ValidationContext {
            prompt_ids: &prompt_ids,
            updates_allowed: !crate::settings::update_checks_forced_disabled(),
        };
        validate_ui(key, value, &context)
    }
    /// The settings-UI scalar write path: shared validation, then a commit on the latest stored
    /// settings. A refusal changes neither the disk nor the published snapshot.
    ///
    /// # Errors
    /// Validation refusals, then every refusal of the repository commit.
    pub fn commit_ui_value(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<LiveCommit, SettingsError> {
        let mutation = self.validate_ui_value(key, value)?;
        self.commit_scalar(mutation, None)
    }
    /// Commit one validated scalar (the path shared by the settings UI and the CLI).
    ///
    /// # Errors
    /// Every refusal of the repository commit; nothing is published on error.
    pub fn commit_scalar(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<LiveCommit, SettingsError> {
        self.commit(&SettingsEdit::Scalar(mutation), expected)
    }
    /// Commit one typed UI or manager edit, applied to the freshly locked settings.
    ///
    /// # Errors
    /// Every refusal of the repository commit or of the edit itself.
    pub(crate) fn commit_edit(
        &self,
        edit: SettingsEditFn<'_>,
        expected: Option<Revision>,
    ) -> Result<LiveCommit, SettingsError> {
        self.commit(&SettingsEdit::Apply(edit), expected)
    }
    /// Order matters: serialize in-process commits, take `before` from the published state,
    /// commit durably, then publish, so `before`/`changed` describe exactly this commit.
    pub(crate) fn commit(
        &self,
        edit: &SettingsEdit<'_>,
        expected: Option<Revision>,
    ) -> Result<LiveCommit, SettingsError> {
        let _serial = self.commits.lock().unwrap_or_else(PoisonError::into_inner);
        let before = self.settings();
        let read = self.commit_durably(edit, expected).inspect_err(|error| {
            if error.outcome() == Some(Outcome::Unknown) {
                self.refresh();
            }
        })?;
        let revision = read.snapshot.revision;
        let after = read.snapshot.settings.clone();
        // Order matters: the commit is durable, so it is published before anything else can fail.
        self.publish(read.snapshot);
        let changed = changed_keys(&before, &after)
            .map_err(|_| SettingsError::committed_effect_failed(revision, ""))?;
        Ok(LiveCommit {
            before,
            after,
            revision,
            changed,
        })
    }
    fn commit_durably(
        &self,
        edit: &SettingsEdit<'_>,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        match &self.authority {
            Authority::Owner(lease) => self.repository.commit_owned_observed(
                lease,
                edit,
                expected,
                &ObserverRef(self.observer.as_ref()),
            ),
            Authority::Offline(probe) => {
                self.repository
                    .commit_edit_with_probe(edit, expected, &ProbeRef(probe.as_ref()))
            }
            Authority::ReadOnly(code) => Err(SettingsError::new(*code)),
        }
    }
    /// After an unconfirmed replacement the disk may hold the new generation: reread it. A failed
    /// reread keeps the previous snapshot; the caller already reports the unknown outcome.
    fn refresh(&self) {
        if let Ok(read) = self.repository.read_snapshot() {
            self.publish(read.snapshot);
        }
    }
    /// Never move the published revision backwards.
    fn publish(&self, snapshot: SettingsSnapshot) {
        let mut published = self
            .published
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if snapshot.revision.0 >= published.revision.0 {
            *published = snapshot;
        }
    }
}
/// Descriptor keys whose serialized value differs between two settings.
fn changed_keys(
    before: &AppSettings,
    after: &AppSettings,
) -> Result<Vec<&'static str>, SettingsError> {
    let internal = || SettingsError::new(SettingsErrorCode::InternalError);
    let before = serde_json::to_value(before).map_err(|_| internal())?;
    let after = serde_json::to_value(after).map_err(|_| internal())?;
    Ok(super::descriptor::descriptors()
        .iter()
        .filter(|entry| before.get(entry.key) != after.get(entry.key))
        .map(|entry| entry.key)
        .collect())
}
/// Backend over the service alone (no live effects, no write order): used by channel tests. The
/// running app routes CLI operations through the Tauri adapter, which adds both.
impl super::operations::SettingsBackend for LiveSettings {
    fn read(&self) -> Result<SettingsRead, SettingsError> {
        Ok(SettingsRead {
            snapshot: self.snapshot(),
            warning: self.recovered,
        })
    }
    fn commit(
        &self,
        mutation: SettingsMutation,
        expected: Option<Revision>,
    ) -> Result<SettingsRead, SettingsError> {
        let commit = self.commit_scalar(mutation, expected)?;
        Ok(SettingsRead {
            snapshot: SettingsSnapshot {
                settings: commit.after,
                revision: commit.revision,
            },
            warning: self.recovered,
        })
    }
}
#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
