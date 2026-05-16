//! Arrow rendering (`arrows` layer) — stroke + arrow head + optional label.
//!
//! All arrows are expected to have already been rewritten to absolute
//! endpoints by the layout engine (`Annotations.arrows` resolved).
//!
//! The entry point is `SvgBuf::write(&Arrow)` via the [`WriteSvgOn`] trait;
//! `Arrow` is the only caller-visible name.

use crate::arrow::{Arrow, ArrowEnd, ArrowHead, ArrowStyle, LineDashStyle};
use crate::defaults::{DEFAULT_ARROW_LABEL_OUTLINE_COLOR, DEFAULT_ARROW_LABEL_OUTLINE_WIDTH_PX};
use crate::geometry::Point;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::text::{FontSpec, UserText};
use crate::units::Px;

const HEAD_SIZE_PX: f32 = 6.0;
const LABEL_OFFSET_PX: f32 = 4.0;
const DASHED: &str = "6 3";
const DOTTED: &str = "1 2";

/// Render every arrow in the slice.
pub(super) struct ArrowList<'arrows>(pub(super) &'arrows [Arrow]);

impl WriteSvgOn for ArrowList<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        for arrow in self.0 {
            target.write(arrow);
        }
    }
}

impl WriteSvgOn for Arrow {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let Some(from) = absolute_endpoint(&self.from) else {
            return;
        };
        let Some(to) = absolute_endpoint(&self.to) else {
            return;
        };
        let style = &self.style;
        target.write(&ArrowLine { from, to, style });
        write_heads(target, from, to, style);
        if let Some(label) = self.label.as_ref() {
            target.write(&ArrowLabel {
                from,
                to,
                label,
                font: &self.label_font,
            });
        }
    }
}

fn write_heads(target: &mut SvgBuf, from: Point, to: Point, style: &ArrowStyle) {
    let (head_at_to, head_at_from) = match style.head {
        ArrowHead::None => (false, false),
        ArrowHead::EndOnly => (true, false),
        ArrowHead::BothEnds => (true, true),
    };
    if head_at_to {
        target.write(&ArrowHeadPath { from, to, style });
    }
    if head_at_from {
        target.write(&ArrowHeadPath {
            from: to,
            to: from,
            style,
        });
    }
}

fn absolute_endpoint(end: &ArrowEnd) -> Option<Point> {
    match end {
        ArrowEnd::Absolute(point) => Some(*point),
        ArrowEnd::Anchor(_) => None,
    }
}

/// `<line>` element drawn between two endpoints with the arrow's stroke style.
struct ArrowLine<'style> {
    from: Point,
    to: Point,
    style: &'style ArrowStyle,
}

impl WriteSvgOn for ArrowLine<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<line");
        target.write_px_attribute("x1", self.from.x);
        target.write_px_attribute("y1", self.from.y);
        target.write_px_attribute("x2", self.to.x);
        target.write_px_attribute("y2", self.to.y);
        target.write_user_attribute("stroke", &self.style.color);
        target.write_px_attribute("stroke-width", self.style.width);
        if let Some(dash) = format_dash_value(self.style.line) {
            target.write_static_attribute("stroke-dasharray", dash);
        }
        target.write_literal("/>");
    }
}

fn format_dash_value(line: LineDashStyle) -> Option<&'static str> {
    match line {
        LineDashStyle::Solid => None,
        LineDashStyle::Dashed => Some(DASHED),
        LineDashStyle::Dotted => Some(DOTTED),
    }
}

struct ArrowHeadGeometry {
    tip: Point,
    base_x: f32,
    base_y: f32,
    perp_x: f32,
    perp_y: f32,
    half_width: f32,
}

impl ArrowHeadGeometry {
    fn compute(from: Point, to: Point) -> Self {
        let delta_x = to.x.to_f32() - from.x.to_f32();
        let delta_y = to.y.to_f32() - from.y.to_f32();
        let length = (delta_x * delta_x + delta_y * delta_y).sqrt().max(1e-6);
        let unit_x = delta_x / length;
        let unit_y = delta_y / length;
        let perp_x = -unit_y;
        let perp_y = unit_x;
        let base_x = to.x.to_f32() - unit_x * HEAD_SIZE_PX;
        let base_y = to.y.to_f32() - unit_y * HEAD_SIZE_PX;
        Self {
            tip: to,
            base_x,
            base_y,
            perp_x,
            perp_y,
            half_width: HEAD_SIZE_PX * 0.5,
        }
    }
}

/// `<path>` triangle for one arrow head, pointing at `to`.
///
/// Spec (`docs/spec/svg-rendering.md` §「矢印頭」) mandates `<path>` (not
/// `<polygon>`) so that `<polygon>` remains exclusive to clock edge markers
/// in the `edge-marks` layer and to dontcare fills in the `dontcares` layer.
struct ArrowHeadPath<'style> {
    from: Point,
    to: Point,
    style: &'style ArrowStyle,
}

impl WriteSvgOn for ArrowHeadPath<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let geometry = ArrowHeadGeometry::compute(self.from, self.to);
        let point1_x = geometry.base_x + geometry.perp_x * geometry.half_width;
        let point1_y = geometry.base_y + geometry.perp_y * geometry.half_width;
        let point2_x = geometry.base_x - geometry.perp_x * geometry.half_width;
        let point2_y = geometry.base_y - geometry.perp_y * geometry.half_width;
        target.write_literal("<path d=\"M");
        target.write_px(geometry.tip.x);
        target.write_char(',');
        target.write_px(geometry.tip.y);
        target.write_char('L');
        target.write_px(Px(point1_x));
        target.write_char(',');
        target.write_px(Px(point1_y));
        target.write_char('L');
        target.write_px(Px(point2_x));
        target.write_char(',');
        target.write_px(Px(point2_y));
        target.write_literal("Z\"");
        target.write_user_attribute("fill", &self.style.color);
        target.write_literal("/>");
    }
}

/// Centered `<text>` label rendered slightly above the midpoint of an arrow.
///
/// The label carries a white outline (`paint-order="stroke fill"` + fixed
/// stroke color/width/linejoin) so it remains readable when crossing waveform
/// lines.  See `docs/spec/svg-rendering.md` §「矢印 (`arrows`)」§「ラベル」.
struct ArrowLabel<'label> {
    from: Point,
    to: Point,
    label: &'label UserText,
    /// Font captured at `@->` declaration time. Written as attributes on the
    /// `<text>` element (not in `<style>`) per the CSS-injection-avoidance rule
    /// in `docs/spec/svg-rendering.md`.
    font: &'label FontSpec,
}

impl WriteSvgOn for ArrowLabel<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let mid_x = (self.from.x + self.to.x) * 0.5;
        let mid_y = (self.from.y + self.to.y) * 0.5 - Px(LABEL_OFFSET_PX);
        target.write_literal("<text");
        target.write_px_attribute("x", mid_x);
        target.write_px_attribute("y", mid_y);
        target.write_user_attribute("font-family", &self.font.family().as_unsafe_line());
        target.write_px_attribute("font-size", self.font.size());
        target.write_static_attribute("text-anchor", "middle");
        target.write_static_attribute("paint-order", "stroke fill");
        target.write_static_attribute("stroke", DEFAULT_ARROW_LABEL_OUTLINE_COLOR);
        target.write_px_attribute("stroke-width", Px(DEFAULT_ARROW_LABEL_OUTLINE_WIDTH_PX));
        target.write_static_attribute("stroke-linejoin", "round");
        target.write_char('>');
        target.write_escaped(self.label);
        target.write_literal("</text>");
    }
}
