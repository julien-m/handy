//! The destination context of an insertion's final preparation: what the bounded read of
//! feature 015, extended by smart spacing (017), found around the selection that the dictation
//! replaces. Pure data; its `Debug` output names only its kind, never the field's text.
use super::decision::CursorContext;

/// The context before the cursor when nothing was read (static, so it can be borrowed).
static NO_CONTEXT: CursorContext = CursorContext::Absent;

/// What follows the selection end in the destination field (017).
#[derive(Clone, PartialEq, Eq)]
pub enum AfterContext {
    /// Not read (smart spacing off) or the read failed: nothing is deduced after the text.
    Absent,
    /// At most `AFTER_WINDOW_UNITS` UTF-16 units after the selection end, and whether they
    /// reach the end of the field (an empty `text` with `reaches_end` is the end of the field).
    After { text: String, reaches_end: bool },
}

/// The destination context of one insertion.
#[derive(Clone, PartialEq, Eq)]
pub enum InsertionContext {
    /// Nothing could be read: option off, no permission, platform without a reader, another
    /// application than the one in front, Handy in front, a failed query or a spent budget.
    Unavailable,
    /// The focus is proven not to be an editable text field by 010's rule (a terminal's
    /// read-only text, a password field, an unknown role): the formatting steps change nothing.
    NotText,
    /// An editable field answered: the text before the selection (always `Before`) and, when
    /// asked, the text after it.
    Field {
        before: CursorContext,
        after: AfterContext,
    },
}

impl InsertionContext {
    /// The text before the cursor, as smart capitalization (015) decides from it: `Absent`
    /// unless an editable field answered.
    pub fn before(&self) -> &CursorContext {
        match self {
            Self::Field { before, .. } => before,
            Self::Unavailable | Self::NotText => &NO_CONTEXT,
        }
    }
}

impl std::fmt::Debug for AfterContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The field content never reaches a log (015 AC-006, 017 AC-009): only its kind.
        match self {
            Self::Absent => formatter.write_str("Absent"),
            Self::After { reaches_end, .. } => formatter
                .debug_struct("After")
                .field("reaches_end", reaches_end)
                .finish_non_exhaustive(),
        }
    }
}

impl std::fmt::Debug for InsertionContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("Unavailable"),
            Self::NotText => formatter.write_str("NotText"),
            Self::Field { before, after } => formatter
                .debug_struct("Field")
                .field("before", before)
                .field("after", after)
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_field_has_a_context_before() {
        assert_eq!(
            InsertionContext::Unavailable.before(),
            &CursorContext::Absent
        );
        assert_eq!(InsertionContext::NotText.before(), &CursorContext::Absent);
        let before = CursorContext::Before {
            text: "Bonjour".to_owned(),
            reaches_start: true,
        };
        let field = InsertionContext::Field {
            before: before.clone(),
            after: AfterContext::Absent,
        };
        assert_eq!(field.before(), &before);
    }

    #[test]
    fn debug_never_prints_the_field_text() {
        let field = InsertionContext::Field {
            before: CursorContext::Before {
                text: "secret before".to_owned(),
                reaches_start: false,
            },
            after: AfterContext::After {
                text: "secret after".to_owned(),
                reaches_end: true,
            },
        };
        let printed = format!("{field:?}");
        assert!(!printed.contains("secret"), "{printed}");
        assert!(printed.contains("reaches_end: true"), "{printed}");
    }
}
