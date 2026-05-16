//! Label (signal-name) style and horizontal alignment.

use crate::color::Color;
use crate::defaults::{
    DEFAULT_CAP_HEIGHT_RATIO, DEFAULT_NAMEPAD_PX, DEFAULT_OVERLINE_GAP_PX,
    DEFAULT_OVERLINE_THICKNESS_PX,
};
use crate::geometry::Rect;
use crate::text::FontSpec;
use crate::units::Px;

/// Horizontal text alignment used by labels and titles.
///
/// See `docs/spec/types.md` §4 / §3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HorizontalAlign {
    /// Align to the left edge of the available box.
    Left,
    /// Center within the available box.
    Center,
    /// Align to the right edge of the available box.
    Right,
}

impl HorizontalAlign {
    /// SVG `text-anchor` attribute value matching this alignment.
    pub(crate) fn svg_text_anchor(self) -> &'static str {
        match self {
            HorizontalAlign::Left => "start",
            HorizontalAlign::Center => "middle",
            HorizontalAlign::Right => "end",
        }
    }

    /// Parse a case-insensitive alignment keyword (`left` / `center` /
    /// `right`). Returns `None` when the keyword is not recognised.
    pub(crate) fn from_keyword(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("center") {
            Some(Self::Center)
        } else if value.eq_ignore_ascii_case("left") {
            Some(Self::Left)
        } else if value.eq_ignore_ascii_case("right") {
            Some(Self::Right)
        } else {
            None
        }
    }
}

/// Style applied to per-row signal names.
///
/// See `docs/spec/types.md` §4.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LabelStyle {
    font: FontSpec,
    align: HorizontalAlign,
    color: Color,
    padding: Px,
    overline_gap: Px,
    overline_thickness: Px,
}

impl LabelStyle {
    /// Font for the signal name.
    pub(crate) fn font(&self) -> &FontSpec {
        &self.font
    }

    /// Update the font family.
    pub(super) fn set_font_family(&mut self, family: crate::text::FontFamily) {
        self.font.set_family(family);
    }

    /// Update the font size. Driven by the CLI/WASM `--font-size` override
    /// which must propagate into every per-row label snapshot taken at parse
    /// time, since SVG `<text font-size>` is emitted from that snapshot.
    pub(super) fn set_font_size(&mut self, size: Px) {
        self.font.set_size(size);
    }

    /// Horizontal alignment within the label box.
    pub(crate) fn align(&self) -> HorizontalAlign {
        self.align
    }

    /// Text color.
    pub(crate) fn color(&self) -> Color {
        self.color
    }

    /// Update the text color. Driven by `@signal_color` — TCML has no
    /// dedicated label-color directive, so label text shares the signal
    /// stroke color.
    pub(super) fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Padding between the label box and the waveform.
    pub(crate) fn padding(&self) -> Px {
        self.padding
    }

    /// Set the padding.
    pub(super) fn set_padding(&mut self, padding: Px) {
        self.padding = padding;
    }

    /// Stroke width of the signal-name overline.
    pub(crate) fn overline_thickness(&self) -> Px {
        self.overline_thickness
    }

    /// Gap between signal-name cap-top and the overline. Used by parser unit
    /// tests to verify that the `@overline_gap` directive lands on this style.
    #[cfg(test)]
    pub(crate) fn overline_gap(&self) -> Px {
        self.overline_gap
    }

    /// Set the overline gap.
    pub(super) fn set_overline_gap(&mut self, gap: Px) {
        self.overline_gap = gap;
    }

    /// Set the overline thickness.
    pub(super) fn set_overline_thickness(&mut self, thickness: Px) {
        self.overline_thickness = thickness;
    }

    /// Compute the SVG `x` text-anchor coordinate for a label inside `label_box`
    /// whose chart origin starts at `origin_x`.
    pub(crate) fn resolve_anchor_x(&self, origin_x: Px, label_box: Rect) -> Px {
        let x_left = origin_x + label_box.origin.x;
        let x_right = x_left + label_box.size.width - self.padding;
        match self.align {
            HorizontalAlign::Left => x_left,
            HorizontalAlign::Center => (x_left + x_right) * 0.5,
            HorizontalAlign::Right => x_right,
        }
    }

    /// Y coordinate of the overline above the first text line.
    ///
    /// `cap_top = baseline - cap_height` (where `cap_height = font.size * ratio`),
    /// then the overline sits `overline_gap` above the cap top.
    pub(crate) fn overline_y(&self, baseline_y: Px) -> Px {
        let cap_height = self.font.size() * DEFAULT_CAP_HEIGHT_RATIO;
        let cap_top = baseline_y - cap_height;
        cap_top - self.overline_gap
    }

    /// Horizontal extent of the overline (`(x1, x2)` ascending) given the
    /// rendered text width and the SVG text-anchor x coordinate.
    pub(crate) fn overline_x_extent(&self, text_width: Px, anchor_x: Px) -> (Px, Px) {
        match self.align {
            HorizontalAlign::Left => (anchor_x, anchor_x + text_width),
            HorizontalAlign::Center => {
                let half = text_width * 0.5;
                (anchor_x - half, anchor_x + half)
            }
            HorizontalAlign::Right => (anchor_x - text_width, anchor_x),
        }
    }
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            font: FontSpec::default(),
            align: HorizontalAlign::Right,
            color: Color::BLACK,
            padding: DEFAULT_NAMEPAD_PX,
            overline_gap: DEFAULT_OVERLINE_GAP_PX,
            overline_thickness: DEFAULT_OVERLINE_THICKNESS_PX,
        }
    }
}
