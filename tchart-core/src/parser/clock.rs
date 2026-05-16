//! `@clock(...)` attribute parser.
//!
//! Parses the comma-separated key/value list inside `@clock(...)` into a
//! [`ClockSpec`]. Per-call mark overrides are bundled into [`LocalMarkOptions`]
//! and handed to [`Parser::resolve_clock_mark_style`] which combines them with
//! the global `@clockmark_*` defaults (priority described on that method).

use std::num::NonZeroU32;

use crate::clock::{ClockEdge, ClockPhase, ClockPulse, ClockSpec};
use crate::color::Color;
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::units::Px;

use super::attr;
use super::directive::normalise;
use super::state::{ASCII_SPACE_OR_TAB, Parser};

/// Per-`@clock(...)` triangle-style overrides.
///
/// Parameter object: bundles the four optional per-call overrides that
/// [`Parser::resolve_clock_mark_style`] consumes in one call. There are no
/// inter-field invariants and the type is never stored or aliased — it lives
/// inside one [`ClockSpecParser`] and is fed to a single resolver call. This
/// is the closed-list "parameter object" exception in the field-visibility
/// rules; outside that exception, struct fields would have to be private.
#[derive(Debug, Default)]
pub(super) struct LocalMarkOptions {
    pub position: Option<f32>,
    pub height: Option<Px>,
    pub width: Option<Px>,
    pub color: Option<Color>,
}

/// Entry point for parsing a `@clock(...)` body.
///
/// Lives as an associated function on [`ClockSpecParser`] (the type being
/// returned, post-resolve, is [`ClockSpec`] but the immediate parse target is
/// `Self`). State is required so per-call overrides can be merged with the
/// global `@clockmark_*` defaults.
impl ClockSpecParser {
    /// `inner_location` is the source location of `inner[0]` — i.e. the column
    /// immediately after the opening `(` of `@clock(...)`. Used to compute
    /// per-attribute locations so error reports point at the offending token,
    /// not at `@clock` itself.
    pub(super) fn parse(
        inner: &str,
        inner_location: SourceLocation,
        state: &Parser,
    ) -> Result<ClockSpec, ParseError> {
        let mut spec_parser = Self::default();
        // Walk byte-wise through `inner` so we can recover the source column of
        // each comma-separated segment. The directive grammar is ASCII for
        // keys / numeric values / hex colours, and named-colour names are also
        // ASCII; treat bytes==chars within ASCII and use chars().count() for
        // anything that does slip through (which only affects column accuracy
        // for non-ASCII garbage that would error out anyway).
        let mut byte_pos = 0usize;
        for segment in inner.split(',') {
            let leading_ws_bytes =
                segment.len() - segment.trim_start_matches(ASCII_SPACE_OR_TAB).len();
            let trimmed = segment.trim();
            let token_byte_in_inner = byte_pos + leading_ws_bytes;
            let token_col_offset = inner[..token_byte_in_inner].chars().count() as u32;
            let token_len = trimmed.chars().count() as u32;
            let token_location = SourceLocation::new(
                inner_location.line(),
                inner_location.column() + token_col_offset,
            );
            spec_parser.consume_attr(trimmed, token_location, token_len)?;
            byte_pos += segment.len() + 1; // +1 for the ',' (no-op on the last iteration).
        }
        spec_parser.into_spec(state)
    }
}

#[derive(Default)]
pub(super) struct ClockSpecParser {
    edge: Option<ClockEdge>,
    low_units: Option<NonZeroU32>,
    high_units: Option<NonZeroU32>,
    start: Option<ClockPhase>,
    local_mark: LocalMarkOptions,
}

/// Generate a setter that enforces single assignment for one attribute slot of
/// `@clock(...)`. If the slot is already filled, returns `ClockInvalidAttribute`
/// pointing at the duplicate token. Field paths may be nested (e.g.
/// `local_mark.position`).
macro_rules! once_setter {
    ($name:ident, $type:ty, $($field:ident).+) => {
        fn $name(
            &mut self,
            value: $type,
            token: &str,
            location: SourceLocation,
            length: u32,
        ) -> Result<(), ParseError> {
            if self.$($field).+.is_some() {
                return Err(invalid_attribute_error(token, location, length));
            }
            self.$($field).+ = Some(value);
            Ok(())
        }
    };
}

