//! Typed edits applied to the freshly locked settings, with shared validation and normalization.
use super::{
    errors::{SettingsError, ValidationReason},
    protocol::Revision,
    raw_merge::{apply_changed_fields, normalized_payload},
    store::{persistence_failed, Document},
    validation::SettingsMutation,
};
use crate::settings::AppSettings;

/// Typed edit callback: it receives the locked current settings and may refuse with a typed error.
pub(crate) type SettingsEditFn<'a> = &'a dyn Fn(&mut AppSettings) -> Result<(), SettingsError>;

/// One change applied under the transaction lock to the latest stored settings, never to a
/// caller's stale snapshot.
pub(crate) enum SettingsEdit<'a> {
    /// A validated scalar shared by the UI and the CLI; it is revalidated against the locked state.
    Scalar(SettingsMutation),
    /// A UI or manager edit of non-scalar or UI-owned fields; it validates its own input against
    /// the locked state. It may run twice (absent-store pre-check, then locked), so it must be pure.
    Apply(SettingsEditFn<'a>),
}

/// Apply `edit` to the locked snapshot; `None` means a semantic no-op that must not be written.
///
/// Every candidate passes the shared cross-field rules and must be a fixed point of read
/// normalization. The complete normalization is persisted with the change under one revision:
/// writing only the changed fields would leave legacy migration markers stale, and the next read
/// would replay the migrations over the committed value.
pub(super) fn apply_edit(
    document: &mut Document,
    edit: &SettingsEdit<'_>,
) -> Result<Option<(AppSettings, Revision)>, SettingsError> {
    let before = &document.read.snapshot.settings;
    let settings = preview(before, edit)?;
    let current = serde_json::to_value(before).map_err(|_| persistence_failed())?;
    let candidate = serde_json::to_value(&settings).map_err(|_| persistence_failed())?;
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
/// Settings `edit` would commit on `before`, with every rule of the commit (shared cross-field
/// rules, read-normalization fixed point) but without writing: the dry-run path and the first
/// half of `apply_edit`.
///
/// # Errors
/// The edit's own refusal, `validation_failed` for a cross-field rule or a value read
/// normalization would replace.
pub(crate) fn preview(
    before: &AppSettings,
    edit: &SettingsEdit<'_>,
) -> Result<AppSettings, SettingsError> {
    let settings = edited_settings(before, edit)?;
    super::validation::validate_dependencies(before, &settings)?;
    let current = serde_json::to_value(before).map_err(|_| persistence_failed())?;
    let candidate = serde_json::to_value(&settings).map_err(|_| persistence_failed())?;
    require_stable(&current, &candidate)?;
    Ok(settings)
}
/// Candidate settings produced by the edit from the locked `before`.
pub(super) fn edited_settings(
    before: &AppSettings,
    edit: &SettingsEdit<'_>,
) -> Result<AppSettings, SettingsError> {
    match edit {
        SettingsEdit::Scalar(mutation) => {
            let mutation = revalidate_mutation(before, mutation)?;
            let mut candidate = serde_json::to_value(before).map_err(|_| persistence_failed())?;
            candidate[mutation.key()] =
                serde_json::to_value(mutation.value()).map_err(|_| persistence_failed())?;
            serde_json::from_value(candidate).map_err(|_| persistence_failed())
        }
        SettingsEdit::Apply(apply) => {
            let mut candidate = before.clone();
            apply(&mut candidate)?;
            Ok(candidate)
        }
    }
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
/// The committed candidate must be a fixed point of read normalization; otherwise the next read
/// would rewrite the accepted value and the returned snapshot would differ from the reread.
///
/// # Errors
/// `validation_failed` naming the field normalization would change, preferring one this edit
/// changed (so a multi-field edit names the user's field, not an unrelated one).
fn require_stable(
    current: &serde_json::Value,
    candidate: &serde_json::Value,
) -> Result<(), SettingsError> {
    let (normalized, _) = super::normalization::normalize(candidate)?;
    let normalized = serde_json::to_value(&normalized).map_err(|_| persistence_failed())?;
    let entries = super::descriptor::descriptors();
    let diverges = |entry: &&super::descriptor::SettingDescriptor| {
        candidate.get(entry.key) != normalized.get(entry.key)
    };
    let edited = |entry: &&super::descriptor::SettingDescriptor| {
        current.get(entry.key) != candidate.get(entry.key)
    };
    match entries
        .iter()
        .filter(diverges)
        .find(edited)
        .or_else(|| entries.iter().find(diverges))
    {
        Some(entry) => Err(SettingsError::validation(
            entry.key,
            ValidationReason::CanonicalValue,
        )),
        None => Ok(()),
    }
}
/// Merge startup normalization into the raw payload; `None` when the stored payload is already normal.
pub(super) fn normalize_document(
    document: &mut Document,
) -> Result<Option<Revision>, SettingsError> {
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
/// Refuse a non-writable (recovered) document and a stale expected revision.
pub(super) fn check_revision(
    document: &Document,
    expected: Option<Revision>,
) -> Result<(), SettingsError> {
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
