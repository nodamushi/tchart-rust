//! Layout engine — fills `Line.bbox`, `SignalRow.geometry`, anchor positions,
//! and arrow endpoints for a parsed [`ChartDocument`].
//!
//! See `docs/spec/types.md` §3.2 (symmetric gap), §9 (algorithm), §11
//! (BUG防止条項), and `docs/spec/architecture.md` "レイアウトエンジンの責務".

mod anchors;
mod arrows;
mod errors;
mod font;

#[cfg(test)]
mod tests;

pub use errors::LayoutError;
pub use font::FontMetrics;

use crate::document::ChartDocument;
use crate::line::{Line, LineContent};
use crate::style::ChartStyle;
use crate::text::{FontSpec, SignalName};
use crate::units::Px;

/// Resolved chart-wide dimensions returned by [`layout`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartDimensions {
    /// Total chart width (page-margin × 2 + content).
    pub(crate) width: Px,
    /// Total chart height (page-margin × 2 + stacked rows).
    pub(crate) height: Px,
}

/// Run the full layout pass on `document` in place.
///
/// On success returns the resolved chart dimensions. The function fills
/// `Line.bbox`, `SignalRow.geometry`, the anchor registry, every arrow
/// endpoint, and synthesises clock-edge arrows.
///
/// # Errors
///
/// Returns [`LayoutError::UnresolvedAnchor`] when an arrow references an
/// anchor missing from the registry (a parser invariant violation).
pub fn layout(
    document: &mut ChartDocument,
    fonts: &dyn FontMetrics,
) -> Result<ChartDimensions, LayoutError> {
    let capwidth = resolve_capwidth(&document.style, &document.lines, fonts);
    let style = &document.style;
    let lines = &mut document.lines;
    let stacking_result = Line::stack_lines(lines, style, capwidth);
    let annotations = &mut document.annotations;
    anchors::resolve_inline_anchors(&mut annotations.anchors, lines);
    anchors::rewrite_arrows(&mut annotations.arrows, &annotations.anchors)?;
    // Populate SignalRow.edge_marks (triangle polygons) instead of pushing
    // clock-derived Arrows into Annotations.arrows.
    arrows::emit_clock_edge_marks(lines);
    Ok(stacking_result.into_chart_dimensions(style))
}

fn resolve_capwidth(style: &ChartStyle, lines: &[Line], fonts: &dyn FontMetrics) -> Px {
    if let Some(explicit) = style.layout().capwidth() {
        return explicit;
    }
    let label_style = style.default_label_style();
    let label_font = label_style.font();
    let mut max_label_width = Px::ZERO;
    for line in lines {
        if let LineContent::Signal(row) = &line.content {
            let measured = measure_signal_name(row.name(), label_font, fonts);
            max_label_width = max_label_width.max(measured);
        }
    }
    max_label_width + label_style.padding()
}

fn measure_signal_name(name: &SignalName, font: &FontSpec, fonts: &dyn FontMetrics) -> Px {
    let mut widest = Px::ZERO;
    for line in name.lines() {
        widest = widest.max(fonts.measure_text_width(line.unsafe_text(), font));
    }
    widest
}
