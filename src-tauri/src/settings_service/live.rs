//! Running-process settings authority: one repository, its write authority and the published
//! snapshot that every in-process reader sees between commits.
use super::{
    edit::{SettingsEdit, SettingsEditFn},
    errors::{Outcome, SettingsError},
    ownership::{OwnerProbe, OwnerState, StartupLease},
    protocol::{Revision, SettingsSnapshot},
    store::{SettingsRead, SettingsRepository},
    validation::{validate_ui, SettingsMutation, ValidationContext},
};
use crate::settings::AppSettings;
use std::sync::{Mutex, PoisonError, RwLock};

/// Startup probe of the GUI instance. Tauri runs `setup` only after the single-instance plugin has
/// either exited this process (another Handy, old or new, is alive) or claimed the singleton
/// resources for it, so the passive legacy probe could only observe this process itself.
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
}
/// Borrowed dynamic probe usable where the repository expects a sized probe.
struct ProbeRef<'a>(&'a (dyn OwnerProbe + Send + Sync));
impl OwnerProbe for ProbeRef<'_> {
    fn inspect(&self) -> OwnerState {
        self.0.inspect()
    }
}

/// Live settings of one process. Commits are serialized in-process, then published.
// @spec FR-003: All writers and live adapters — .specs/features/008-settings-cli/spec.md#fr-003
pub struct LiveSettings {
    repository: SettingsRepository,
    authority: Authority,
    published: RwLock<SettingsSnapshot>,
    commits: Mutex<()>,
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
        }
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
    fn commit(
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
        let changed = changed_keys(&before, &read.snapshot.settings)?;
        let revision = read.snapshot.revision;
        let after = read.snapshot.settings.clone();
        self.publish(read.snapshot);
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
            Authority::Owner(lease) => self.repository.commit_owned_edit(lease, edit, expected),
            Authority::Offline(probe) => {
                self.repository
                    .commit_edit_with_probe(edit, expected, &ProbeRef(probe.as_ref()))
            }
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
    let internal = || SettingsError::new(super::errors::SettingsErrorCode::InternalError);
    let before = serde_json::to_value(before).map_err(|_| internal())?;
    let after = serde_json::to_value(after).map_err(|_| internal())?;
    Ok(super::descriptor::descriptors()
        .iter()
        .filter(|entry| before.get(entry.key) != after.get(entry.key))
        .map(|entry| entry.key)
        .collect())
}
/// The running instance answers routed CLI operations from its published snapshot and commits
/// them through the same service as the settings UI. Live effects are added by the Tauri adapter.
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
