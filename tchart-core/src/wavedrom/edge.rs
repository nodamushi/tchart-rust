//! Arrow → WaveDrom `edge` string conversion.
//!
//! See `docs/spec/wavedrom.md` §edge 配列.

use crate::arrow::{ArrowEnd, ArrowHead, LineDashStyle};
use crate::document::ChartDocument;

use super::node::NodeMap;

/// Convert all `@->` arrows in `document` to WaveDrom edge strings.
///
/// Arrows whose endpoints cannot be resolved to node characters (because the
/// anchor exceeded the 52-character limit) are silently dropped here; the
/// warning was already emitted during [`super::node::build_node_map`].
pub(super) fn build_edges(document: &ChartDocument, node_map: &NodeMap) -> Vec<String> {
    document
        .annotations
        .arrows
        .iter()
        .filter_map(|arrow| {
            let from_char = resolve_node_char(&arrow.from, node_map)?;
            let to_char = resolve_node_char(&arrow.to, node_map)?;
            Some(encode_edge(from_char, to_char, arrow))
        })
        .collect()
}

/// Resolve one [`ArrowEnd`] to the node character assigned by the [`NodeMap`].
fn resolve_node_char(end: &ArrowEnd, node_map: &NodeMap) -> Option<char> {
    match end {
        ArrowEnd::Anchor(id) => node_map.get(id).copied(),
        ArrowEnd::Absolute(_) => None,
    }
}

/// Encode one arrow as a WaveDrom edge string.
fn encode_edge(from: char, to: char, arrow: &crate::arrow::Arrow) -> String {
    let style = edge_style(arrow.style.line, arrow.style.head);
    let label_part = arrow.label.as_ref().map_or(String::new(), |text| {
        let joined = text
            .lines()
            .map(|line| line.unsafe_text())
            .collect::<Vec<_>>()
            .join(" ");
        format!(" {joined}")
    });
    format!("{from}{style}{to}{label_part}")
}

/// Map TCML line/head style to a WaveDrom edge style string.
fn edge_style(line: LineDashStyle, head: ArrowHead) -> &'static str {
    match (line, head) {
        (LineDashStyle::Solid, ArrowHead::EndOnly) => "->",
        (LineDashStyle::Solid, ArrowHead::BothEnds) => "<->",
        (LineDashStyle::Solid, ArrowHead::None) => "-",
        (LineDashStyle::Dashed, ArrowHead::EndOnly)
        | (LineDashStyle::Dotted, ArrowHead::EndOnly) => "-~>",
        (LineDashStyle::Dashed, ArrowHead::BothEnds)
        | (LineDashStyle::Dotted, ArrowHead::BothEnds) => "<-~>",
        (LineDashStyle::Dashed, ArrowHead::None) | (LineDashStyle::Dotted, ArrowHead::None) => "-~",
    }
}
