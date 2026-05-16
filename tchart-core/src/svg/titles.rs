//! Title row rendering (`titles` layer).

use crate::geometry::Rect;
use crate::line::{Line, LineContent, TitleRow};
use crate::style::TitleStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};
use crate::units::Px;

/// `WriteSvgOn` source for the `titles` layer.
pub(super) struct TitleRows<'lines> {
    pub(super) lines: &'lines [Line],
    pub(super) page_margin: Px,
}

impl WriteSvgOn for TitleRows<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        for line in self.lines {
            if let LineContent::Title(title) = &line.content {
                target.write(&OneTitle {
                    bounding_box: line.bounding_box,
                    title,
                    page_margin: self.page_margin,
                });
            }
        }
    }
}

/// One title row: `<text>` element with one `<tspan>` per text line.
struct OneTitle<'title> {
    bounding_box: Rect,
    title: &'title TitleRow,
    page_margin: Px,
}

impl WriteSvgOn for OneTitle<'_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let style: &TitleStyle = &self.title.style;
        let font = style.font();
        let font_size = font.size();
        let x = style.resolve_anchor_x(self.bounding_box, self.page_margin);
        let baseline = self.bounding_box.origin.y + font_size;

        target.write_literal("<text");
        target.write_px_attribute("x", x);
        target.write_px_attribute("y", baseline);
        target.write_user_attribute("font-family", &font.family().as_unsafe_line());
        target.write_px_attribute("font-size", font_size);
        target.write_user_attribute("fill", &style.color());
        target.write_static_attribute("text-anchor", style.align().svg_text_anchor());
        target.write_char('>');

        let mut first = true;
        for fragment in self.title.text.lines() {
            target.write_literal("<tspan");
            target.write_px_attribute("x", x);
            if first {
                target.write_static_attribute("dy", "0");
                first = false;
            } else {
                target.write_px_attribute("dy", font_size);
            }
            target.write_char('>');
            target.write_escaped(&fragment);
            target.write_literal("</tspan>");
        }
        target.write_literal("</text>");
    }
}
