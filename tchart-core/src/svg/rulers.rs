//! Background guide-line layer (`<g class="rulers">`) for `@ruler` donations.
//!
//! Each row carries a `Vec<RulerContribution>` filled at parse time. The
//! renderer walks every row in document order, merges contributions sharing
//! an x position (last-wins per x), sorts by x ascending, and emits one
//! dashed `<line>` per surviving x.
//!
//! The lines are written in chart-inner coordinates (`y1=0` at the top of
//! the first row, `y2=chart_inner_height` at the bottom of the last row).
//! A wrapper `transform` on the parent `<g>` shifts them into the SVG-wide
//! coordinate system so they visually overlay the rows. The contribution x
//! is also chart-inner-local (relative to the waveform-area start) so the
//! same transform handles the x offset (`page_margin + capwidth`).
//!
//! See `docs/spec/svg-rendering.md` §「`rulers` (`@ruler` 由来の背景縦線)」.

use std::collections::BTreeMap;

use crate::color::Color;
use crate::line::{Line, LineContent};
use crate::style::ChartStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::units::Px;

/// Stroke width applied to every ruler `<line>`. Spec-fixed value.
const RULER_STROKE_WIDTH: &str = "0.5";

/// Dash pattern applied to every ruler `<line>`. Spec-fixed value
/// (`stroke-dasharray="3 5"`).
const RULER_STROKE_DASHARRAY: &str = "3 5";

/// `WriteSvgOn` source emitting the merged `<line>` set plus an opening
/// `<g class="rulers" transform="...">` wrapper.
///
/// The wrapper is written manually because [`SvgBuf::write_layer`] does
/// not support per-layer attributes; we still honour the
/// "1 本も寄付がなければ `<g class="rulers">` 自体を省略" rule by checking
/// the merged map up front.
pub(super) struct Rulers<'lines, 'style> {
    pub(super) lines: &'lines [Line],
    pub(super) style: &'style ChartStyle,
}

impl WriteSvgOn for Rulers<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let merged = merge_contributions(self.lines);
        if merged.is_empty() {
            return;
        }
        let translate_x = chart_inner_origin_x(self.lines, self.style);
        let translate_y = chart_inner_origin_y(self.lines, self.style);
        let inner_height = chart_inner_height(self.lines);
        target.write_literal("<g class=\"rulers\"");
        write_transform_attribute(target, translate_x, translate_y);
        target.write_literal(">");
        for (quantised_x, color) in merged {
            target.write(&RulerLine {
                x: Px(quantised_x.into_px()),
                y_top: Px::ZERO,
                y_bottom: inner_height,
                color,
            });
        }
        target.write_literal("</g>");
    }
}

/// Append a `transform="translate(dx,dy)"` attribute. Both values are
/// emitted via [`SvgBuf::write_px`] to match the buffer's existing numeric
/// formatting (trimmed three-decimal form).
fn write_transform_attribute(target: &mut SvgBuf, dx: Px, dy: Px) {
    target.write_literal(" transform=\"translate(");
    target.write_px(dx);
    target.write_literal(",");
    target.write_px(dy);
    target.write_literal(")\"");
}

/// Merge every line's `RulerContribution` into a `(quantised_x → color)` map.
///
/// `BTreeMap` keeps the iteration in ascending x order, satisfying the
/// "x 昇順" output requirement. Insertion order across rows is preserved
/// from the slice walk, so a later row's color overwrites an earlier row's
/// color at the same quantised x (last-wins per x).
fn merge_contributions(lines: &[Line]) -> BTreeMap<QuantisedX, Color> {
    let mut merged: BTreeMap<QuantisedX, Color> = BTreeMap::new();
    for line in lines {
        for contribution in &line.ruler_contributions {
            merged.insert(QuantisedX::from_px(contribution.x), contribution.color);
        }
    }
    merged
}

/// Chart-coordinate x of the waveform-area origin (= `page_margin + capwidth`).
///
/// Uses the first signal row's bbox origin + signal_box origin when available
/// — the same offset the waveform renderer uses for the x = 0 reference.
/// When no signal row exists, falls back to the first row's bbox origin (so
/// `@skip`-only charts still emit ruler lines at well-defined positions).
/// When the chart is completely empty, returns the canvas page margin.
fn chart_inner_origin_x(lines: &[Line], style: &ChartStyle) -> Px {
    for line in lines {
        if let LineContent::Signal(row) = &line.content {
            return line.bounding_box.origin.x + row.geometry().signal_box.origin.x;
        }
    }
    lines
        .first()
        .map(|line| line.bounding_box.origin.x)
        .unwrap_or_else(|| style.canvas().page_margin())
}

/// Y of the chart inner top (the y of the first row's bbox origin). Falls
/// back to the canvas page margin for empty charts.
fn chart_inner_origin_y(lines: &[Line], style: &ChartStyle) -> Px {
    lines
        .first()
        .map(|line| line.bounding_box.origin.y)
        .unwrap_or_else(|| style.canvas().page_margin())
}

/// Sum of every row's bbox height (= chart inner height). Zero for empty
/// charts (in which case the rulers layer is suppressed anyway).
fn chart_inner_height(lines: &[Line]) -> Px {
    lines
        .iter()
        .map(|line| line.bounding_box.size.height)
        .fold(Px::ZERO, |total, height| total + height)
}

/// Integer key derived from a `Px` value so that two contributions with
/// the same logical x merge regardless of accumulated floating-point error.
/// Uses `(value * 1000).round() as i64` per `docs/spec/svg-rendering.md`'s
/// quantisation recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct QuantisedX(i64);

impl QuantisedX {
    fn from_px(value: Px) -> Self {
        Self((value.to_f32() as f64 * 1000.0).round() as i64)
    }

    /// Restore an approximate `Px` value for rendering. The inverse of
    /// [`Self::from_px`].
    fn into_px(self) -> f32 {
        (self.0 as f64 / 1000.0) as f32
    }
}

/// One `<line>` element of the rulers layer.
struct RulerLine {
    x: Px,
    y_top: Px,
    y_bottom: Px,
    color: Color,
}

impl WriteSvgOn for RulerLine {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<line");
        target.write_px_attribute("x1", self.x);
        target.write_px_attribute("y1", self.y_top);
        target.write_px_attribute("x2", self.x);
        target.write_px_attribute("y2", self.y_bottom);
        target.write_user_attribute("stroke", &self.color);
        target.write_static_attribute("stroke-width", RULER_STROKE_WIDTH);
        target.write_static_attribute("stroke-dasharray", RULER_STROKE_DASHARRAY);
        target.write_literal("/>");
    }
}