impl ClockSpecParser {
    fn consume_attr(
        &mut self,
        trimmed: &str,
        location: SourceLocation,
        length: u32,
    ) -> Result<(), ParseError> {
        if trimmed.is_empty() {
            return Ok(());
        }
        if let Some(value) = ClockEdge::from_keyword(trimmed) {
            return self.set_edge(value, trimmed, location, length);
        }
        let (key, raw_value) = attr::split_key_value(trimmed)
            .ok_or_else(|| invalid_attribute_error(trimmed, location, length))?;
        self.apply_key_value(&normalise(key), raw_value, trimmed, location, length)
    }

    fn apply_key_value(
        &mut self,
        key: &str,
        raw_value: &str,
        token: &str,
        location: SourceLocation,
        length: u32,
    ) -> Result<(), ParseError> {
        match key {
            "_" => self.set_low_units(
                parse_pulse(raw_value, token, location, length)?,
                token,
                location,
                length,
            ),
            "~" => self.set_high_units(
                parse_pulse(raw_value, token, location, length)?,
                token,
                location,
                length,
            ),
            "start" => self.set_start(
                parse_phase(raw_value, token, location, length)?,
                token,
                location,
                length,
            ),
            "mark_position" => self.set_position(
                parse_mark_position(raw_value, token, location, length)?,
                token,
                location,
                length,
            ),
            "mark_height" => self.set_height(
                Px(parse_positive_finite(raw_value, token, location, length)?),
                token,
                location,
                length,
            ),
            "mark_width" => self.set_width(
                Px(parse_positive_finite(raw_value, token, location, length)?),
                token,
                location,
                length,
            ),
            "mark_color" => self.set_color(
                parse_color(raw_value, token, location, length)?,
                token,
                location,
                length,
            ),
            _ => Err(invalid_attribute_error(token, location, length)),
        }
    }

    once_setter!(set_edge, ClockEdge, edge);
    once_setter!(set_low_units, NonZeroU32, low_units);
    once_setter!(set_high_units, NonZeroU32, high_units);
    once_setter!(set_start, ClockPhase, start);
    once_setter!(set_position, f32, local_mark.position);
    once_setter!(set_height, Px, local_mark.height);
    once_setter!(set_width, Px, local_mark.width);
    once_setter!(set_color, Color, local_mark.color);

    fn into_spec(self, state: &Parser) -> Result<ClockSpec, ParseError> {
        let mark_style = state.resolve_clock_mark_style(&self.local_mark);
        let edge = self.edge.unwrap_or(ClockEdge::None);
        Ok(ClockSpec::new(
            edge,
            ClockPulse::new(
                self.low_units.unwrap_or(NonZeroU32::MIN),
                self.high_units.unwrap_or(NonZeroU32::MIN),
            ),
            self.start.unwrap_or(ClockPhase::StartLow),
            mark_style,
        ))
    }
}

// --------------------------------------------------------------- value parsers
//
// Pure value conversions, not methods. `token` / `location` / `length` are
// passed directly so each value-error wraps the offending attribute's verbatim
// text and source range.

fn invalid_attribute_error(token: &str, location: SourceLocation, length: u32) -> ParseError {
    ParseError::with_length(
        location,
        length,
        ParseErrorKind::ClockInvalidAttribute(token.to_owned()),
    )
}

