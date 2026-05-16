//! Color values used in styling and SVG output.
//!
//! See `docs/spec/types.md` §2.1.
//!
//! Accepted input forms (parsed by [`Color::parse`]):
//!
//! * `none`            — explicit "no color".
//! * `#rgb`            — 3 hex digits, expanded to `#rrggbb` with alpha 1.
//! * `#rrggbb`         — 6 hex digits, alpha 1.
//! * `#rrggbbaa`       — 8 hex digits including alpha.
//! * Named CSS color (case-insensitive) — see [`NAMED_COLORS`].

/// RGBA color or the explicit "no color" sentinel.
///
/// The internal representation is intentionally opaque; callers must round-trip
/// through [`Color::parse`] / [`Color::to_css_string`].
///
/// `PartialEq` / `Eq` / `Hash` compare only the value identity (the RGBA tuple
/// or the `None` sentinel). The optional CSS name attached to a parsed named
/// color is a rendering hint for [`Self::to_css_string`] and is intentionally
/// excluded from equality, so `Color::RED` and `Color::parse("red")` compare
/// as equal even though only the latter remembers the keyword `red`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Color {
    repr: ColorRepr,
}

#[derive(Debug, Clone, Copy)]
enum ColorRepr {
    /// No paint at all (`none`).
    None,
    /// Opaque or alpha-blended RGBA. When the value was parsed from a CSS
    /// named color, `name` carries the matching lowercase name so that
    /// `to_css_string()` round-trips back to the original token.
    Rgba {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
        /// Lowercase CSS name table entry, when this value originated from
        /// [`NAMED_COLORS`] (entry source is a `&'static str` literal).
        ///
        /// Excluded from `PartialEq` / `Hash`; see the `Color` doc comment.
        name: Option<&'static str>,
    },
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self.repr, other.repr) {
            (ColorRepr::None, ColorRepr::None) => true,
            (
                ColorRepr::Rgba {
                    r: r1,
                    g: g1,
                    b: b1,
                    a: a1,
                    ..
                },
                ColorRepr::Rgba {
                    r: r2,
                    g: g2,
                    b: b2,
                    a: a2,
                    ..
                },
            ) => r1 == r2 && g1 == g2 && b1 == b2 && a1 == a2,
            _ => false,
        }
    }
}

impl Eq for Color {}

impl std::hash::Hash for Color {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.repr {
            ColorRepr::None => 0u8.hash(state),
            ColorRepr::Rgba { r, g, b, a, .. } => {
                1u8.hash(state);
                r.hash(state);
                g.hash(state);
                b.hash(state);
                a.hash(state);
            }
        }
    }
}

/// Errors produced by [`Color::parse`].
///
/// `InvalidHexDigit` carries the 0-based char offset of the bad nibble
/// *within the `#`-prefixed hex literal* (i.e. `#zzz` → offset 1 for the
/// first `z`); the caller adds the `#` column to get the source column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ColorError {
    /// Input string was empty.
    #[error("color value is empty")]
    Empty,
    /// `#` literal had a length other than 3, 6, or 8 hex digits.
    #[error("hex color must have 3, 6, or 8 digits after '#'")]
    InvalidHexLength,
    /// `#` literal contained a non-hex character.
    #[error("hex color contains non-hex digit")]
    InvalidHexDigit { char_offset: u32 },
    /// Identifier was not a recognised CSS color name.
    #[error("unknown CSS color name")]
    UnknownName,
}

impl ColorError {
    /// 0-based char offset within the *parsed value* (including the `#`
    /// prefix where applicable) where the caret should land, or `None`
    /// when the error spans the value as a whole.
    pub(crate) fn char_offset(&self) -> Option<u32> {
        match self {
            Self::Empty | Self::InvalidHexLength | Self::UnknownName => None,
            // `+1` accounts for the `#` byte that prefixes the hex slice
            // passed into `parse_hex_digit` (the inner offset is measured
            // from after the `#`).
            Self::InvalidHexDigit { char_offset } => Some(char_offset.saturating_add(1)),
        }
    }
}

impl Color {
    /// The explicit "no color" value.
    pub(crate) const NONE: Color = Color {
        repr: ColorRepr::None,
    };

    /// Opaque black (`#000000`).
    pub(crate) const BLACK: Color = Color::from_rgba_const(0x00, 0x00, 0x00, 0xFF, None);

    /// Opaque red (`#FF0000`).
    pub(crate) const RED: Color = Color::from_rgba_const(0xFF, 0x00, 0x00, 0xFF, None);

    /// `#a0a0a0` — default stroke color for `@ruler` background guide lines.
    ///
    /// See `docs/spec/tcml-format.md` §「`@ruler` の詳細」.
    pub(crate) const RULER_DEFAULT: Color = Color::from_rgba_const(0xA0, 0xA0, 0xA0, 0xFF, None);

    /// `const`-evaluable RGBA constructor used by named constants and parse paths.
    ///
    /// `name` is `Some(...)` only for colors produced from the [`NAMED_COLORS`]
    /// table; it is preserved across [`Self::to_css_string`] so a parsed
    /// keyword like `red` round-trips back to the same token instead of
    /// `#ff0000`. Constants and `#rrggbb` parse paths pass `None` and emit
    /// canonical hex form.
    const fn from_rgba_const(r: u8, g: u8, b: u8, a: u8, name: Option<&'static str>) -> Color {
        Color {
            repr: ColorRepr::Rgba { r, g, b, a, name },
        }
    }

