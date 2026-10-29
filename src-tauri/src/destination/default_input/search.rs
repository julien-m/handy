//! Bounded search of the remembered field (feature 013, spec AC-006): breadth-first over the
//! focused window of one application, at most `MAX_DEPTH` levels, `MAX_NODES` elements and
//! `SEARCH_BUDGET`, each read bounded by the remaining budget (never below 5 ms). Only a
//! complete search with exactly one match proves the field; two matches stop at once.
use super::fingerprint::matches;
use super::model::FieldFingerprint;
use super::tree::{TreeAccess, TreeNode};
use crate::destination::field::MIN_AX_TIMEOUT;
use crate::destination::workspace::Clock;
use crate::recovery::classifier::{AxQueryError, Deadline};
use std::collections::VecDeque;
use std::time::Duration;

/// Whole budget of one search (spec decision `search_bounds`: Codex measured 108 ms).
pub const SEARCH_BUDGET: Duration = Duration::from_millis(500);
/// Deepest level expanded (Codex's composer measured at depth 31).
pub const MAX_DEPTH: usize = 48;
/// Most elements read (Codex measured 1572 with one session open).
pub const MAX_NODES: usize = 6000;
/// Roles whose children never hold a text field (Chromium text runs, images): not expanded.
const LEAF_ROLES: [&str; 2] = ["AXStaticText", "AXImage"];

/// Why a search could not prove uniqueness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Incomplete {
    /// The focused window could not be read.
    NoWindow,
    /// An element below `MAX_DEPTH` was not expanded.
    Depth,
    /// More than `MAX_NODES` elements.
    Nodes,
    /// The budget ran out (or a read timed out).
    Budget,
}

/// Result of one search.
pub enum SearchOutcome {
    Unique(Box<dyn TreeNode>),
    NotFound,
    Several,
    Incomplete(Incomplete),
}
impl std::fmt::Debug for SearchOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unique(_) => formatter.write_str("Unique"),
            Self::NotFound => formatter.write_str("NotFound"),
            Self::Several => formatter.write_str("Several"),
            Self::Incomplete(why) => write!(formatter, "Incomplete({why:?})"),
        }
    }
}

/// Counts of one search (logged; never a value or a name).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SearchStats {
    pub visited: usize,
    pub matches: usize,
    pub elapsed: Duration,
}

// @spec FR-006: Bounded breadth-first search, unique match — .specs/features/013-default-input-per-app/spec.md#fr-006
/// Search the focused window of `pid` for the field of `fingerprint`; see the module docs.
pub fn search(
    access: &dyn TreeAccess,
    pid: u32,
    fingerprint: &FieldFingerprint,
    clock: &dyn Clock,
) -> (SearchOutcome, SearchStats) {
    let started = clock.now();
    let deadline = Deadline::new(clock, SEARCH_BUDGET, MIN_AX_TIMEOUT);
    let mut stats = SearchStats::default();
    let outcome = match deadline.remaining() {
        Ok(timeout) => match access.focused_window(pid, timeout) {
            Ok(window) => walk(window, fingerprint, &deadline, &mut stats),
            Err(_) => SearchOutcome::Incomplete(Incomplete::NoWindow),
        },
        Err(_) => SearchOutcome::Incomplete(Incomplete::Budget),
    };
    stats.elapsed = clock.now().saturating_duration_since(started);
    (outcome, stats)
}

/// Breadth-first walk from `window`. Strategy: read each element once (facts and children),
/// keep the first match, stop at the second; an element that cannot be read (gone, refused) is
/// skipped, a timeout or a spent budget makes the search incomplete.
fn walk(
    window: Box<dyn TreeNode>,
    fingerprint: &FieldFingerprint,
    deadline: &Deadline<'_>,
    stats: &mut SearchStats,
) -> SearchOutcome {
    let mut queue: VecDeque<(Box<dyn TreeNode>, usize)> = VecDeque::from([(window, 0)]);
    let mut found: Option<Box<dyn TreeNode>> = None;
    while let Some((mut node, depth)) = queue.pop_front() {
        if stats.visited >= MAX_NODES {
            return SearchOutcome::Incomplete(Incomplete::Nodes);
        }
        let Ok(timeout) = deadline.remaining() else {
            return SearchOutcome::Incomplete(Incomplete::Budget);
        };
        stats.visited += 1;
        let read = match node.read(timeout) {
            Ok(read) => read,
            Err(AxQueryError::Timeout) => return SearchOutcome::Incomplete(Incomplete::Budget),
            // The element vanished or refuses the read: it holds nothing to search.
            Err(_) => continue,
        };
        if matches(fingerprint, &read.facts) {
            stats.matches += 1;
            // A second match ends the search: the field is not unique.
            if found.is_some() {
                return SearchOutcome::Several;
            }
            found = Some(node);
        }
        let is_leaf = read
            .facts
            .role
            .as_deref()
            .is_some_and(|role| LEAF_ROLES.contains(&role));
        if is_leaf || read.children.is_empty() {
            continue;
        }
        if depth >= MAX_DEPTH {
            return SearchOutcome::Incomplete(Incomplete::Depth);
        }
        queue.extend(read.children.into_iter().map(|child| (child, depth + 1)));
    }
    found.map_or(SearchOutcome::NotFound, SearchOutcome::Unique)
}

#[cfg(test)]
#[path = "search_tests.rs"]
pub(crate) mod tests;
