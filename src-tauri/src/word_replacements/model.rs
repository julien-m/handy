//! Ordered replacement content and opaque identities, without settings services.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Stable opaque rule identity; exchange files carry content rather than IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct RuleId(String);

impl RuleId {
    /// Borrow the opaque value for API routing; its format is not business data.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RuleId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for RuleId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// @spec FR-001: Ordered stable replacement rules — .specs/features/002-word-replacements/spec.md#fr-001
/// Persisted content; normalized keys and positions are derived from the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ReplacementRule {
    pub id: RuleId,
    pub recognized: String,
    pub replacement: String,
}

/// Injected identity allocation port; previews need not allocate durable IDs.
pub trait RuleIdSource {
    /// Allocate an identity for a newly committed rule.
    fn next_id(&mut self) -> RuleId;
}

/// Production UUID-v4 allocation, isolated from deterministic domain tests.
#[derive(Default)]
pub struct UuidRuleIdSource;

impl RuleIdSource for UuidRuleIdSource {
    fn next_id(&mut self) -> RuleId {
        RuleId(uuid::Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserve_opaque_ids_and_order_through_json() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            ReplacementRule {
                id: "not-a-uuid".into(),
                recognized: "first".into(),
                replacement: "one".into(),
            },
            ReplacementRule {
                id: "".into(),
                recognized: "second".into(),
                replacement: "two".into(),
            },
        ];
        let json = serde_json::to_string(&rules)?;
        assert_eq!(serde_json::from_str::<Vec<ReplacementRule>>(&json)?, rules);
        Ok(())
    }

    #[test]
    fn allocate_distinct_uuid_v4_ids() -> Result<(), Box<dyn std::error::Error>> {
        let mut source = UuidRuleIdSource;
        let first = source.next_id();
        let second = source.next_id();
        assert_ne!(first, second);
        assert_eq!(uuid::Uuid::parse_str(first.as_str())?.get_version_num(), 4);
        Ok(())
    }
}
