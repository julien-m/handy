//! Field fingerprint (feature 013, spec AC-002 and AC-006): what identifies a text field across
//! sessions of an application, built from the accessibility facts of one element, and the match
//! rule of the search. Pure: the facts are read by the platform adapter (`tree`).
use super::model::{FieldFingerprint, MAX_TEXT_CHARS};

/// Password fields, as a role or a subrole (010's rule).
pub const SECURE_TEXT_FIELD: &str = "AXSecureTextField";
/// Longest DOM id kept: longer ids are framework-generated paths, not names.
const MAX_STABLE_DOM_ID_CHARS: usize = 64;

/// Identity facts of one accessibility element; never its value or selected text.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NodeFacts {
    pub role: Option<String>,
    pub subrole: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    pub dom_id: Option<String>,
}
impl NodeFacts {
    /// True for a password field (role or subrole `AXSecureTextField`).
    pub fn is_secure(&self) -> bool {
        self.role.as_deref() == Some(SECURE_TEXT_FIELD)
            || self.subrole.as_deref() == Some(SECURE_TEXT_FIELD)
    }
    /// The accessible name: `AXTitle`, else `AXDescription`.
    pub fn name(&self) -> Option<&str> {
        self.title.as_deref().or(self.description.as_deref())
    }
}

/// Why a focused element cannot become a fingerprint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FingerprintRefusal {
    /// Password field: never remembered (spec AC-010).
    Password,
    /// No name, placeholder or stable DOM id: it could not be told apart later.
    Unrecognizable,
}

/// Blank values and values longer than the stored bound count as absent (spec AC-002).
fn kept(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= MAX_TEXT_CHARS)
        .map(str::to_owned)
}

// @spec AC-002: Generated DOM ids are never kept — .specs/features/013-default-input-per-app/spec.md#ac-002
/// Business rule (spec decision `generated_dom_id`): a DOM id looks generated when it is longer
/// than 64 characters, holds a `:` (React `useId`), or has a segment, split on `-`, `_` or `.`,
/// that mixes letters and digits or holds three digits in a row (CSS-module hashes, UUIDs,
/// counters such as `ember123`).
pub fn looks_generated(dom_id: &str) -> bool {
    if dom_id.chars().count() > MAX_STABLE_DOM_ID_CHARS || dom_id.contains(':') {
        return true;
    }
    dom_id
        .split(['-', '_', '.'])
        .any(|segment| mixes_letters_and_digits(segment) || has_digit_run(segment, 3))
}

fn mixes_letters_and_digits(segment: &str) -> bool {
    segment.chars().any(|c| c.is_ascii_digit()) && segment.chars().any(char::is_alphabetic)
}

fn has_digit_run(segment: &str, length: usize) -> bool {
    let mut run = 0;
    for character in segment.chars() {
        run = if character.is_ascii_digit() {
            run + 1
        } else {
            0
        };
        // Stop at the first run long enough: the segment is already generated.
        if run >= length {
            return true;
        }
    }
    false
}

// @spec FR-002: Fingerprint without content — .specs/features/013-default-input-per-app/spec.md#fr-002
/// The fingerprint of an element already classified editable, or why it cannot be one.
///
/// # Errors
/// `Password` for a password field, `Unrecognizable` without a role or a distinctive attribute.
pub fn fingerprint_of(facts: &NodeFacts) -> Result<FieldFingerprint, FingerprintRefusal> {
    if facts.is_secure() {
        return Err(FingerprintRefusal::Password);
    }
    let role = kept(facts.role.as_deref()).ok_or(FingerprintRefusal::Unrecognizable)?;
    let dom_id = kept(facts.dom_id.as_deref()).filter(|id| !looks_generated(id));
    let fingerprint = FieldFingerprint {
        role,
        subrole: kept(facts.subrole.as_deref()),
        name: kept(facts.name()),
        placeholder: kept(facts.placeholder.as_deref()),
        dom_id,
    };
    if fingerprint.name.is_none()
        && fingerprint.placeholder.is_none()
        && fingerprint.dom_id.is_none()
    {
        return Err(FingerprintRefusal::Unrecognizable);
    }
    Ok(fingerprint)
}

// @spec AC-006: Every attribute equal, never a password — .specs/features/013-default-input-per-app/spec.md#ac-006
/// True when `facts` is a non-password element whose role, subrole, name, placeholder and DOM id
/// equal the fingerprint's (an absent attribute must be absent on both sides). The DOM id is
/// compared only when the fingerprint kept one: a generated id read now is ignored.
pub fn matches(fingerprint: &FieldFingerprint, facts: &NodeFacts) -> bool {
    if facts.is_secure() {
        return false;
    }
    let dom_id_matches = match &fingerprint.dom_id {
        Some(expected) => kept(facts.dom_id.as_deref()).as_ref() == Some(expected),
        None => true,
    };
    kept(facts.role.as_deref()).as_ref() == Some(&fingerprint.role)
        && kept(facts.subrole.as_deref()) == fingerprint.subrole
        && kept(facts.name()) == fingerprint.name
        && kept(facts.placeholder.as_deref()) == fingerprint.placeholder
        && dom_id_matches
}

#[cfg(test)]
#[path = "fingerprint_tests.rs"]
mod tests;
