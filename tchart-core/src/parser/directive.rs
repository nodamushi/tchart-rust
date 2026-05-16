//! `@<name> <value>` directive model.
//!
//! Every parameter-style directive parses into a [`Directive`] enum value and
//! is then handed to [`super::state::Parser::apply_directive`] which performs
//! the matching state mutation.
//!
//! Line-shaped directives (`@title`, `@skip`, `@clock`, `@signal`,
//! `% overlay`) live as methods on [`super::state::Parser`] because they need
//! multi-line context that does not fit the value-only `Directive::parse`
//! shape.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::color::Color;
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};
use crate::style::{HorizontalAlign, SvgAttrList};
use crate::text::FontFamily;
use crate::units::{LengthError, Px};

// ---------------------------------------------------------------- Directive

/// One `@<name> <value>` parameter directive after value parsing.
///
/// Only "value-only" directives appear here — those whose semantics consist
/// of updating a single chart-style or pending-state field.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Directive {
    /// `@fontsize <px>`.
    FontSize(Px),
    /// `@lineheight <ratio>`.
    LineHeight(f32),
    /// `@capwidth <px>`. `None` = auto layout (`px <= 0`).
    CapWidth(Option<Px>),
    /// `@namepad <px>`.
    NamePad(Px),
    /// `@scale <ratio>`.
    /// `@scale` is parsed and validated, but the rendered SVG does not yet
    /// apply a transform.
    Scale(f32),
    /// `@page-margin <px>`.
    PageMargin(Px),
    /// `@step <px>`.
    Step(Px),
    /// `@slant <px>`.
    Slant(Px),
    /// `@h_space <px>` (alias: `@signal_gap`).
    SignalGap(Px),
    /// `@font <family>`.
    Font(FontFamily),
    /// `@signal_color <color>`.
    SignalColor(Color),
    /// `@signal_width <px>`.
    SignalWidth(Px),
    /// `@guide_color <color>`.
    GuideColor(Color),
    /// `@guide_width <px>`.
    GuideWidth(Px),
    /// `@bg <color|none>`. `None` clears any pending color.
    Bg(Option<Color>),
    /// `@bgcolor0 <color>`.
    BgColor0(Color),
    /// `@bgcolor1 <color>`.
    BgColor1(Color),
    /// `@highlight_style key="value" ...`.
    HighlightStyle(SvgAttrList),
    /// `@dontcare_color <color>`.
    DontcareColor(Color),
    /// `@titlealign <center|left|right>`.
    TitleAlign(HorizontalAlign),
    /// `@clockmark_position <ratio>`.
    ClockmarkPosition(f32),
    /// `@clockmark_height <px>`.
    ClockmarkHeight(Px),
    /// `@clockmark_width <px>`.
    ClockmarkWidth(Px),
    /// `@clockmark_color <color>`.
    ClockmarkColor(Color),
    /// `@overline_gap <px>`.
    OverlineGap(Px),
    /// `@overline_thickness <px>`.
    OverlineThickness(Px),
    /// `@ruler on` / `@ruler off`. Toggles the parser-state flag that
    /// controls whether subsequent signal / `@skip` rows donate ruler-line
    /// positions (see `docs/spec/tcml-format.md` §「`@ruler` の詳細」).
    Ruler(bool),
    /// `@ruler_color <color>`. Updates the parser-state color used as the
    /// snapshot for subsequent rows committed under `@ruler on`.
    RulerColor(Color),
}

impl Directive {
    /// Parse a `@<name> <value>` directive. Returns
    /// `ParseErrorKind::UnknownParameter` when the name is not registered.
    ///
    /// - `value_location` points at the first character of the directive's
    ///   argument run within the source line (used for the value-error
    ///   caret).
    /// - `name_location` points at the leading `@` of the directive (used
    ///   for the unknown-parameter caret, so the caret sits on the bad
    ///   directive name rather than on a missing/empty value).
    pub(super) fn parse(
        name: &str,
        value: &str,
        value_location: SourceLocation,
        name_location: SourceLocation,
    ) -> Result<Self, ParseError> {
        let spec = ParamSpec::lookup(name).ok_or_else(|| {
            // For unknown parameters the offending token is the directive
            // name itself, so the caret length spans the name characters.
            let name_length = u32::try_from(name.chars().count()).unwrap_or(u32::MAX);
            ParseError::with_length(
                name_location,
                name_length,
                ParseErrorKind::UnknownParameter(name.to_owned()),
            )
        })?;
        // Compute the trimmed value and its starting column so per-char
        // offsets carried in inner errors (e.g. `InvalidColor::InvalidHexDigit`
        // -> "bad nibble at offset N within `#<hex>`") can be translated into
        // exact source columns.
        let trimmed_value = value.trim();
        let leading_ws_bytes = value.len() - value.trim_start().len();
        let trimmed_value_col = value_location
            .column()
            .saturating_add(value[..leading_ws_bytes].chars().count() as u32);
        let trimmed_value_length = u32::try_from(trimmed_value.chars().count()).unwrap_or(u32::MAX);
        (spec.parse)(value).map_err(|kind| {
            let (col_off, len) = inner_kind_caret(&kind, trimmed_value, trimmed_value_length);
            ParseError::with_length(
                SourceLocation::new(value_location.line(), trimmed_value_col + col_off),
                len,
                kind,
            )
        })
    }
}

