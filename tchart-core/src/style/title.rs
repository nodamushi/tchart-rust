//! Title row style.

use crate::color::Color;
use crate::defaults::DEFAULT_TITLE_ALIGN;
use crate::geometry::Rect;
use crate::style::label::HorizontalAlign;
use crate::text::FontSpec;
use crate::units::Px;

/// Style applied to `@title` rows.
///
/// See `docs/spec/types.md` §3.3.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TitleStyle {
    font: FontSpec,
    align: HorizontalAlign,
    color: Color,
}

impl TitleStyle {
    /// Construct a title style with explicit fields.
    pub(crate) fn new(font: FontSpec, align: HorizontalAlign, color: Color) -> Self {
        Self { font, align, color }
    }

    /// Font used for the title text.
    pub(crate) fn font(&self) -> &FontSpec {
        &self.font
    }

    /// Update the font size. Driven by the CLI/WASM `--font-size` override —
    /// title rows take a font snapshot at parse time, so the SVG `<text>`
    /// element only honours the override if every snapshot is updated.
    pub(crate) fn set_font_size(&mut self, size: Px) {
        self.font.set_size(size);
    }

    /// Horizontal alignment.
    pub(crate) fn align(&self) -> HorizontalAlign {
        self.align
    }

    /// Text color.
    pub(crate) fn color(&self) -> Color {
        self.color
    }

    /// Compute the SVG `x` text-anchor coordinate for a title placed inside
    /// `bbox` with the given `page_margin`.
    pub(crate) fn resolve_anchor_x(&self, bbox: Rect, page_margin: Px) -> Px {
        let left = bbox.origin.x;
        let right = left + bbox.size.width;
        match self.align {
            HorizontalAlign::Left => left + page_margin,
            HorizontalAlign::Center => left + bbox.size.width * 0.5,
            HorizontalAlign::Right => right - page_margin,
        }
    }
}

impl Default for TitleStyle {
    fn default() -> Self {
        Self {
            font: FontSpec::default(),
            align: DEFAULT_TITLE_ALIGN,
            color: Color::BLACK,
        }
    }
}
