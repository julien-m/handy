//! Compiled single-pass replacement matching over original UTF-8 offsets.
//!
//! Strategy: rule keys (full case folding, whitespace runs collapsed) are
//! compiled into a trie. The input is folded once with source offsets; at each
//! original scalar the trie is walked and the deepest terminal whose span ends
//! on a complete source scalar and passes the word-boundary test wins. The
//! matched span is emitted as a protected replacement and scanning resumes after
//! it, so replacement output is never rescanned.

use std::collections::HashMap;
use std::ops::Range;

use super::{
    model::{ReplacementRule, RuleId},
    unicode::{FoldedToken, UnicodeData},
    validation::{validate_rules, RuleIssue},
};
use crate::audio_toolkit::TextSpan;

const ROOT: u32 = 0;

/// Why a rule set cannot be compiled; never carries rule text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// The rule set fails the shared validator.
    InvalidRules(Vec<RuleIssue>),
    /// The embedded Unicode tables are unusable.
    UnicodeDataUnavailable,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRules(issues) => write!(
                formatter,
                "Invalid replacement rule set ({} issues)",
                issues.len()
            ),
            Self::UnicodeDataUnavailable => {
                formatter.write_str("Replacement Unicode data unavailable")
            }
        }
    }
}

impl std::error::Error for CompileError {}

struct CompiledRule {
    id: RuleId,
    replacement: String,
    /// AC-006: a recognized side starting/ending with CJK skips its boundary test.
    is_start_exempt: bool,
    is_end_exempt: bool,
}

/// Immutable matcher for one captured rule snapshot; cheap to share by reference.
#[derive(Default)]
pub struct CompiledReplacements {
    data: Option<&'static UnicodeData>,
    rules: Vec<CompiledRule>,
    edges: HashMap<(u32, char), u32>,
    terminals: HashMap<u32, usize>,
    node_count: u32,
}

/// One piece of the matcher output, with its original byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputSegment {
    /// Original text kept byte for byte.
    Source { source: Range<usize>, text: String },
    /// Exact replacement text that later approximate correction must not alter.
    Replacement {
        rule_id: RuleId,
        source: Range<usize>,
        text: String,
    },
}

/// Ordered segments whose concatenation is the replaced text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplacementOutput {
    pub segments: Vec<OutputSegment>,
}

impl ReplacementOutput {
    /// Concatenate every segment into the final replaced text.
    pub fn text(&self) -> String {
        self.spans().iter().map(|span| span.text).collect()
    }

    /// Whether at least one rule matched.
    pub fn has_replacements(&self) -> bool {
        self.segments
            .iter()
            .any(|segment| matches!(segment, OutputSegment::Replacement { .. }))
    }

    /// Borrow the segments as spans for the span-aware approximate correction.
    pub fn spans(&self) -> Vec<TextSpan<'_>> {
        self.segments
            .iter()
            .map(|segment| match segment {
                OutputSegment::Source { text, .. } => TextSpan {
                    text,
                    is_protected: false,
                },
                OutputSegment::Replacement { text, .. } => TextSpan {
                    text,
                    is_protected: true,
                },
            })
            .collect()
    }
}

impl CompiledReplacements {
    /// A matcher with no rules; applying it leaves text unchanged.
    pub fn empty() -> Self {
        Self::default()
    }

    // @spec FR-005: Compile rules once per snapshot — .specs/features/002-word-replacements/spec.md#fr-005
    /// Validate and compile an ordered rule snapshot.
    ///
    /// # Errors
    ///
    /// - `CompileError::InvalidRules` when the set fails the shared validator.
    /// - `CompileError::UnicodeDataUnavailable` when the embedded tables are unusable.
    pub fn compile(rules: &[ReplacementRule]) -> Result<Self, CompileError> {
        let rules = validate_rules(rules).map_err(CompileError::InvalidRules)?;
        let data = UnicodeData::load().map_err(|_| CompileError::UnicodeDataUnavailable)?;
        let mut compiled = Self {
            data: Some(data),
            node_count: 1,
            ..Self::default()
        };
        for rule in &rules {
            compiled.insert(data, rule);
        }
        Ok(compiled)
    }

    fn insert(&mut self, data: &UnicodeData, rule: &ReplacementRule) {
        let mut node = ROOT;
        for character in data.normalized_key(&rule.recognized).chars() {
            node = match self.edges.get(&(node, character)) {
                Some(&next) => next,
                None => {
                    let next = self.node_count;
                    self.node_count += 1;
                    self.edges.insert((node, character), next);
                    next
                }
            };
        }
        let is_cjk = |character: Option<char>| character.is_some_and(|c| data.is_cjk(c));
        self.terminals.insert(node, self.rules.len());
        self.rules.push(CompiledRule {
            id: rule.id.clone(),
            replacement: rule.replacement.clone(),
            is_start_exempt: is_cjk(rule.recognized.chars().next()),
            is_end_exempt: is_cjk(rule.recognized.chars().next_back()),
        });
    }

