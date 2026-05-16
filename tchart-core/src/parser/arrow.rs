//! `@-> (from, to, attrs...) label` parser.
//!
//! The surface entry point lives on [`PendingArrow`] (the type returned),
//! not as a free function. Endpoint tokens are parsed via [`ArrowEnd::parse`].
//! Attribute keywords delegate to the relevant types' `from_keyword`
//! constructors.

use std::borrow::Cow;

use crate::arrow::{ArrowEnd, ArrowHead, ArrowStyle, LineDashStyle};
use crate::color::Color;
use crate::errors::TextError;
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::style::ChartStyle;
use crate::text::FontSpec;
use crate::text::UserText;
use crate::units::Px;

use super::attr;
use super::state::PendingArrow;
use super::text_quote::QuotedToken;

impl PendingArrow {
    /// Parse one `@->` argument string (everything after the `@->`) into a
    /// pending arrow. The chart style supplies fallback values for any
    /// attribute the user did not specify.
    ///
    /// The trailing `[<label text>]` may be a `"..."`-quoted string that
    /// continues on the next source lines. The parser walks `lines` starting
    /// at `index` (the row that contains the directive) and consumes
    /// additional rows for the closing `"`. Returns the resolved arrow and
    /// the number of source lines consumed (always `>= 1`).
    pub(super) fn parse(
        args: &str,
        location: SourceLocation,
        chart_style: &ChartStyle,
        lines: &[&str],
        index: usize,
    ) -> Result<(Self, usize), ParseError> {
        let parts = ArrowLineParts::split(args.trim());
        let inner = attr::strip_parens(parts.head, location, ParseErrorKind::InvalidArrowSyntax)?;
        // Compute the source column of `inner[0]` so per-token errors can pin
        // their caret on the offender rather than on `@->` itself. `args`
        // starts immediately after `@->`, hence the `+ 3` offset against the
        // `@` column carried by `location`.
        let args_column = location.column().saturating_add(3);
        let head_byte_offset = args.find(parts.head).unwrap_or(0);
        let head_col = args_column + args[..head_byte_offset].chars().count() as u32;
        let inner_byte_in_head = parts.head.find('(').map(|paren| paren + 1).unwrap_or(0);
        let inner_col_offset = parts.head[..inner_byte_in_head].chars().count() as u32;
        let inner_location = SourceLocation::new(location.line(), head_col + inner_col_offset);
        let token_iter = SegmentLocations::new(inner, inner_location);
        let mut tokens = token_iter.collect::<Vec<_>>().into_iter();
        let from_seg = tokens
            .next()
            .ok_or_else(|| ParseError::new(location, ParseErrorKind::InvalidArrowSyntax))?;
        let to_seg = tokens
            .next()
            .ok_or_else(|| ParseError::new(location, ParseErrorKind::InvalidArrowSyntax))?;
        let from = ArrowEnd::parse(from_seg.text, from_seg.location)?;
        let to = ArrowEnd::parse(to_seg.text, to_seg.location)?;
        let mut attrs = ArrowAttrs::collect(tokens, location)?;
        let consumed = attrs.merge_trailing_label(parts.label, location, lines, index)?;
        let label = attrs.label.take();
        let style = attrs.into_style(chart_style);
        let label_font: FontSpec = chart_style.canvas().font().clone();
        Ok((
            Self::new(
                from,
                to,
                style,
                label,
                label_font,
                from_seg.location,
                to_seg.location,
            ),
            consumed,
        ))
    }
}

/// One comma-separated segment of an `@->` argument list, with the source
/// location and character length of the trimmed token. Empty segments are
/// preserved so duplicate `,` does not silently shift the offset math; the
/// caller is responsible for skipping segments whose `text` is empty.
#[derive(Clone, Copy)]
pub(super) struct ArrowSegment<'a> {
    pub(super) text: &'a str,
    pub(super) location: SourceLocation,
    pub(super) length: u32,
}

struct SegmentLocations<'a> {
    inner: &'a str,
    inner_location: SourceLocation,
    byte_pos: usize,
    done: bool,
}

impl<'a> SegmentLocations<'a> {
    fn new(inner: &'a str, inner_location: SourceLocation) -> Self {
        Self {
            inner,
            inner_location,
            byte_pos: 0,
            done: false,
        }
    }
}

impl<'a> Iterator for SegmentLocations<'a> {
    type Item = ArrowSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let remainder = &self.inner[self.byte_pos..];
        let (segment, advance) = match remainder.find(',') {
            Some(idx) => (&remainder[..idx], idx + 1),
            None => {
                self.done = true;
                (remainder, remainder.len())
            }
        };
        let leading_ws_bytes = segment.len() - segment.trim_start().len();
        let trimmed = segment.trim();
        let token_col_offset = self.inner[..self.byte_pos + leading_ws_bytes]
            .chars()
            .count() as u32;
        let location = SourceLocation::new(
            self.inner_location.line(),
            self.inner_location.column() + token_col_offset,
        );
        let length = u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX);
        self.byte_pos += advance;
        Some(ArrowSegment {
            text: trimmed,
            location,
            length,
        })
    }
}

