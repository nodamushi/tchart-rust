//! Signal stroke / guide / highlight / don't-care styles.

use crate::color::Color;
use crate::defaults::{
    DEFAULT_DONTCARE_HATCH_STROKE_COLOR, DEFAULT_GUIDE_WIDTH_PX, DEFAULT_HIGHLIGHT_STYLE,
    DEFAULT_SIGNAL_WIDTH_PX,
};
use crate::style::svg_attrs::SvgAttrList;
use crate::units::Px;

/// Stroke style for the signal polyline plus its sub-styles.
///
/// See `docs/spec/types.md` §4.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalStyle {
    color: Color,
    width: Px,
    guide: GuideStyle,
    highlight: SvgAttrList,
    dontcare_color: Color,
}

impl SignalStyle {
    /// Polyline stroke color (used by the parser when resolving clock-mark colors).
    pub(crate) fn color(&self) -> Color {
        self.color
    }

    /// Polyline stroke width (used by the parser when constructing arrow styles).
    pub(crate) fn stroke_width(&self) -> Px {
        self.width
    }

    /// Set the polyline stroke color.
    pub(super) fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Set the polyline stroke width.
    pub(super) fn set_width(&mut self, width: Px) {
        self.width = width;
    }

    /// Style applied to `|` guide vertical lines.
    pub(crate) fn guide(&self) -> &GuideStyle {
        &self.guide
    }

    /// Set the guide stroke color.
    pub(super) fn set_guide_color(&mut self, color: Color) {
        self.guide.color = color;
    }

    /// Set the guide stroke width.
    pub(super) fn set_guide_width(&mut self, width: Px) {
        self.guide.width = width;
    }

    /// Attribute list applied to `[...]` highlight rectangles.
    pub(crate) fn highlight_attrs(&self) -> &SvgAttrList {
        &self.highlight
    }

    /// Replace the highlight attribute list.
    pub(super) fn set_highlight_attrs(&mut self, attrs: SvgAttrList) {
        self.highlight = attrs;
    }

    /// Hatch line stroke color used for `?` don't-care fills in this row.
    pub(crate) fn dontcare_color(&self) -> Color {
        self.dontcare_color
    }

    /// Replace the don't-care hatch line color.
    pub(super) fn set_dontcare_color(&mut self, color: Color) {
        self.dontcare_color = color;
    }
}

impl Default for SignalStyle {
    fn default() -> Self {
        let dontcare_color = Color::parse(DEFAULT_DONTCARE_HATCH_STROKE_COLOR)
            .expect("DEFAULT_DONTCARE_HATCH_STROKE_COLOR must parse");
        Self {
            color: Color::BLACK,
            width: DEFAULT_SIGNAL_WIDTH_PX,
            guide: GuideStyle::default(),
            highlight: SvgAttrList::from_pairs(DEFAULT_HIGHLIGHT_STYLE),
            dontcare_color,
        }
    }
}

/// Style for `|` guide vertical lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GuideStyle {
    color: Color,
    width: Px,
}

impl GuideStyle {
    /// Stroke color.
    pub(crate) fn color(&self) -> Color {
        self.color
    }

    /// Stroke width.
    pub(crate) fn width(&self) -> Px {
        self.width
    }
}

impl Default for GuideStyle {
    fn default() -> Self {
        Self {
            color: Color::RED,
            width: DEFAULT_GUIDE_WIDTH_PX,
        }
    }
}
