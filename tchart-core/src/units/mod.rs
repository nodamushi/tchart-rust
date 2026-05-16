//! Length units for tchart-core.
//!
//! See `docs/spec/types.md` §1.1 (Length) and §1.2 (Px).
//!
//! Two domain types live in this module:
//!
//! * [`Length`] — user-facing, may be `lh` (line-height units) or `px`.
//!   Layout never stores [`Length`]; values are resolved into [`Px`] at the parser/style boundary.
//! * [`Px`] — resolved scalar length used by every layout calculation and SVG output.

use std::ops::{Add, Div, Mul, Sub};

use crate::errors::{ParseError, ParseErrorKind, SourceLocation};

/// Resolved pixel scalar.
///
/// See `docs/spec/types.md` §1.2.
///
/// Holds a single `f32`. Construct directly with `Px(value)` or use [`Px::ZERO`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Px(pub f32);

impl Px {
    /// The zero pixel constant.
    pub(crate) const ZERO: Px = Px(0.0);

    /// Returns the raw `f32` value for SVG output or platform API calls.
    pub fn to_f32(self) -> f32 {
        self.0
    }

    /// Construct a [`Px`] from a value that must be a strictly positive finite
    /// `f32`. Zero, negative, `NaN`, and infinite values are rejected.
    ///
    /// Returned on failure: the rejected value (so callers can put it into a
    /// crate-specific error type or message). Used as the shared validation
    /// kernel for both the CLI `--font-size` flag and the wasm `fontSize`
    /// option so the two surfaces cannot drift apart.
    pub fn try_from_positive_finite(value: f32) -> Result<Px, f32> {
        if value.is_finite() && value > 0.0 {
            Ok(Px(value))
        } else {
            Err(value)
        }
    }

    /// Returns the larger of two values.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the same NaN-handling behaviour as [`f32::max`].
    pub(crate) fn max(self, other: Px) -> Px {
        if other.0 > self.0 { other } else { self }
    }

    /// Returns the smaller of two values.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the same NaN-handling behaviour as [`f32::min`].
    pub(crate) fn min(self, other: Px) -> Px {
        if other.0 < self.0 { other } else { self }
    }

    /// Parse a token that may carry a trailing `px` suffix (e.g. `"2"` or
    /// `"2px"`). Returns `error_kind` (wrapped at `location`) when the numeric
    /// part fails to parse.
    pub(crate) fn parse_with_optional_unit(
        token: &str,
        location: SourceLocation,
        error_kind: ParseErrorKind,
    ) -> Result<Self, ParseError> {
        let stripped = token.strip_suffix("px").unwrap_or(token);
        stripped
            .parse::<f32>()
            .map(Px)
            .map_err(|_| ParseError::new(location, error_kind))
    }
}

impl Add for Px {
    type Output = Px;
    fn add(self, rhs: Px) -> Px {
        Px(self.0 + rhs.0)
    }
}

impl Sub for Px {
    type Output = Px;
    fn sub(self, rhs: Px) -> Px {
        Px(self.0 - rhs.0)
    }
}

impl Mul<f32> for Px {
    type Output = Px;
    fn mul(self, rhs: f32) -> Px {
        Px(self.0 * rhs)
    }
}

impl Div<f32> for Px {
    type Output = Px;
    fn div(self, rhs: f32) -> Px {
        Px(self.0 / rhs)
    }
}

/// User-facing length with an attached unit.
///
/// See `docs/spec/types.md` §1.1.
///
/// Construct via [`Length::new_lh`] / [`Length::new_px`] which reject negatives,
/// and resolve to [`Px`] via [`Length::resolve`] before entering layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Length {
    /// Multiples of the current line-height (e.g. `2.5` for `@skip(2.5)`).
    Lh(f32),
    /// Absolute pixels (e.g. `20.0` for `@skip(20px)`).
    Px(f32),
}

/// Error returned when constructing a [`Length`] from an invalid scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum LengthError {
    /// The supplied value is negative.
    #[error("length must be non-negative")]
    Negative,
    /// The supplied value is `NaN` or infinite.
    #[error("length must be a finite number")]
    NotFinite,
}

impl Length {
    /// Build a line-height length, rejecting negative or non-finite values.
    pub(crate) fn new_lh(value: f32) -> Result<Self, LengthError> {
        validate_length_value(value)?;
        Ok(Length::Lh(value))
    }