/// Result of [`ArrowLineParts::split`]: the `(...)` head and the trailing
/// label text.
struct ArrowLineParts<'a> {
    head: &'a str,
    label: &'a str,
}

impl<'a> ArrowLineParts<'a> {
    fn split(input: &'a str) -> Self {
        if let Some(close) = input.rfind(')') {
            return Self {
                head: input[..=close].trim(),
                label: input[close + 1..].trim(),
            };
        }
        Self {
            head: input,
            label: "",
        }
    }
}

#[derive(Default)]
struct ArrowAttrs {
    color: Option<Color>,
    width: Option<Px>,
    line: Option<LineDashStyle>,
    head: Option<ArrowHead>,
    label: Option<UserText>,
}

impl ArrowAttrs {
    /// Parse every comma-separated attribute token (skipping empties).
    /// Duplicate attributes raise [`ParseErrorKind::DuplicateArrowAttribute`]
    /// pinned to the offending token's source range.
    fn collect<'a>(
        tokens: impl IntoIterator<Item = ArrowSegment<'a>>,
        fallback_location: SourceLocation,
    ) -> Result<Self, ParseError> {
        let mut attrs = Self::default();
        for segment in tokens {
            if segment.text.is_empty() {
                continue;
            }
            attrs.apply_token(segment, fallback_location)?;
        }
        Ok(attrs)
    }

    fn apply_token(
        &mut self,
        segment: ArrowSegment<'_>,
        fallback_location: SourceLocation,
    ) -> Result<(), ParseError> {
        if let Some((key, value)) = attr::split_key_value(segment.text) {
            return self.apply_keyed(key, value, segment, fallback_location);
        }
        self.apply_positional(segment, fallback_location)
    }

    /// Dispatch a `key=value` attribute token. The key is normalised to lower
    /// case and with `-` collapsed to `_` so `Color=red` and `color=red` are
    /// equivalent (per the keyed-attribute normalisation rules described in
    /// `docs/spec/tcml-format.md`).
    fn apply_keyed(
        &mut self,
        key: &str,
        value: &str,
        segment: ArrowSegment<'_>,
        fallback_location: SourceLocation,
    ) -> Result<(), ParseError> {
        let unknown = || {
            ParseError::with_length(
                segment.location,
                segment.length,
                ParseErrorKind::UnknownArrowAttribute(segment.text.to_owned()),
            )
        };
        match normalise_key(key).as_ref() {
            "color" => self.set_color(Color::parse(value).map_err(|_| unknown())?, segment),
            "width" => self.set_width(
                Px::parse_with_optional_unit(
                    value,
                    segment.location,
                    ParseErrorKind::UnknownArrowAttribute(segment.text.to_owned()),
                )?,
                segment,
            ),
            "style" => self.set_line(
                LineDashStyle::from_keyword(value).ok_or_else(unknown)?,
                segment,
            ),
            "head" => self.set_head(ArrowHead::from_keyword(value, segment.location)?, segment),
            "label" => self.set_label(
                UserText::parse(value).map_err(|error| {
                    invalid_text_to_parse_error(error, value, fallback_location)
                })?,
                segment,
            ),
            _ => Err(unknown()),
        }
    }

    /// Apply the trailing `[<text>]` form (the text after the closing `)`) to
    /// the attribute set, returning the number of source lines consumed
    /// (always `>= 1`).
    ///
    /// When the trailing text begins with an unescaped `"` it is treated as a
    /// quoted string that may continue across line boundaries; `QuotedToken`
    /// is used to walk `lines` starting at `index + 1` for the closing `"`,
    /// matching the multi-line behaviour of `@title "..."`. Newlines inside
    /// the resulting `UserText` are preserved verbatim — the SVG and WaveDrom
    /// emitters decide how to format them (the WaveDrom edge string joins
    /// them with a single space).
    ///
    /// If both `label=` and the trailing form are supplied for the same arrow
    /// this returns [`ParseErrorKind::DuplicateArrowAttribute`].
    fn merge_trailing_label(
        &mut self,
        trailing: &str,
        location: SourceLocation,
        lines: &[&str],
        index: usize,
    ) -> Result<usize, ParseError> {
        if trailing.is_empty() {
            return Ok(1);
        }
        let (text, consumed) = if let Some(after_open) = trailing.strip_prefix('"') {
            let token = QuotedToken::collect(lines, index, after_open, location)?;
            (token.text, token.consumed_lines)
        } else {
            (trailing.to_owned(), 1)
        };
        let user_text = UserText::parse(&text)
            .map_err(|error| invalid_text_to_parse_error(error, &text, location))?;
        // The trailing label has no per-token slice inside the `inner` body,
        // so use the directive head's location and the trailing text's length
        // for the dup-error caret. The duplicate is rare and only fires when
        // both `label=` and a trailing label coexist.
        let trailing_length = u32::try_from(trailing.chars().count()).unwrap_or(u32::MAX);
        let dup_segment = ArrowSegment {
            text: trailing,
            location,
            length: trailing_length,
        };
        self.set_label(user_text, dup_segment)?;
        Ok(consumed)
    }

    /// Dispatch a positional attribute token (a bare `red`, `2px`, `dashed`
    /// without an `=`). Order: try colour, then dash style, then width.
    fn apply_positional(
        &mut self,
        segment: ArrowSegment<'_>,
        _fallback_location: SourceLocation,
    ) -> Result<(), ParseError> {
        let token = segment.text;
        if let Ok(color) = Color::parse(token) {
            return self.set_color(color, segment);
        }
        if let Some(line) = LineDashStyle::from_keyword(token) {
            return self.set_line(line, segment);
        }
        if Self::looks_like_width(token) {
            let width = Px::parse_with_optional_unit(
                token,
                segment.location,
                ParseErrorKind::UnknownArrowAttribute(segment.text.to_owned()),
            )?;
            return self.set_width(width, segment);
        }
        Err(ParseError::with_length(
            segment.location,
            segment.length,
            ParseErrorKind::UnknownArrowAttribute(segment.text.to_owned()),
        ))
    }

    /// `true` when `token` could plausibly be a width literal (number with an
    /// optional `px` suffix). Avoids feeding arbitrary keywords like `foobar`
    /// to [`Px::parse_with_optional_unit`].
    fn looks_like_width(token: &str) -> bool {
        let probe = token.strip_suffix("px").unwrap_or(token);
        let mut characters = probe.chars();
        let Some(first) = characters.next() else {
            return false;
        };
        first.is_ascii_digit() || (first == '.' && characters.clone().any(|c| c.is_ascii_digit()))
    }

    fn set_color(&mut self, color: Color, segment: ArrowSegment<'_>) -> Result<(), ParseError> {
        Self::assign_once(&mut self.color, color, segment)
    }

    fn set_width(&mut self, width: Px, segment: ArrowSegment<'_>) -> Result<(), ParseError> {
        Self::assign_once(&mut self.width, width, segment)
    }

    fn set_line(
        &mut self,
        line: LineDashStyle,
        segment: ArrowSegment<'_>,
    ) -> Result<(), ParseError> {
        Self::assign_once(&mut self.line, line, segment)
    }

    fn set_head(&mut self, head: ArrowHead, segment: ArrowSegment<'_>) -> Result<(), ParseError> {
        Self::assign_once(&mut self.head, head, segment)
    }

    fn set_label(&mut self, label: UserText, segment: ArrowSegment<'_>) -> Result<(), ParseError> {
        Self::assign_once(&mut self.label, label, segment)
    }

    /// Helper used by all `set_*` methods. Kept private to `ArrowAttrs`
    /// so the `&mut Option` argument never escapes the type.
    fn assign_once<T>(
        slot: &mut Option<T>,
        value: T,
        segment: ArrowSegment<'_>,
    ) -> Result<(), ParseError> {
        if slot.is_some() {
            return Err(ParseError::with_length(
                segment.location,
                segment.length,
                ParseErrorKind::DuplicateArrowAttribute(segment.text.to_owned()),
            ));
        }
        *slot = Some(value);
        Ok(())
    }

    fn into_style(self, chart_style: &ChartStyle) -> ArrowStyle {
        let signal = chart_style.default_signal_style();
        ArrowStyle::new(
            self.color.unwrap_or(signal.color()),
            self.width.unwrap_or(signal.stroke_width()),
            self.line.unwrap_or(LineDashStyle::Solid),
            self.head.unwrap_or(ArrowHead::EndOnly),
        )
    }
}