    /// Whether no rule is compiled.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    // @spec AC-006: Case-insensitive whole-word matching, AC-007: Longest single pass — .specs/features/002-word-replacements/spec.md#ac-006
    /// Replace matches in one left-to-right pass; unmatched bytes are kept.
    pub fn apply(&self, text: &str) -> ReplacementOutput {
        let Some(data) = self.data.filter(|_| !self.rules.is_empty()) else {
            return unchanged(text);
        };
        let tokens = data.fold_with_offsets(text);
        let mut output = SegmentBuilder::new(text);
        let mut index = 0;
        while let Some(token) = tokens.get(index) {
            if let Some((end, rule)) = self.longest_match(data, text, &tokens, index) {
                output.push_replacement(
                    &self.rules[rule],
                    token.source_start..tokens[end].source_end,
                );
                index = end + 1;
            } else {
                output.push_source(token.source_start..token.source_end);
                index = next_source_scalar(&tokens, index);
            }
        }
        output.finish()
    }

    /// Deepest valid terminal from `start`; list order only matters for equal
    /// lengths, which unique keys on one trie path cannot produce.
    fn longest_match(
        &self,
        data: &UnicodeData,
        text: &str,
        tokens: &[FoldedToken],
        start: usize,
    ) -> Option<(usize, usize)> {
        let source_start = tokens.get(start)?.source_start;
        let mut node = ROOT;
        let mut best = None;
        for (index, token) in tokens.iter().enumerate().skip(start) {
            // Stop as soon as no rule key continues with this folded scalar.
            let Some(&next) = self.edges.get(&(node, token.character)) else {
                break;
            };
            node = next;
            // A match may not end halfway through one source scalar's expansion.
            let span = source_start..token.source_end;
            let valid = self.terminals.get(&node).copied().filter(|&rule| {
                token.is_source_end && has_boundaries(data, text, span.clone(), &self.rules[rule])
            });
            best = valid.map(|rule| (index, rule)).or(best);
        }
        best
    }
}

/// AC-006 boundary: neighbours are not Letter/Decimal_Number, unless that side
/// of the recognized text is CJK (written without spaces).
fn has_boundaries(data: &UnicodeData, text: &str, span: Range<usize>, rule: &CompiledRule) -> bool {
    let is_word = |character: Option<char>| character.is_some_and(|c| data.is_letter_or_digit(c));
    let before = text
        .get(..span.start)
        .and_then(|head| head.chars().next_back());
    let after = text.get(span.end..).and_then(|tail| tail.chars().next());
    (rule.is_start_exempt || !is_word(before)) && (rule.is_end_exempt || !is_word(after))
}

/// Index of the first token of the next original scalar or whitespace run.
fn next_source_scalar(tokens: &[FoldedToken], index: usize) -> usize {
    tokens
        .iter()
        .enumerate()
        .skip(index)
        .find(|(_, token)| token.is_source_end)
        .map_or(tokens.len(), |(end, _)| end + 1)
}

fn unchanged(text: &str) -> ReplacementOutput {
    let mut output = SegmentBuilder::new(text);
    output.push_source(0..text.len());
    output.finish()
}

/// Accumulates contiguous source ranges into single segments.
struct SegmentBuilder<'t> {
    text: &'t str,
    segments: Vec<OutputSegment>,
    pending: Option<Range<usize>>,
}

impl<'t> SegmentBuilder<'t> {
    fn new(text: &'t str) -> Self {
        Self {
            text,
            segments: Vec::new(),
            pending: None,
        }
    }

    fn push_source(&mut self, range: Range<usize>) {
        self.pending = match self.pending.take() {
            Some(pending) if pending.end == range.start => Some(pending.start..range.end),
            Some(pending) => {
                self.flush(pending);
                Some(range)
            }
            None => Some(range),
        };
    }

    fn push_replacement(&mut self, rule: &CompiledRule, source: Range<usize>) {
        if let Some(pending) = self.pending.take() {
            self.flush(pending);
        }
        self.segments.push(OutputSegment::Replacement {
            rule_id: rule.id.clone(),
            source,
            text: rule.replacement.clone(),
        });
    }

    fn flush(&mut self, source: Range<usize>) {
        if source.is_empty() {
            return;
        }
        let text = self.text.get(source.clone()).unwrap_or_default().to_owned();
        self.segments.push(OutputSegment::Source { source, text });
    }

    fn finish(mut self) -> ReplacementOutput {
        if let Some(pending) = self.pending.take() {
            self.flush(pending);
        }
        ReplacementOutput {
            segments: self.segments,
        }
    }
}
