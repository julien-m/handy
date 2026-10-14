//! Compiled rules of the committed settings: one immutable matcher per settings revision, shared
//! by every dictation that starts at that revision. The 008 settings service publishes the
//! settings snapshot; this module only compiles its rules once. A commit that changes the rules
//! compiles them in its live effect; a reader that finds an older revision compiles the current
//! one itself. The matcher is thus always derived from the published snapshot, never a second
//! owner of the rules.

use std::sync::{Arc, Mutex, PoisonError};

use tauri::{AppHandle, Manager};

use super::{
    dictation::compile_for_dictation, matcher::CompiledReplacements, model::ReplacementRule,
};

/// The matcher of the newest settings revision compiled so far.
#[derive(Default)]
pub struct PublishedReplacements {
    newest: Mutex<Option<(u64, Arc<CompiledReplacements>)>>,
}

impl PublishedReplacements {
    // @spec AC-007: Captured rules apply to their dictation — .specs/features/002-word-replacements/spec.md#ac-007
    /// Matcher of `rules` read at settings `revision`: the published one when it was compiled
    /// from this revision, otherwise compiled now (and published unless a newer revision is).
    pub fn matcher_for(
        &self,
        revision: u64,
        rules: &[ReplacementRule],
    ) -> Arc<CompiledReplacements> {
        // Held while compiling, so concurrent readers of a new revision compile it once.
        let mut newest = self.newest.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((published, matcher)) = newest.as_ref() {
            if *published == revision {
                return Arc::clone(matcher);
            }
        }
        let matcher = Arc::new(compile_for_dictation(rules));
        // Never move backwards: a late reader of an older revision keeps its matcher to itself.
        if newest
            .as_ref()
            .is_none_or(|(published, _)| *published < revision)
        {
            *newest = Some((revision, Arc::clone(&matcher)));
        }
        matcher
    }
}

/// Matcher of the settings published to this process now (a dictation starts or a saved
/// recording is transcribed again). Without the publication state (a headless run), the rules of
/// the published settings are compiled for this caller alone.
pub fn current_matcher(app: &AppHandle) -> Arc<CompiledReplacements> {
    let (revision, settings) = crate::settings_service::tauri_adapter::view(app);
    match app.try_state::<PublishedReplacements>() {
        Some(published) => published.matcher_for(revision.0, &settings.word_replacements),
        None => Arc::new(compile_for_dictation(&settings.word_replacements)),
    }
}

/// Live effect of a commit that changed the rules: compile them for the next dictation now.
pub fn publish_committed(app: &AppHandle, revision: u64, rules: &[ReplacementRule]) {
    if let Some(published) = app.try_state::<PublishedReplacements>() {
        published.matcher_for(revision, rules);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(replacement: &str) -> Vec<ReplacementRule> {
        vec![ReplacementRule {
            id: "a".into(),
            recognized: "cloud code".into(),
            replacement: replacement.into(),
        }]
    }

    #[test]
    fn one_matcher_per_revision_is_shared_by_its_readers() {
        let published = PublishedReplacements::default();
        let first = published.matcher_for(3, &rules("Claude Code"));
        let again = published.matcher_for(3, &rules("ignored: same revision"));
        assert!(Arc::ptr_eq(&first, &again));
        let next = published.matcher_for(4, &rules("Claude Code CLI"));
        assert_eq!(next.apply("cloud code").text(), "Claude Code CLI");
        assert_eq!(first.apply("cloud code").text(), "Claude Code");
    }

    #[test]
    fn a_late_reader_of_an_older_revision_never_replaces_the_newest() {
        let published = PublishedReplacements::default();
        let newest = published.matcher_for(5, &rules("new"));
        let older = published.matcher_for(4, &rules("old"));
        assert_eq!(older.apply("cloud code").text(), "old");
        assert!(Arc::ptr_eq(&newest, &published.matcher_for(5, &rules("x"))));
    }

    #[test]
    fn an_invalid_stored_set_disables_replacement_without_failing() {
        let published = PublishedReplacements::default();
        let mut damaged = rules("one");
        damaged.extend(rules("two"));
        let matcher = published.matcher_for(1, &damaged);
        assert!(matcher.is_empty());
    }
}
