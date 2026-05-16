//! Signal name labels (`signal-labels` layer) including the `name_overline` decoration.
//!
//! Overline is rendered as an independent `<line>` element, not via
//! `text-decoration="overline"`. See `docs/spec/svg-rendering.md`
//! §「信号名上線 (`@signal(overline)`)」.

use crate::geometry::Point;
use crate::layout::FontMetrics;
use crate::line::{Line, LineContent, SignalRow};
use crate::style::LabelStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::text::SignalName;
use crate::units::Px;

/// `WriteSvgOn` source for the `signal-labels` layer.
pub(super) struct SignalLabels<'lines, 'fonts> {
    pub(super) lines: &'lines [Line],
    pub(super) fonts: &'fonts dyn FontMetrics,
}

impl WriteSvgOn for SignalLabels<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        for line in self.lines {
            if let LineContent::Signal(row) = &line.content {
                target.write(&OneSignalLabel {
                    origin: line.bounding_box.origin,
                    row,
                    fonts: self.fonts,
                });
            }
        }
    }
}

/// One signal-row label: optional overline `<line>` plus the `<text>` element.
struct OneSignalLabel<'row, 'fonts> {
    origin: Point,
    row: &'row SignalRow,
    fonts: &'fonts dyn FontMetrics,
}

impl WriteSvgOn for OneSignalLabel<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let label_box = self.row.geometry().label_box;
        let style = self.row.style().label();
        let font_size = style.font().size();
        let anchor_x = style.resolve_anchor_x(self.origin.x, label_box);
        let baseline_y = self.origin.y + label_box.origin.y + font_size;

        if self.row.decorations().is_name_overline() {
            target.write(&OverlineLine {
                name: self.row.name(),
                anchor_x,
                baseline_y,
                style,
                fonts: self.fonts,
            });
        }

        target.write(&LabelText {
            name: self.row.name(),
            anchor_x,
            baseline_y,
            style,
        });
    }
}

/// `<line>` element drawn above the first text line for `@signal(overline)`.
struct OverlineLine<'style, 'fonts> {
    name: &'style SignalName,
    anchor_x: Px,
    baseline_y: Px,
    style: &'style LabelStyle,
    fonts: &'fonts dyn FontMetrics,
}

impl WriteSvgOn for OverlineLine<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let text_width = self
            .name
            .lines()
            .map(|line| {
                self.fonts
                    .measure_text_width(line.unsafe_text(), self.style.font())
            })
            .fold(Px::ZERO, Px::max);
        let y = self.style.overline_y(self.baseline_y);
        let (x1, x2) = self.style.overline_x_extent(text_width, self.anchor_x);

        target.write_literal("<line");
        target.write_px_attribute("x1", x1);
        target.write_px_attribute("y1", y);
        target.write_px_attribute("x2", x2);
        target.write_px_attribute("y2", y);
        target.write_user_attribute("stroke", &self.style.color());
        target.write_px_attribute("stroke-width", self.style.overline_thickness());
        target.write_literal("/>");
    }
}

/// `<text>…<tspan>…</tspan>…</text>` for one signal label (one tspan per line).
struct LabelText<'style> {
    name: &'style SignalName,
    anchor_x: Px,
    baseline_y: Px,
    style: &'style LabelStyle,
}

impl WriteSvgOn for LabelText<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let font_size = self.style.font().size();
        target.write_literal("<text");
        target.write_px_attribute("x", self.anchor_x);
        target.write_px_attribute("y", self.baseline_y);
        target.write_user_attribute("font-family", &self.style.font().family().as_unsafe_line());
        target.write_px_attribute("font-size", font_size);
        target.write_user_attribute("fill", &self.style.color());
        target.write_static_attribute("text-anchor", self.style.align().svg_text_anchor());
        target.write_char('>');
        for (index, line) in self.name.lines().enumerate() {
            target.write_literal("<tspan");
            target.write_px_attribute("x", self.anchor_x);
            if index == 0 {
                target.write_static_attribute("dy", "0");
            } else {
                target.write_px_attribute("dy", font_size);
            }
            target.write_char('>');
            target.write_escaped(&line);
            target.write_literal("</tspan>");
        }
        target.write_literal("</text>");
    }
}
