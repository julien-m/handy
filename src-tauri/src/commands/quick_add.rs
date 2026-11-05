//! Commands of the Quick Add window (018): append one replacement rule or one vocabulary term
//! through the settings service, with the same validator, commit path and typed refusals as the
//! settings editors (002, 003) and `handy settings`. A quick addition commits at the current
//! revision: an append is checked under the service lock, so it never overwrites a concurrent
//! change.
use crate::quick_add::{reveal_in_settings, QuickAddKind};
use crate::recognition_vocabulary::edits::VocabularyEdit;
use crate::settings::AppSettings;
use crate::settings_service::{
    command_error::SettingsCommandError, domains::replacements::refusal, errors::SettingsError,
    tauri_adapter,
};
use crate::word_replacements::{
    model::{RuleId, RuleIdSource, UuidRuleIdSource},
    service::add_rule,
};
use serde::Serialize;
use specta::Type;
use tauri::AppHandle;

/// A saved quick addition: how many items its list now holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
pub struct QuickAdded {
    pub count: u32,
}

fn added(count: usize) -> QuickAdded {
    QuickAdded {
        count: u32::try_from(count).unwrap_or(u32::MAX),
    }
}

/// Edit appending the rule `id` after the saved rules, refused by the shared 002 validator.
fn append_rule_edit(
    id: RuleId,
    recognized: String,
    replacement: String,
) -> impl Fn(&mut AppSettings) -> Result<(), SettingsError> {
    move |settings| {
        settings.word_replacements = add_rule(
            &settings.word_replacements,
            id.clone(),
            &recognized,
            &replacement,
        )
        .map_err(refusal)?;
        Ok(())
    }
}

// @spec FR-003: Append one rule in one commit — .specs/features/018-quick-add-from-tray/spec.md#fr-003
// @spec AC-004: Rule appended with a new identity — .specs/features/018-quick-add-from-tray/spec.md#ac-004
// @spec AC-005: Refused rule saves nothing — .specs/features/018-quick-add-from-tray/spec.md#ac-005
#[tauri::command]
#[specta::specta]
/// Append the rule `recognized` → `replacement` after the saved rules.
///
/// # Errors
/// A `SettingsCommandError`, nothing saved: `validation_failed` with the 002 reasons of the
/// refused fields (`invalid_items`), and every refusal of the settings service.
pub async fn quick_add_replacement(
    app: AppHandle,
    recognized: String,
    replacement: String,
) -> Result<QuickAdded, SettingsCommandError> {
    let edit = append_rule_edit(UuidRuleIdSource.next_id(), recognized, replacement);
    tauri_adapter::blocking(move || {
        let commit = tauri_adapter::edit_at(&app, &edit, None)?;
        Ok(added(commit.after.word_replacements.len()))
    })
    .await
}

// @spec FR-004: Append one term through VocabularyEdit::Add — .specs/features/018-quick-add-from-tray/spec.md#fr-004
// @spec AC-006: Term added or refused — .specs/features/018-quick-add-from-tray/spec.md#ac-006
#[tauri::command]
#[specta::specta]
/// Append `term` to the recognition vocabulary.
///
/// # Errors
/// A `SettingsCommandError`, nothing saved: the 003 refusals (invalid or duplicate term, list
/// limit) and every refusal of the settings service.
pub async fn quick_add_vocabulary_term(
    app: AppHandle,
    term: String,
) -> Result<QuickAdded, SettingsCommandError> {
    tauri_adapter::blocking(move || {
        let change = VocabularyEdit::Add(term).into_change();
        let commit = tauri_adapter::write_change(&app, &change, None)?;
        Ok(added(commit.after.custom_words.len()))
    })
    .await
}

#[tauri::command]
#[specta::specta]
/// Open the settings window on the list of `kind` (Advanced section), from the Quick Add window.
pub fn open_settings_list(app: AppHandle, kind: QuickAddKind) {
    reveal_in_settings(&app, kind);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::get_default_settings;
    use crate::settings_service::errors::SettingsErrorCode;
    use crate::word_replacements::model::ReplacementRule;

    fn rule(id: &str, recognized: &str) -> ReplacementRule {
        ReplacementRule {
            id: RuleId::from(id),
            recognized: recognized.to_owned(),
            replacement: "x".to_owned(),
        }
    }

    #[test]
    fn a_valid_rule_is_appended_after_the_saved_rules() -> Result<(), SettingsError> {
        let mut settings = get_default_settings();
        settings.word_replacements = vec![rule("a", "first")];
        let id = UuidRuleIdSource.next_id();
        append_rule_edit(id.clone(), "cloud code".into(), "Claude Code".into())(&mut settings)?;
        let last = settings.word_replacements.last().cloned();
        assert_eq!(settings.word_replacements.len(), 2);
        assert_eq!(settings.word_replacements[0], rule("a", "first"));
        assert_eq!(
            last,
            Some(ReplacementRule {
                id,
                recognized: "cloud code".into(),
                replacement: "Claude Code".into(),
            })
        );
        Ok(())
    }

    #[test]
    fn a_duplicate_heard_text_is_refused_and_saves_nothing() {
        let mut settings = get_default_settings();
        settings.word_replacements = vec![rule("a", "cloud code")];
        let before = settings.word_replacements.clone();
        let edit = append_rule_edit(UuidRuleIdSource.next_id(), "Cloud Code".into(), "y".into());
        let error = edit(&mut settings).err();
        assert_eq!(
            error.map(|e| e.code()),
            Some(SettingsErrorCode::ValidationFailed)
        );
        assert_eq!(settings.word_replacements, before);
    }

    #[test]
    fn an_empty_field_is_refused() {
        let mut settings = get_default_settings();
        let edit = append_rule_edit(UuidRuleIdSource.next_id(), "  ".into(), "y".into());
        assert!(edit(&mut settings).is_err());
        assert!(settings.word_replacements.is_empty());
    }
}