    /// Parse a CSS-style color string.
    ///
    /// Whitespace is trimmed and matching against named colors is case-insensitive.
    pub(crate) fn parse(input: &str) -> Result<Self, ColorError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ColorError::Empty);
        }
        if trimmed.eq_ignore_ascii_case("none") {
            return Ok(Color::NONE);
        }
        if let Some(hex) = trimmed.strip_prefix('#') {
            return parse_hex(hex);
        }
        lookup_named(trimmed).ok_or(ColorError::UnknownName)
    }

    /// Render as a CSS string suitable for SVG attribute values.
    ///
    /// Round-trip property: `Color::parse(c.to_css_string()) == Ok(c)` for any
    /// `Color` value, including the unnamed constants ([`Color::RED`],
    /// [`Color::BLACK`], ...). The equality only inspects the RGBA tuple so
    /// the round-trip succeeds whether or not the source value carried a CSS
    /// name. Named-color inputs (`red`, `Blue`, ...) are additionally
    /// preserved verbatim (lowercased) so SVG output keeps the original
    /// keyword instead of an equivalent `#rrggbb`.
    pub(crate) fn to_css_string(self) -> String {
        match self.repr {
            ColorRepr::None => "none".to_owned(),
            ColorRepr::Rgba {
                name: Some(name), ..
            } => name.to_owned(),
            ColorRepr::Rgba {
                r,
                g,
                b,
                a,
                name: None,
            } => {
                if a == 0xFF {
                    format!("#{r:02x}{g:02x}{b:02x}")
                } else {
                    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
                }
            }
        }
    }

    /// Return `true` if this is the [`Color::NONE`] sentinel.
    pub(crate) fn is_none(self) -> bool {
        matches!(self.repr, ColorRepr::None)
    }
}

fn parse_hex(hex: &str) -> Result<Color, ColorError> {
    match hex.len() {
        3 => parse_short_hex(hex),
        6 => parse_long_hex(hex),
        8 => parse_long_hex_with_alpha(hex),
        _ => Err(ColorError::InvalidHexLength),
    }
}

fn parse_short_hex(hex: &str) -> Result<Color, ColorError> {
    let r = parse_hex_digit(hex, 0)? * 0x11;
    let g = parse_hex_digit(hex, 1)? * 0x11;
    let b = parse_hex_digit(hex, 2)? * 0x11;
    Ok(Color::from_rgba_const(r, g, b, 0xFF, None))
}

fn parse_long_hex(hex: &str) -> Result<Color, ColorError> {
    let r = parse_hex_byte(hex, 0)?;
    let g = parse_hex_byte(hex, 2)?;
    let b = parse_hex_byte(hex, 4)?;
    Ok(Color::from_rgba_const(r, g, b, 0xFF, None))
}

fn parse_long_hex_with_alpha(hex: &str) -> Result<Color, ColorError> {
    let r = parse_hex_byte(hex, 0)?;
    let g = parse_hex_byte(hex, 2)?;
    let b = parse_hex_byte(hex, 4)?;
    let a = parse_hex_byte(hex, 6)?;
    Ok(Color::from_rgba_const(r, g, b, a, None))
}

fn parse_hex_digit(hex: &str, index: usize) -> Result<u8, ColorError> {
    hex.as_bytes()
        .get(index)
        .ok_or(ColorError::InvalidHexLength)
        .and_then(|byte| {
            char::from(*byte)
                .to_digit(16)
                .map(|value| value as u8)
                .ok_or_else(|| ColorError::InvalidHexDigit {
                    char_offset: u32::try_from(index).unwrap_or(u32::MAX),
                })
        })
}

fn parse_hex_byte(hex: &str, index: usize) -> Result<u8, ColorError> {
    let high = parse_hex_digit(hex, index)?;
    let low = parse_hex_digit(hex, index + 1)?;
    Ok((high << 4) | low)
}

fn lookup_named(name: &str) -> Option<Color> {
    NAMED_COLORS.iter().find_map(|(candidate, rgb)| {
        if candidate.eq_ignore_ascii_case(name) {
            Some(Color::from_rgba_const(
                rgb[0],
                rgb[1],
                rgb[2],
                0xFF,
                Some(candidate),
            ))
        } else {
            None
        }
    })
}

/// CSS named colors recognised by [`Color::parse`].
///
/// The list covers the commonly used SVG 1.1 / CSS 3 keyword set. Names are
/// matched case-insensitively.
pub(crate) const NAMED_COLORS: &[(&str, [u8; 3])] = &[
    ("black", [0x00, 0x00, 0x00]),
    ("silver", [0xC0, 0xC0, 0xC0]),
    ("gray", [0x80, 0x80, 0x80]),
    ("white", [0xFF, 0xFF, 0xFF]),
    ("maroon", [0x80, 0x00, 0x00]),
    ("red", [0xFF, 0x00, 0x00]),
    ("purple", [0x80, 0x00, 0x80]),
    ("fuchsia", [0xFF, 0x00, 0xFF]),
    ("magenta", [0xFF, 0x00, 0xFF]),
    ("green", [0x00, 0x80, 0x00]),
    ("lime", [0x00, 0xFF, 0x00]),
    ("olive", [0x80, 0x80, 0x00]),
    ("yellow", [0xFF, 0xFF, 0x00]),
    ("navy", [0x00, 0x00, 0x80]),
    ("blue", [0x00, 0x00, 0xFF]),
    ("teal", [0x00, 0x80, 0x80]),
    ("aqua", [0x00, 0xFF, 0xFF]),
    ("cyan", [0x00, 0xFF, 0xFF]),
    ("orange", [0xFF, 0xA5, 0x00]),
    ("pink", [0xFF, 0xC0, 0xCB]),
    ("brown", [0xA5, 0x2A, 0x2A]),
    ("gold", [0xFF, 0xD7, 0x00]),
];

#[cfg(test)]
mod tests;
