//! Canvas-wide style: font, line height, page margin, background colors.

use crate::color::Color;
use crate::defaults::{DEFAULT_LINEHEIGHT_RATIO, DEFAULT_PAGE_MARGIN_PX};
use crate::text::FontSpec;
use crate::units::Px;

/// Canvas-wide style covering font, line height, page margin, and the SVG-root
/// display scale.
///
/// See `docs/spec/types.md` §4 and the global-parameters section of
/// `docs/spec/tcml-format.md` (`@scale`). The `scale` field affects only the
/// SVG root `width`/`height` attributes — internal coordinates remain at 1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasStyle {
    font: FontSpec,
    line_height: Px,
    page_margin: Px,
    scale: f32,
}

impl CanvasStyle {
    /// Default font specification.
    pub fn font(&self) -> &FontSpec {
        &self.font
    }

    /// Resolved line height.
    pub fn line_height(&self) -> Px {
        self.line_height
    }

    /// Page margin.
    pub(crate) fn page_margin(&self) -> Px {
        self.page_margin
    }

    /// SVG-root display scale (multiplier applied only to root
    /// `width`/`height` attributes; internal coordinates are scale-free).
    ///
    /// `pub(crate)` because the SVG renderer (a sibling module) reads it; the
    /// matching `set_scale` is `pub(super)` because writes are routed through
    /// `ChartStyle`. This mirrors `page_margin` / `set_page_margin`.
    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    /// Override the page margin.
    pub(super) fn set_page_margin(&mut self, margin: Px) {
        self.page_margin = margin;
    }

    /// Override the SVG-root display scale.
    pub(super) fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Override the resolved line height directly (used by layout unit tests
    /// to exercise non-default line heights without recomputing the ratio).
    #[cfg(test)]
    pub(super) fn set_line_height(&mut self, height: Px) {
        self.line_height = height;
    }

    /// Update font size and recalculate line height atomically.
    pub(super) fn set_font_size(&mut self, size: Px) {
        self.font.set_size(size);
        self.line_height = Self::line_height_for_size(size);
    }

    /// Compute the default line height from a font size.
    fn line_height_for_size(size: Px) -> Px {
        size * DEFAULT_LINEHEIGHT_RATIO
    }

    /// Update font family while keeping the current font size.
    pub(super) fn set_font_family(&mut self, family: crate::text::FontFamily) {
        self.font.set_family(family);
    }

    /// Replace the line-height ratio applied to the current font size.
    pub(super) fn set_line_height_ratio(&mut self, ratio: f32) {
        self.line_height = self.font.size() * ratio;
    }
}

impl Default for CanvasStyle {
    fn default() -> Self {
        let font = FontSpec::default();
        let line_height = Self::line_height_for_size(font.size());
        Self {
            font,
            line_height,
            page_margin: DEFAULT_PAGE_MARGIN_PX,
            scale: 1.0,
        }
    }
}

/// Background-fill style for the chart and even/odd row stripes.
///
/// See `docs/spec/types.md` §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundStyle {
    bgcolor0: Color,
    bgcolor1: Color,
}

impl BackgroundStyle {
    /// Pick the row-stripe color for a given 0-based signal-row index.
    ///
    /// Even index → bgcolor0, odd → bgcolor1.
    pub(crate) fn stripe_for_index(&self, index: u32) -> Color {
        if index.is_multiple_of(2) {
            self.bgcolor0
        } else {
            self.bgcolor1
        }
    }

    /// Set the even-row background color.
    pub(super) fn set_bgcolor0(&mut self, color: Color) {
        self.bgcolor0 = color;
    }

    /// Set the odd-row background color.
    pub(super) fn set_bgcolor1(&mut self, color: Color) {
        self.bgcolor1 = color;
    }
}

impl Default for BackgroundStyle {
    fn default() -> Self {
        // Default bgcolor0/bgcolor1 are spec-defined as `none`; encoded directly
        // as `Color::NONE` constants instead of round-tripping through
        // `Color::parse(...).expect(...)` at startup.
        Self {
            bgcolor0: Color::NONE,
            bgcolor1: Color::NONE,
        }
    }
}
