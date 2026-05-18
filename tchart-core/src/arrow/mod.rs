//! Arrow types — `@->` declarations.
//!
//! See `docs/spec/types.md` §3.5.

use crate::anchor::{AnchorId, AnchorName};
use crate::color::Color;
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::geometry::Point;
use crate::text::{FontSpec, UserText};
use crate::units::Px;

/// A rendered arrow between two endpoints.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Arrow {
    /// Source endpoint.
    pub(crate) from: ArrowEnd,
    /// Target endpoint.
    pub(crate) to: ArrowEnd,
    /// Visual style.
    pub(crate) style: ArrowStyle,
    /// Optional label text.
    pub(crate) label: Option<UserText>,
    /// Font used for the label text (captured at `@->` parse time).
    ///
    /// Per spec, `font-family` must be written as a `<text>` attribute directly
    /// (not in `<style>`) to avoid CSS injection. This field carries the font
    /// that was active at the `@->` declaration site.
    pub(crate) label_font: FontSpec,
}

impl Arrow {
    /// Construct an arrow.
    pub(crate) fn new(
        from: ArrowEnd,
        to: ArrowEnd,
        style: ArrowStyle,
        label: Option<UserText>,
        label_font: FontSpec,
    ) -> Self {
        Self {
            from,
            to,
            style,
            label,
            label_font,
        }
    }

    /// Propagate a CLI/WASM `--font-size` override into the label font
    /// snapshot. `@->` declarations capture the active font at parse time so
    /// each arrow needs its own update for the override to reach the SVG.
    pub(crate) fn set_label_font_size(&mut self, size: Px) {
        self.label_font.set_size(size);
    }

    /// Rewrite the start endpoint (used during anchor resolution).
    pub(crate) fn set_from(&mut self, endpoint: ArrowEnd) {
        self.from = endpoint;
    }

    /// Rewrite the end endpoint (used during anchor resolution).
    pub(crate) fn set_to(&mut self, endpoint: ArrowEnd) {
        self.to = endpoint;
    }
}

/// One endpoint of an [`Arrow`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArrowEnd {
    /// Reference to an existing anchor.
    Anchor(AnchorId),
    /// Absolute chart-coordinate point.
    Absolute(Point),
}

impl ArrowEnd {
    /// Parse one `@->` endpoint token (must begin with `@`).
    ///
    /// Recognises both `@{name}` (named anchor) and `@<digits>` (indexed
    /// anchor). Absolute-coordinate endpoints are not yet supported by the
    /// `@->` syntax — every endpoint resolves to [`ArrowEnd::Anchor`].
    pub(crate) fn parse(text: &str, location: SourceLocation) -> Result<Self, ParseError> {
        let length = u32::try_from(text.chars().count())
            .unwrap_or(u32::MAX)
            .max(1);
        let bad_syntax =
            || ParseError::with_length(location, length, ParseErrorKind::InvalidArrowSyntax);
        let stripped = text.strip_prefix('@').ok_or_else(bad_syntax)?;
        match stripped.strip_prefix('{') {
            Some(rest) => Self::parse_named(rest, location, length),
            None => Self::parse_indexed(stripped, location, length),
        }
    }

    fn parse_named(rest: &str, location: SourceLocation, length: u32) -> Result<Self, ParseError> {
        let name_str = rest.strip_suffix('}').ok_or_else(|| {
            ParseError::with_length(location, length, ParseErrorKind::InvalidArrowSyntax)
        })?;
        let name = AnchorName::parse(name_str).map_err(|error| {
            // Same translation as in the inline anchor scanner: name text
            // begins two chars after the `@` (`@` + `{`). Inner offset → col
            // offset of the offending char.
            let (col_off, len) = match error.char_offset() {
                Some(offset) => (2u32.saturating_add(offset), 1),
                None => (
                    2,
                    u32::try_from(name_str.chars().count()).unwrap_or(u32::MAX),
                ),
            };
            ParseError::with_length(
                SourceLocation::new(location.line(), location.column() + col_off),
                len,
                ParseErrorKind::InvalidAnchorName(error),
            )
        })?;
        Ok(Self::Anchor(AnchorId::Named(name)))
    }

    fn parse_indexed(
        text: &str,
        location: SourceLocation,
        length: u32,
    ) -> Result<Self, ParseError> {
        let value: u32 = text.parse().map_err(|_| {
            ParseError::with_length(location, length, ParseErrorKind::InvalidArrowSyntax)
        })?;
        Ok(Self::Anchor(AnchorId::Indexed(value)))
    }
}

/// Visual style for an [`Arrow`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ArrowStyle {
    /// Stroke color.
    pub(crate) color: Color,
    /// Stroke width.
    pub(crate) width: Px,
    /// Dash pattern.
    pub(crate) line: LineDashStyle,
    /// Arrow head placement.
    pub(crate) head: ArrowHead,
}

impl ArrowStyle {
    /// Construct a style with all fields explicit.
    pub(crate) fn new(color: Color, width: Px, line: LineDashStyle, head: ArrowHead) -> Self {
        Self {
            color,
            width,
            line,
            head,
        }
    }
}

/// Dash pattern for arrows and other strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineDashStyle {
    /// Solid stroke (default).
    Solid,
    /// Dashed stroke.
    Dashed,
    /// Dotted stroke.
    Dotted,
}

impl LineDashStyle {
    /// Parse a dash-style keyword (`solid` / `dashed` / `dotted`). Returns
    /// `None` when `token` is not a dash keyword so the caller can fall back
    /// to other attribute parsing.
    pub(crate) fn from_keyword(token: &str) -> Option<Self> {
        match token {
            "solid" => Some(Self::Solid),
            "dashed" => Some(Self::Dashed),
            "dotted" => Some(Self::Dotted),
            _ => None,
        }
    }
}

/// Where arrow heads appear on an [`Arrow`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrowHead {
    /// Head only at the destination end.
    EndOnly,
    /// Heads at both ends.
    BothEnds,
    /// No arrow heads (line segment only).
    None,
}

impl ArrowHead {
    /// Parse a `head=...` keyword value. Carries the bad value text in the
    /// resulting error so the rendered message can quote it.
    pub(crate) fn from_keyword(value: &str, location: SourceLocation) -> Result<Self, ParseError> {
        match value {
            "end" => Ok(Self::EndOnly),
            "both" => Ok(Self::BothEnds),
            "none" => Ok(Self::None),
            _ => Err(ParseError::new(
                location,
                ParseErrorKind::UnknownArrowAttribute(value.to_owned()),
            )),
        }
    }
}

#[cfg(test)]
mod tests;
