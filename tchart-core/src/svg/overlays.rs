//! Text-overlay (`%`) rendering.

use crate::document::TextOverlay;
use crate::style::CanvasStyle;
use crate::svg::buf::{SvgBuf, WriteSvgOn};

/// `WriteSvgOn` source for the `overlays` layer (every `%` text overlay).
pub(super) struct TextOverlays<'overlays, 'canvas> {
    pub(super) overlays: &'overlays [TextOverlay],
    pub(super) canvas: &'canvas CanvasStyle,
}

impl WriteSvgOn for TextOverlays<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        for overlay in self.overlays {
            target.write(&OneTextOverlay {
                overlay,
                canvas: self.canvas,
            });
        }
    }
}

/// One `<text>` element rendered at `overlay.at` using the canvas font.
struct OneTextOverlay<'overlay, 'canvas> {
    overlay: &'overlay TextOverlay,
    canvas: &'canvas CanvasStyle,
}

impl WriteSvgOn for OneTextOverlay<'_, '_> {
    fn write_svg_on(&self, target: &mut SvgBuf) {
        let font = self.canvas.font();
        target.write_literal("<text");
        target.write_px_attribute("x", self.overlay.at.x);
        target.write_px_attribute("y", self.overlay.at.y);
        target.write_user_attribute("font-family", &font.family().as_unsafe_line());
        target.write_px_attribute("font-size", font.size());
        target.write_char('>');
        target.write_escaped(&self.overlay.text);
        target.write_literal("</text>");
    }
}
