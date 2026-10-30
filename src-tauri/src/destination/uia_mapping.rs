//! 011's own definition of a Windows text input (spec AC-007; 010 classifies on macOS only),
//! expressed in the vocabulary of 010's editable rule so the start read applies one rule on both
//! platforms: an element whose UI Automation `ControlType` is Edit or Document and that exposes
//! `ValuePattern` or `TextPattern` is a text field; it is a password field when `IsPassword` is
//! true; it is not editable when its `ValuePattern` reports `IsReadOnly`. Any other element is
//! an unknown role, which 010's rule leaves undetermined (nothing remembered). Pure, so it is
//! tested on every platform.
use crate::recovery::classifier::{AxQueryError, EditAttribute, FocusedElement};
use std::time::Duration;

/// `UIA_EditControlTypeId`.
pub const EDIT_CONTROL_TYPE: i32 = 50_004;
/// `UIA_DocumentControlTypeId`.
pub const DOCUMENT_CONTROL_TYPE: i32 = 50_030;
/// 010's text role a Windows text input maps to.
pub const TEXT_ROLE: &str = "AXTextArea";
/// 010's password role.
pub const PASSWORD_ROLE: &str = "AXSecureTextField";
/// A role 010's rule does not know: undetermined.
pub const OTHER_ROLE: &str = "UIAOther";

/// What UI Automation reports about the focused element (never its value or text).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiaFacts {
    pub control_type: i32,
    pub has_value_pattern: bool,
    pub has_text_pattern: bool,
    pub is_password: bool,
    /// `ValuePattern.CurrentIsReadOnly`, when the pattern exists.
    pub read_only: Option<bool>,
}

// @spec AC-007: Windows text input rules — .specs/features/011-dictation-field-return/spec.md#ac-007
/// The 010 role of `facts`; see the module documentation.
pub fn role_of(facts: &UiaFacts) -> &'static str {
    let text_control = matches!(
        facts.control_type,
        EDIT_CONTROL_TYPE | DOCUMENT_CONTROL_TYPE
    );
    let text_pattern = facts.has_value_pattern || facts.has_text_pattern;
    match (text_control && text_pattern, facts.is_password) {
        (_, true) => PASSWORD_ROLE,
        (true, false) => TEXT_ROLE,
        (false, false) => OTHER_ROLE,
    }
}

/// Whether the text input takes text: not read-only (`ValuePattern` absent counts as writable,
/// `TextPattern` alone exposing no read-only flag).
pub fn takes_text(facts: &UiaFacts) -> bool {
    !facts.read_only.unwrap_or(false)
}

/// The facts of one element seen through 010's `FocusedElement` (answers need no messaging:
/// the Windows adapter reads the facts once, bounded, on its UI Automation thread).
pub struct FactsView {
    pub pid: u32,
    pub facts: UiaFacts,
}
impl FocusedElement for FactsView {
    fn pid(&mut self) -> Result<u32, AxQueryError> {
        Ok(self.pid)
    }
    fn role(&mut self, _timeout: Duration) -> Result<String, AxQueryError> {
        Ok(role_of(&self.facts).to_owned())
    }
    fn subrole(&mut self, _timeout: Duration) -> Result<Option<String>, AxQueryError> {
        Ok(None)
    }
    fn is_settable(&mut self, _: EditAttribute, _timeout: Duration) -> Result<bool, AxQueryError> {
        Ok(takes_text(&self.facts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::destination::workspace::SystemClock;
    use crate::recovery::classifier::{
        classify_focused, Deadline, FocusClassification, Undetermined,
    };

    /// 010's rule over the view: what 011's start read decides on Windows.
    fn decided(facts: UiaFacts) -> Result<FocusClassification, Undetermined> {
        let clock = SystemClock;
        let deadline = Deadline::new(&clock, Duration::from_millis(50), Duration::from_millis(5));
        classify_focused(&mut FactsView { pid: 7, facts }, 7, &deadline)
    }

    fn facts(control_type: i32, value: bool, text: bool) -> UiaFacts {
        UiaFacts {
            control_type,
            has_value_pattern: value,
            has_text_pattern: text,
            read_only: value.then_some(false),
            ..UiaFacts::default()
        }
    }

    // @spec AC-007: Windows text input rules — .specs/features/011-dictation-field-return/spec.md#ac-007
    #[test]
    fn edit_or_document_with_a_value_or_text_pattern_is_a_text_input() {
        for control_type in [EDIT_CONTROL_TYPE, DOCUMENT_CONTROL_TYPE] {
            for (value, text) in [(true, false), (false, true), (true, true)] {
                let facts = facts(control_type, value, text);
                assert_eq!(role_of(&facts), TEXT_ROLE, "{facts:?}");
                assert!(takes_text(&facts));
            }
            assert_eq!(role_of(&facts(control_type, false, false)), OTHER_ROLE);
        }
        // A button, a list, a pane: never a text input, whatever its patterns.
        for control_type in [50_000, 50_008, 50_033] {
            assert_eq!(role_of(&facts(control_type, true, true)), OTHER_ROLE);
        }
    }

    #[test]
    fn a_password_or_read_only_input_is_never_remembered() {
        let password = UiaFacts {
            is_password: true,
            ..facts(EDIT_CONTROL_TYPE, true, false)
        };
        assert_eq!(role_of(&password), PASSWORD_ROLE);
        let read_only = UiaFacts {
            read_only: Some(true),
            ..facts(EDIT_CONTROL_TYPE, true, false)
        };
        assert!(!takes_text(&read_only));
        assert_eq!(decided(password), Err(Undetermined::SecureField));
        assert_eq!(decided(read_only), Err(Undetermined::TextWithoutAffordance));
        assert_eq!(
            decided(facts(EDIT_CONTROL_TYPE, true, false)),
            Ok(FocusClassification::Editable)
        );
        assert_eq!(
            decided(facts(50_000, true, true)),
            Err(Undetermined::UnknownRole)
        );
    }
}
