//! Deliberate layouts that smart paragraphs never reorganize (feature 016 AC-005): existing
//! line breaks, list items and code.

/// The layout found in a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Structure {
    /// A line break, dictated ("new line"), produced by the AI prompt, or part of a block.
    LineBreak,
    /// Starts with a list marker: `-`, `*`, `•` or a number with `.` or `)`, then a space.
    ListItem,
    /// Contains a fenced code marker (```` ``` ````).
    CodeFence,
    /// Starts with a tab or four spaces, the indentation of a code block.
    IndentedCode,
}

const CODE_FENCE: &str = "```";
const CODE_INDENT: &str = "    ";
const BULLETS: [char; 3] = ['-', '*', '•'];

// @spec AC-005: Lists, code and line breaks kept — .specs/features/016-smart-paragraphs/spec.md#ac-005
/// The deliberate layout of `text`, or `None` for plain prose.
///
/// A line break alone is enough: whatever the user or their prompt laid out across lines (lists,
/// code blocks, dictated breaks) is kept as is rather than partly rearranged.
pub(crate) fn detect(text: &str) -> Option<Structure> {
    if text.contains(['\n', '\r']) {
        return Some(Structure::LineBreak);
    }
    if text.contains(CODE_FENCE) {
        return Some(Structure::CodeFence);
    }
    if text.starts_with('\t') || text.starts_with(CODE_INDENT) {
        return Some(Structure::IndentedCode);
    }
    starts_with_list_marker(text.trim_start()).then_some(Structure::ListItem)
}

/// True when `line` starts with a bullet or an ordinal (`1.`, `12)`) followed by whitespace.
fn starts_with_list_marker(line: &str) -> bool {
    let rest = match line.strip_prefix(BULLETS) {
        Some(rest) => rest,
        None => {
            let digits = line.len() - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            if digits == 0 {
                return false;
            }
            match line[digits..].strip_prefix(['.', ')']) {
                Some(rest) => rest,
                None => return false,
            }
        }
    };
    rest.starts_with(char::is_whitespace)
}

#[cfg(test)]
#[path = "structure_tests.rs"]
mod tests;