/// Translate an inner-error kind's char offset (when available) into a
/// `(column_offset, length)` pair relative to the *trimmed value*. Falls
/// back to "whole trimmed value" when the inner error has no specific
/// offset (or is not one of the offset-bearing kinds).
fn inner_kind_caret(
    kind: &ParseErrorKind,
    _trimmed_value: &str,
    trimmed_length: u32,
) -> (u32, u32) {
    let inner_offset = match kind {
        ParseErrorKind::InvalidColor(error) => error.char_offset(),
        ParseErrorKind::InvalidText(error) => error.char_offset(),
        _ => return (0, trimmed_length),
    };
    match inner_offset {
        Some(offset) => (offset, 1),
        None => (0, trimmed_length),
    }
}

// --------------------------------------------------- ParamSpec dispatch table

/// One entry of the `@name`-to-parser dispatch table.
///
/// `parse` returns the [`Directive`] without a source location; the caller
/// rewrites the location when wrapping into [`ParseError`].
struct ParamSpec {
    /// Canonical name plus aliases. All in normalised form (lowercase,
    /// underscores), matching the output of [`normalise`].
    names: &'static [&'static str],
    /// Value-string to [`Directive`] parser.
    parse: fn(&str) -> Result<Directive, ParseErrorKind>,
}

impl ParamSpec {
    /// Look up a directive parser by `@<name>`. Case-insensitive; treats
    /// `-` and `_` as equivalent.
    fn lookup(name: &str) -> Option<&'static ParamSpec> {
        static INDEX: OnceLock<HashMap<&'static str, &'static ParamSpec>> = OnceLock::new();
        let map = INDEX.get_or_init(ParamSpec::build_index);
        map.get(normalise(name).as_ref()).copied()
    }

    fn build_index() -> HashMap<&'static str, &'static ParamSpec> {
        let mut index: HashMap<&'static str, &'static ParamSpec> = HashMap::new();
        for spec in PARAM_SPECS {
            for alias in spec.names {
                assert!(
                    index.insert(alias, spec).is_none(),
                    "duplicate ParamSpec alias: {alias}",
                );
            }
        }
        index
    }
}

/// Canonicalise a directive name (lowercase ASCII, `-` becomes `_`).
///
/// Returns `Cow::Borrowed` when no transformation is needed, avoiding the
/// allocation cost on every lookup (E-02).
pub(super) fn normalise(input: &str) -> Cow<'_, str> {
    if input
        .chars()
        .all(|character| !character.is_ascii_uppercase() && character != '-')
    {
        return Cow::Borrowed(input);
    }
    Cow::Owned(
        input
            .chars()
            .map(|character| match character {
                'A'..='Z' => character.to_ascii_lowercase(),
                '-' => '_',
                other => other,
            })
            .collect(),
    )
}

