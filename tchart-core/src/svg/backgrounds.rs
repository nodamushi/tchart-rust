//! Row background rectangles (`@bgcolor0` / `@bgcolor1` / `@bg`).
//!
//! `Line.background` (set by `@bg`) overrides the alternating `bgcolor0/1`
//! stripes for that row. `Skip` and `Title` rows are excluded from the
//! even/odd index used for stripe selection.

use crate::color::Color;
use crate::geometry::Rect;
use crate::line::{Line, LineContent};
use crate::style::ChartStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};

/// `WriteSvgOn` source that emits `<rect>` elements for the row-background
/// layer (one per line that has a non-NONE color).
pub(super) struct RowBackgrounds<'lines, 'style> {
    pub(super) lines: &'lines [Line],
    pub(super) style: &'style ChartStyle,
}

impl WriteSvgOn for RowBackgrounds<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let mut signal_index = 0u32;
        for line in self.lines {
            let color = resolve_row_color(line, self.style, &mut signal_index);
            if !color.is_none() {
                target.write(&RowRect {
                    bounding_box: line.bounding_box,
                    color,
                });
            }
        }
    }
}

fn resolve_row_color(line: &Line, style: &ChartStyle, signal_index: &mut u32) -> Color {
    match line.background {
        Some(local) => {
            if matches!(line.content, LineContent::Signal(_)) {
                *signal_index += 1;
            }
            local
        }
        None => match &line.content {
            LineContent::Signal(_) => {
                let color = style.stripe_for_signal_index(*signal_index);
                *signal_index += 1;
                color
            }
            LineContent::Skip(_) | LineContent::Title(_) => Color::NONE,
        },
    }
}

/// One row-background `<rect>` filled with `color`.
struct RowRect {
    bounding_box: Rect,
    color: Color,
}

impl WriteSvgOn for RowRect {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        target.write_literal("<rect");
        target.write_px_attribute("x", self.bounding_box.origin.x);
        target.write_px_attribute("y", self.bounding_box.origin.y);
        target.write_px_attribute("width", self.bounding_box.size.width);
        target.write_px_attribute("height", self.bounding_box.size.height);
        target.write_user_attribute("fill", &self.color);
        target.write_literal("/>");
    }
}
