//! Top-level chart document — output of the parser, input to layout/render.
//!
//! See `docs/spec/types.md` §5.

use crate::anchor::AnchorRegistry;
use crate::arrow::Arrow;
use crate::geometry::Point;
use crate::line::Line;
use crate::style::ChartStyle;
use crate::text::UserText;
use crate::units::Px;

/// Complete chart document.
///
/// The four fields are exposed as `pub(crate)` so layout / SVG / parser
/// modules can read and split-borrow them directly instead of routing
/// through a fistful of noun-only accessors. External consumers
/// (`tchart-cli`, `tchart-web`) drive the document through verb-named
/// methods such as [`Self::set_font_size`].
#[derive(Debug, Clone, PartialEq)]
pub struct ChartDocument {
    /// Resolved style.
    pub(crate) style: ChartStyle,
    /// Stacked rows in display order.
    pub(crate) lines: Vec<Line>,
    /// Overlays, arrows, and anchor registry.
    pub(crate) annotations: Annotations,
    /// Source TCML text (for embedding into the SVG output).
    pub(crate) source: TcmlSource,
}

impl ChartDocument {
    /// Construct a chart document from its parts. Used by the parser and by
    /// in-crate test fixtures.
    pub(crate) fn new(
        style: ChartStyle,
        lines: Vec<Line>,
        annotations: Annotations,
        source: TcmlSource,
    ) -> Self {
        Self {
            style,
            lines,
            annotations,
            source,
        }
    }

    /// Override the canvas font size after parsing. Provided so that
    /// `tchart-cli` and `tchart-web` can apply their `--font-size` flag
    /// without exposing the entire `ChartStyle` for mutation.
    ///
    /// The parser snapshots per-row `LabelStyle.font` / `TitleStyle.font` and
    /// per-arrow `label_font` at the point each row / arrow is parsed, so the
    /// canvas-wide font update on its own would not reach the SVG `<text>`
    /// elements. This method walks every line and arrow so the override is
    /// applied uniformly, then `layout` re-derives geometry from the updated
    /// `line_height`.
    pub fn set_font_size(&mut self, size: Px) {
        self.style.set_font_size(size);
        for line in &mut self.lines {
            line.set_font_size(size);
        }
        for arrow in &mut self.annotations.arrows {
            arrow.set_label_font_size(size);
        }
    }
}

/// Annotation collection — `%` overlays, `@->` arrows, and resolved anchors.
///
/// Fields are `pub(crate)` for the same reason as [`ChartDocument`] — direct
/// field access keeps the layout / SVG paths free of noun-only accessors.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Annotations {
    /// `%` text overlays.
    pub(crate) overlays: Vec<TextOverlay>,
    /// `@->` arrows (including clock-edge arrows).
    pub(crate) arrows: Vec<Arrow>,
    /// Resolved anchor positions.
    pub(crate) anchors: AnchorRegistry,
}

impl Annotations {
    /// Construct annotations from existing collections.
    pub(crate) fn new(
        overlays: Vec<TextOverlay>,
        arrows: Vec<Arrow>,
        anchors: AnchorRegistry,
    ) -> Self {
        Self {
            overlays,
            arrows,
            anchors,
        }
    }
}

/// A single `%`-row text overlay.
///
/// Fields are `pub(crate)` so the SVG layer reads them directly.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextOverlay {
    /// Position in chart coordinates.
    pub(crate) at: Point,
    /// Overlay text.
    pub(crate) text: UserText,
}

impl TextOverlay {
    /// Construct an overlay at `at` with `text`.
    pub(crate) fn new(at: Point, text: UserText) -> Self {
        Self { at, text }
    }
}

/// Original TCML source bytes — round-tripped into SVG metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TcmlSource(String);

impl TcmlSource {
    /// Wrap an owned source string.
    pub(crate) fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    /// Borrow the underlying string.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` when the source is empty.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests;