fn parse_pulse(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<NonZeroU32, ParseError> {
    let parsed: u32 = value
        .trim()
        .parse()
        .map_err(|_| invalid_attribute_error(token, location, length))?;
    NonZeroU32::new(parsed).ok_or_else(|| invalid_attribute_error(token, location, length))
}

fn parse_f32(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<f32, ParseError> {
    let parsed: f32 = value
        .trim()
        .parse()
        .map_err(|_| invalid_attribute_error(token, location, length))?;
    if !parsed.is_finite() {
        return Err(invalid_attribute_error(token, location, length));
    }
    Ok(parsed)
}

/// Parse a `mark_position` literal. Spec (`docs/spec/tcml-format.md` §「@clock」)
/// requires `0.0..=1.0`.
fn parse_mark_position(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<f32, ParseError> {
    let parsed = parse_f32(value, token, location, length)?;
    if !(0.0..=1.0).contains(&parsed) {
        return Err(invalid_attribute_error(token, location, length));
    }
    Ok(parsed)
}

/// Parse a strictly positive finite f32 literal (rejects zero, negative,
/// NaN, and ±infinity).
fn parse_positive_finite(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<f32, ParseError> {
    let parsed = parse_f32(value, token, location, length)?;
    if parsed <= 0.0 {
        return Err(invalid_attribute_error(token, location, length));
    }
    Ok(parsed)
}

fn parse_phase(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<ClockPhase, ParseError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => Ok(ClockPhase::StartLow),
        "high" => Ok(ClockPhase::StartHigh),
        _ => Err(invalid_attribute_error(token, location, length)),
    }
}

fn parse_color(
    value: &str,
    token: &str,
    location: SourceLocation,
    length: u32,
) -> Result<Color, ParseError> {
    Color::parse(value.trim()).map_err(|_| invalid_attribute_error(token, location, length))
}

/// Split the body following `@clock` into the `(...)` attribute list and the
/// optional inline signal-line tail.
///
/// Accepts:
/// - `""` / whitespace-only — no attribute list, no inline tail.
/// - `(<attrs>)` — only attribute list.
/// - `(<attrs>) <tail>` — attribute list plus an inline signal line.
/// - `<tail>` (no parens, non-empty) — bare inline signal line (equivalent to
///   `@clock()`), e.g. `@clock ck`.
///
/// Returns `(inner_attrs, tail)`. `tail` is trimmed of ASCII space/tab.
///
/// INVARIANT: `find(')')` terminates the attribute list at the first `)`. The
/// TCML spec (`docs/spec/tcml-format.md` §「@clock」) defines `@clock(...)`
/// attribute values as comma-separated scalar tokens (`pos`/`neg`/`both`/`none`,
/// `_=N`, `~=N`, `start=...`, `mark_position=...`, `mark_height=...`,
/// `mark_width=...`, `mark_color=#RRGGBB[AA]` or named color) — none of these
/// can contain a literal `)`. If the spec ever grows a value form that may
/// embed `)` (e.g. an `rgb(r,g,b)` color literal inside `@clock(...)`), this
/// scanner must be rewritten to track parenthesis nesting.
pub(super) fn split_clock_args_and_inline(
    args: &str,
    location: SourceLocation,
) -> Result<(&str, SourceLocation, &str), ParseError> {
    let leading_ws_bytes = args.len() - args.trim_start_matches(ASCII_SPACE_OR_TAB).len();
    let trimmed = args.trim_matches(ASCII_SPACE_OR_TAB);
    if trimmed.is_empty() {
        return Ok(("", location, ""));
    }
    let Some(rest_after_open) = trimmed.strip_prefix('(') else {
        // No parens at all — treat the whole tail as an inline signal line
        // applied to `@clock()` (edge=none) defaults.
        let inline_location = SourceLocation::new(
            location.line(),
            location.column() + args[..leading_ws_bytes].chars().count() as u32,
        );
        return Ok(("", inline_location, trimmed));
    };
    // Source column of `inner[0]` = column of `args[0]` + chars before `inner`
    // in `args` (leading whitespace + the opening `(`). All those chars are
    // ASCII in well-formed input, but `chars().count()` keeps the math right
    // even when garbage whitespace leaks in.
    let inner_byte_offset = leading_ws_bytes + 1;
    let inner_col_offset = args[..inner_byte_offset].chars().count() as u32;
    let inner_location = SourceLocation::new(location.line(), location.column() + inner_col_offset);
    let close = rest_after_open.find(')').ok_or_else(|| {
        // Underline the entire `(...` remainder so users can see which span
        // is malformed (rather than a 0-length caret at `(`).
        let trimmed_length = u32::try_from(trimmed.chars().count()).unwrap_or(u32::MAX);
        ParseError::with_length(
            location,
            trimmed_length,
            ParseErrorKind::ClockInvalidAttribute(trimmed.to_owned()),
        )
    })?;
    let inner = &rest_after_open[..close];
    let tail = rest_after_open[close + 1..].trim_matches(ASCII_SPACE_OR_TAB);
    Ok((inner, inner_location, tail))
}