/// Build a `ParamSpec` whose value parser produces a single-arg
/// [`Directive`] variant by piping the raw value through one of the
/// `parse_*` helpers. Keeps the table free of repetitive closures.
///
/// Every entry now carries the directive's canonical name so numeric-value
/// errors can be rendered as `@<name> ...` rather than via a generic
/// stringified message.
macro_rules! param_spec {
    // px-valued directive with finite-value validation only.
    (px, [$($alias:literal),+ $(,)?], $variant:ident, $field_name:literal) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(Px(parse_finite_f32($field_name, value)?))),
        }
    };
    // f32-valued directive with finite-value validation only.
    (f32, [$($alias:literal),+ $(,)?], $variant:ident, $field_name:literal) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(parse_finite_f32($field_name, value)?)),
        }
    };
    // Color-valued directive.
    (color, [$($alias:literal),+ $(,)?], $variant:ident) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(parse_color(value)?)),
        }
    };
    // SvgAttrList-valued directive.
    (svg_attrs, [$($alias:literal),+ $(,)?], $variant:ident) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(SvgAttrList::parse(value)?)),
        }
    };
    // Strictly positive Px-valued directive (gap / thickness etc.).
    (px_positive, [$($alias:literal),+ $(,)?], $variant:ident, $field_name:literal) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(Px(parse_positive_finite_f32($field_name, value)?))),
        }
    };
    // Strictly positive ratio-valued directive (dimensionless, e.g. `@scale`).
    (ratio_positive, [$($alias:literal),+ $(,)?], $variant:ident, $field_name:literal) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(parse_positive_finite_f32($field_name, value)?)),
        }
    };
    // Non-negative Px-valued directive (zero allowed).
    (px_non_negative, [$($alias:literal),+ $(,)?], $variant:ident, $field_name:literal) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: |value| Ok(Directive::$variant(parse_non_negative_px($field_name, value)?)),
        }
    };
    // Custom parser function: the closure body is supplied verbatim.
    (custom, [$($alias:literal),+ $(,)?], $function:expr) => {
        ParamSpec {
            names: &[$($alias),+],
            parse: $function,
        }
    };
}

const PARAM_SPECS: &[ParamSpec] = &[
    param_spec!(px_positive, ["fontsize", "font_size"], FontSize, "fontsize"),
    param_spec!(
        ratio_positive,
        ["lineheight", "line_height"],
        LineHeight,
        "lineheight"
    ),
    param_spec!(custom, ["capwidth", "cap_width"], parse_capwidth),
    param_spec!(px, ["namepad", "name_pad"], NamePad, "namepad"),
    param_spec!(ratio_positive, ["scale"], Scale, "scale"),
    param_spec!(px, ["page_margin"], PageMargin, "page_margin"),
    param_spec!(px_positive, ["step"], Step, "step"),
    param_spec!(px_non_negative, ["slant"], Slant, "slant"),
    param_spec!(
        px_non_negative,
        ["h_space", "signal_gap"],
        SignalGap,
        "h_space"
    ),
    param_spec!(custom, ["font"], parse_font),
    param_spec!(color, ["signal_color"], SignalColor),
    param_spec!(px, ["signal_width"], SignalWidth, "signal_width"),
    param_spec!(color, ["guide_color"], GuideColor),
    param_spec!(px, ["guide_width"], GuideWidth, "guide_width"),
    param_spec!(custom, ["bg"], parse_bg),
    param_spec!(color, ["bgcolor0"], BgColor0),
    param_spec!(color, ["bgcolor1"], BgColor1),
    param_spec!(svg_attrs, ["highlight_style"], HighlightStyle),
    param_spec!(color, ["dontcare_color"], DontcareColor),
    param_spec!(custom, ["titlealign", "title_align"], parse_title_align),
    param_spec!(
        f32,
        ["clockmark_position"],
        ClockmarkPosition,
        "clockmark_position"
    ),
    param_spec!(
        px,
        ["clockmark_height"],
        ClockmarkHeight,
        "clockmark_height"
    ),
    param_spec!(px, ["clockmark_width"], ClockmarkWidth, "clockmark_width"),
    param_spec!(color, ["clockmark_color"], ClockmarkColor),
    param_spec!(px_positive, ["overline_gap"], OverlineGap, "overline_gap"),
    param_spec!(
        px_positive,
        ["overline_thickness"],
        OverlineThickness,
        "overline_thickness"
    ),
    param_spec!(custom, ["ruler"], parse_ruler_toggle),
    param_spec!(color, ["ruler_color"], RulerColor),
];

// ----------------------------------------------------------------- value parsers

fn parse_color(value: &str) -> Result<Color, ParseErrorKind> {
    Color::parse(value).map_err(ParseErrorKind::InvalidColor)
}

/// Upper bound enforced on every length/ratio directive value. Values whose
/// absolute magnitude exceeds this constant are rejected as "numeric
/// overflow" — the practical limit for SVG-coordinate-scale rendering. The
/// limit catches both `f32::INFINITY` (returned by `str::parse` for literals
/// beyond `f32::MAX`) and absurdly large literals such as
/// `99999999999999999` whose f32 representation is finite but useless.
const MAX_REPRESENTABLE_LENGTH: f64 = 1.0e9;