    /// Build a pixel length, rejecting negative or non-finite values.
    pub(crate) fn new_px(value: f32) -> Result<Self, LengthError> {
        validate_length_value(value)?;
        Ok(Length::Px(value))
    }

    /// Resolve to absolute pixels using the current line-height.
    pub(crate) fn resolve(self, line_height: Px) -> Px {
        match self {
            Length::Lh(value) => line_height * value,
            Length::Px(value) => Px(value),
        }
    }

    /// Parse a `@skip(...)` argument (e.g. `"2"`, `"2.5"`, `"20px"`).
    ///
    /// Returns `Ok(None)` for a zero amount (caller should not append a row),
    /// `Err` for negative / non-finite values or malformed numerics.
    pub(crate) fn parse_skip_amount(
        inner: &str,
        location: SourceLocation,
    ) -> Result<Option<Self>, ParseError> {
        let length = u32::try_from(inner.chars().count()).unwrap_or(u32::MAX);
        let invalid = |reason_inner: &str| {
            ParseError::with_length(
                location,
                length,
                ParseErrorKind::InvalidSkipAmount(reason_inner.to_owned()),
            )
        };
        let unit = SkipUnit::detect(inner);
        let value: f32 = unit.numeric_part().parse().map_err(|_| invalid(inner))?;
        if !value.is_finite() || value < 0.0 {
            return Err(invalid(inner));
        }
        if value == 0.0 {
            return Ok(None);
        }
        unit.into_length(value).map(Some).map_err(|error| {
            ParseError::with_length(location, length, ParseErrorKind::InvalidLength(error))
        })
    }
}

/// Unit suffix recognised by [`Length::parse_skip_amount`]. Carries the
/// numeric slice so the caller does not re-strip the suffix.
enum SkipUnit<'a> {
    /// Bare value with no suffix — interpreted as `lh` (line-heights).
    Lh(&'a str),
    /// Value followed by `px`.
    Px(&'a str),
}

impl<'a> SkipUnit<'a> {
    fn detect(inner: &'a str) -> Self {
        // `px` suffix is ASCII case-insensitive (`px` / `Px` / `pX` / `PX`).
        // See `docs/spec/tcml-format.md` §「@skip」.
        if let Some(rest) = strip_ascii_case_insensitive_suffix(inner, "px") {
            Self::Px(rest.trim())
        } else {
            Self::Lh(inner)
        }
    }

    fn numeric_part(&self) -> &'a str {
        match self {
            Self::Lh(value) | Self::Px(value) => value,
        }
    }

    fn into_length(self, value: f32) -> Result<Length, LengthError> {
        match self {
            Self::Lh(_) => Length::new_lh(value),
            Self::Px(_) => Length::new_px(value),
        }
    }
}

fn validate_length_value(value: f32) -> Result<(), LengthError> {
    if !value.is_finite() {
        return Err(LengthError::NotFinite);
    }
    if value < 0.0 {
        return Err(LengthError::Negative);
    }
    Ok(())
}

/// Strip an ASCII case-insensitive suffix from `input`. Returns `None` when
/// the suffix is not present or `input` is shorter than `suffix`.
///
/// Used to allow length-unit suffixes such as `px` / `Px` / `PX` without
/// switching to a general-purpose case-folding library. The function only
/// matches when the trailing bytes are pure ASCII; multibyte UTF-8 cannot
/// fool the comparison.
///
/// The byte-level UTF-8 boundary reasoning is the reason this lives as a
/// named helper rather than inline at the one call site: keeping the byte
/// indexing in one place makes it easier to audit.
fn strip_ascii_case_insensitive_suffix<'a>(input: &'a str, suffix: &str) -> Option<&'a str> {
    let input_bytes = input.as_bytes();
    let suffix_bytes = suffix.as_bytes();
    if input_bytes.len() < suffix_bytes.len() {
        return None;
    }
    let split_at = input_bytes.len() - suffix_bytes.len();
    let tail = &input_bytes[split_at..];
    if !tail.eq_ignore_ascii_case(suffix_bytes) {
        return None;
    }
    // `split_at` lands on a UTF-8 boundary because every byte in `tail`
    // matched an ASCII byte (single-byte codepoint).
    Some(&input[..split_at])
}

#[cfg(test)]
mod tests;