/// Normalise an attribute key so callers can match against canonical lower-case
/// forms. ASCII uppercase letters are folded to lowercase and `-` is collapsed
/// to `_`. Per the keyed-attribute normalisation rules in the TCML spec, `-`
/// and `_` are equivalent inside keys, and key matching is case-insensitive.
///
/// Returns the original slice borrowed when no transformation is needed (the
/// common case for canonical lower-case keys without `-`), allocating only
/// when a character must actually change.
/// Wrap a [`TextError`] from `UserText::parse` into a [`ParseError`] whose
/// caret points at the offending character within `value` (or spans the
/// whole value when the error has no specific char offset).
fn invalid_text_to_parse_error(
    error: TextError,
    value: &str,
    value_location: SourceLocation,
) -> ParseError {
    let (col_off, len) = match error.char_offset() {
        Some(offset) => (offset, 1),
        None => (0, u32::try_from(value.chars().count()).unwrap_or(u32::MAX)),
    };
    ParseError::with_length(
        SourceLocation::new(value_location.line(), value_location.column() + col_off),
        len,
        ParseErrorKind::InvalidText(error),
    )
}

fn normalise_key(key: &str) -> Cow<'_, str> {
    if key
        .bytes()
        .all(|byte| !byte.is_ascii_uppercase() && byte != b'-')
    {
        return Cow::Borrowed(key);
    }
    let normalised: String = key
        .chars()
        .map(|character| {
            let lower = character.to_ascii_lowercase();
            if lower == '-' { '_' } else { lower }
        })
        .collect();
    Cow::Owned(normalised)
}