/// Parse a numeric value via `f64`, rejecting `NaN` / `±∞` (mapped to
/// [`LengthError::NotFinite`]) and magnitudes beyond
/// [`MAX_REPRESENTABLE_LENGTH`] (mapped to
/// [`ParseErrorKind::NumericOverflow`]). Going through `f64` rather than
/// `f32` keeps the overflow detection independent of `f32`'s own infinity
/// threshold, so an integer literal that fits f32 but is otherwise
/// nonsensical (`99999999999999999`) is still flagged. The returned `f32`
/// is the down-cast of the validated `f64`.
fn parse_finite_f32(field_name: &str, value: &str) -> Result<f32, ParseErrorKind> {
    let trimmed = value.trim();
    let parsed: f64 = trimmed.parse().map_err(|_| {
        ParseErrorKind::NumericNotParseable(field_name.to_owned(), trimmed.to_owned())
    })?;
    if !parsed.is_finite() {
        return Err(ParseErrorKind::InvalidLength(LengthError::NotFinite));
    }
    if parsed.abs() > MAX_REPRESENTABLE_LENGTH {
        return Err(ParseErrorKind::NumericOverflow(
            field_name.to_owned(),
            parsed,
            MAX_REPRESENTABLE_LENGTH,
        ));
    }
    Ok(parsed as f32)
}

/// Parse a strictly positive finite `f32` (used for ratios such as `@scale`
/// and `@lineheight`). Returns [`ParseErrorKind::NumericNotPositive`] when
/// `value <= 0`.
fn parse_positive_finite_f32(field_name: &str, value: &str) -> Result<f32, ParseErrorKind> {
    let parsed = parse_finite_f32(field_name, value)?;
    if parsed <= 0.0 {
        return Err(ParseErrorKind::NumericNotPositive(
            field_name.to_owned(),
            parsed as f64,
        ));
    }
    Ok(parsed)
}

/// Parse a non-negative finite pixel value. Zero is accepted, negatives
/// emit [`ParseErrorKind::NumericNotNonNegative`] naming the field.
fn parse_non_negative_px(field_name: &str, value: &str) -> Result<Px, ParseErrorKind> {
    let parsed = parse_finite_f32(field_name, value)?;
    if parsed < 0.0 {
        return Err(ParseErrorKind::NumericNotNonNegative(
            field_name.to_owned(),
            parsed as f64,
        ));
    }
    Ok(Px(parsed))
}

fn parse_capwidth(value: &str) -> Result<Directive, ParseErrorKind> {
    let px = parse_finite_f32("capwidth", value)?;
    let width = if px <= 0.0 { None } else { Some(Px(px)) };
    Ok(Directive::CapWidth(width))
}

fn parse_font(value: &str) -> Result<Directive, ParseErrorKind> {
    // `FontFamily::parse` accepts CSV-style fallback lists with optional
    // `"..."` quoting per `docs/spec/tcml-format.md` §「ローカルパラメータ」
    // `font`, so the directive layer hands the raw value through after a
    // trim.
    let family = FontFamily::parse(value.trim()).map_err(ParseErrorKind::InvalidText)?;
    Ok(Directive::Font(family))
}

fn parse_bg(value: &str) -> Result<Directive, ParseErrorKind> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("none") {
        return Ok(Directive::Bg(None));
    }
    Ok(Directive::Bg(Some(parse_color(trimmed)?)))
}

fn parse_title_align(value: &str) -> Result<Directive, ParseErrorKind> {
    let trimmed = value.trim();
    HorizontalAlign::from_keyword(trimmed)
        .map(Directive::TitleAlign)
        .ok_or_else(|| ParseErrorKind::InvalidTitleAlign(trimmed.to_owned()))
}

/// Parse `@ruler on` / `@ruler off`. Empty or unknown values produce
/// [`ParseErrorKind::InvalidRulerValue`].
fn parse_ruler_toggle(value: &str) -> Result<Directive, ParseErrorKind> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("on") {
        Ok(Directive::Ruler(true))
    } else if trimmed.eq_ignore_ascii_case("off") {
        Ok(Directive::Ruler(false))
    } else {
        Err(ParseErrorKind::InvalidRulerValue(trimmed.to_owned()))
    }
}
