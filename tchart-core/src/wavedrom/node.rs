//! Anchor -> WaveDrom node character assignment.
//!
//! See `docs/spec/wavedrom.md` §アンカーと矢印.

use std::collections::{HashMap, HashSet};

use crate::anchor::AnchorId;
use crate::arrow::ArrowEnd;
use crate::document::ChartDocument;
use crate::line::{LineContent, WaveformElement};

use super::warning::WaveDromWarning;

/// Maximum number of assignable node characters (a-z, A-Z).
const MAX_NODE_CHARS: usize = 52;

/// Mapping from [`AnchorId`] to the assigned WaveDrom node character.
///
/// Only anchors actually referenced by at least one `@->` receive an entry.
pub(super) type NodeMap = HashMap<AnchorId, char>;

/// Build the [`NodeMap`] for `document`.
///
/// Returns `(node_map, warning)`. When more than 52 anchors are referenced,
/// the excess are omitted and a [`WaveDromWarning::TooManyAnchors`] is returned.
///
/// Anchors are enumerated in TCML appearance order (signal-row order then
/// within-row order). Characters `a-z`, `A-Z` (52 total) are assigned in
/// sequence.
pub(super) fn build_node_map(document: &ChartDocument) -> (NodeMap, Option<WaveDromWarning>) {
    let referenced = collect_referenced_ids(document);
    let ordered = collect_ordered_ids(document, &referenced);
    assign_node_chars(ordered)
}

/// Build the `node` string for the given signal row.
///
/// Length equals `wave_len`. Anchor positions (those in `node_map`) receive
/// their node character; all other positions receive `.`.
/// Returns `None` when no anchor in the row has a node character assignment.
pub(super) fn build_node_string(
    row: &crate::line::SignalRow,
    node_map: &NodeMap,
    wave_len: usize,
) -> Option<String> {
    let node_chars = fill_node_chars(row.waveform().iter(), node_map, wave_len);
    let has_node = node_chars.iter().any(|character| *character != '.');
    if has_node {
        Some(node_chars.iter().collect())
    } else {
        None
    }
}

/// Scan waveform elements and fill a node-char buffer of length `wave_len`.
///
/// Anchors are zero-width in TCML, so several referenced anchors may share the
/// same wave column. WaveDrom's `node` string can hold only one letter per
/// column, but the spec requires every referenced anchor to receive a node
/// character. When the natural column is already occupied, search for the
/// nearest free `.` slot — preferring the column immediately to the left
/// (which represents the tail of the just-finished level run) and falling
/// back to the right — so each referenced anchor still ends up as a letter
/// in the node string.
fn fill_node_chars<'element>(
    elements: impl Iterator<Item = &'element WaveformElement>,
    node_map: &NodeMap,
    wave_len: usize,
) -> Vec<char> {
    let mut buffer: Vec<char> = vec!['.'; wave_len];
    let mut wave_position: usize = 0;
    for element in elements {
        match element {
            WaveformElement::Anchor(id) => {
                if let Some(&node_char) = node_map.get(id)
                    && let Some(slot) = find_node_slot(&buffer, wave_position)
                {
                    buffer[slot] = node_char;
                }
                // Anchors have zero width — do not advance wave_position.
            }
            WaveformElement::Level(run) => wave_position += run.units() as usize,
            WaveformElement::Gap => wave_position += 1,
            WaveformElement::Guide
            | WaveformElement::HighlightStart
            | WaveformElement::HighlightEnd
            | WaveformElement::Text(_)
            | WaveformElement::Transition(_) => {}
        }
    }
    buffer
}

/// Find a `.` slot for a node character, starting at `preferred_index` and
/// expanding outward (left first, then right). Returns `None` when every slot
/// in the buffer is already occupied.
///
/// When `preferred_index == 0` there is no left candidate to inspect, so the
/// search only expands to the right. Symmetrically, when `preferred_index`
/// equals `buffer.len() - 1` the search only expands to the left.
fn find_node_slot(buffer: &[char], preferred_index: usize) -> Option<usize> {
    if buffer.is_empty() {
        return None;
    }
    let length = buffer.len();
    let start = preferred_index.min(length - 1);
    if buffer[start] == '.' {
        return Some(start);
    }
    let max_distance = start.max(length - 1 - start);
    for distance in 1..=max_distance {
        if distance <= start {
            let left_candidate = start - distance;
            if buffer[left_candidate] == '.' {
                return Some(left_candidate);
            }
        }
        let right_candidate = start + distance;
        if right_candidate < length && buffer[right_candidate] == '.' {
            return Some(right_candidate);
        }
    }
    None
}

/// Collect the set of [`AnchorId`]s that are endpoint of at least one `@->`.
///
/// The spec §アンカーと矢印 hand-wavy describes this as reading from
/// `annotations.anchors` (AnchorRegistry), but `AnchorRegistry` does not
/// expose an iterator over IDs. The equivalent result is obtained by walking
/// `annotations.arrows`, which only ever contains IDs that are already
/// validated against the registry by the parser (any unknown ID is a
/// `ParseError::UndefinedAnchor`). The two approaches are therefore
/// semantically equivalent for any well-formed `ChartDocument`.
fn collect_referenced_ids(document: &ChartDocument) -> HashSet<AnchorId> {
    document
        .annotations
        .arrows
        .iter()
        .flat_map(|arrow| {
            [&arrow.from, &arrow.to]
                .into_iter()
                .filter_map(|arrow_end| {
                    if let ArrowEnd::Anchor(id) = arrow_end {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
        })
        .collect()
}

/// Walk all signal rows in order and return the referenced [`AnchorId`]s in
/// TCML appearance order (deduplicated, first occurrence wins).
fn collect_ordered_ids(document: &ChartDocument, referenced: &HashSet<AnchorId>) -> Vec<AnchorId> {
    let mut collector = OrderedAnchorIds::default();
    for line in &document.lines {
        let LineContent::Signal(row) = &line.content else {
            continue;
        };
        for element in row.waveform().iter() {
            if let WaveformElement::Anchor(id) = element
                && referenced.contains(id)
            {
                collector.push(id);
            }
        }
    }
    collector.into_vec()
}

/// Deduplicating, order-preserving collector for [`AnchorId`]s.
///
/// Holds both a `HashSet` for O(1) membership and a `Vec` for insertion order;
/// the dual storage is encapsulated here so callers see a single
/// `push` method instead of two parallel locals.
#[derive(Default)]
struct OrderedAnchorIds {
    seen: HashSet<AnchorId>,
    ordered: Vec<AnchorId>,
}

impl OrderedAnchorIds {
    /// Append `id` if it has not been seen before; no-op otherwise.
    fn push(&mut self, id: &AnchorId) {
        if self.seen.insert(id.clone()) {
            self.ordered.push(id.clone());
        }
    }

    /// Consume the collector and return the ordered ID vector.
    fn into_vec(self) -> Vec<AnchorId> {
        self.ordered
    }
}

/// Assign node characters `a-z A-Z` (52 total).
///
/// Returns `(node_map, warning)`. When more than 52 anchors are present,
/// a [`WaveDromWarning::TooManyAnchors`] is returned and excess anchors are
/// omitted from the map.
fn assign_node_chars(ordered: Vec<AnchorId>) -> (NodeMap, Option<WaveDromWarning>) {
    let warning = if ordered.len() > MAX_NODE_CHARS {
        Some(WaveDromWarning::TooManyAnchors)
    } else {
        None
    };
    let map = ordered
        .into_iter()
        .take(MAX_NODE_CHARS)
        .zip(('a'..='z').chain('A'..='Z'))
        .collect();
    (map, warning)
}
