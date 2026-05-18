//! Top-level error types for tchart-core.
//!
//! See `docs/spec/types.md` §6 (file split) and §8 (parser error wrapping).
//!
//! Domain-specific error enums live in their own modules: [`ColorError`]
//! in `color`, [`NameError`] / [`TextError`] in `text`, [`LengthError`] in
//! `units`, and [`AnchorNameError`] in `anchor`. They are re-exported here
//! for convenience.

pub(crate) use crate::anchor::AnchorNameError;
pub(crate) use crate::color::ColorError;
pub(crate) use crate::text::{NameError, TextError};
pub(crate) use crate::units::LengthError;

/// Source location (1-based line and column) used in [`ParseError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SourceLocation {
    /// 1-based line number in the original TCML source.
    line: u32,
    /// 1-based column number in the original TCML source.
    column: u32,
}

impl SourceLocation {
    /// Construct a new [`SourceLocation`].
    pub(crate) const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// 1-based line number. Exposed to parser helpers that need to derive a
    /// new column for the same line.
    pub(crate) const fn line(&self) -> u32 {
        self.line
    }

    /// 1-based character column. Used by parser helpers that need to compose
    /// a new location relative to this one.
    pub(crate) const fn column(&self) -> u32 {
        self.column
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line {}, column {}", self.line, self.column)
    }
}

/// Parser error.
///
/// All TCML parser failures wrap an inner [`ParseErrorKind`] together with the
/// [`SourceLocation`] where the failure was detected and a character `length`
/// covering the offending range.
///
/// `length == 0` indicates an **insertion-point** error: the failure has no
/// width and conceptually points "between" two characters (typical for
/// `UnclosedQuote` reaching end of input).
///
/// `line()` / `column()` / `length()` / `message()` are part of the public
/// surface so CLI / editor / wasm front ends can render the rustc-style
/// 4-component error message (`docs/spec/cli.md` §パースエラー出力形式 and
/// `docs/spec/tcml-format.md` §位置情報の必須化). Core itself never formats
/// the rustc-style output — that is the front-end's responsibility.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{location}: {kind}")]
pub struct ParseError {
    /// Where the error was detected.
    location: SourceLocation,
    /// Character-unit width of the offending range. `0` is an insertion point.
    length: u32,
    /// Specific reason. Marked `#[source]` so that `Error::source()` returns
    /// the inner kind and error-chain reporters can traverse the full chain.
    #[source]
    kind: ParseErrorKind,
}

/// Specific reason for a [`ParseError`].
///
/// # Why inner error variants do not carry `#[from]`
///
/// [`ParseError`] always requires a [`SourceLocation`] (line and column in the
/// TCML source). An automatic `From<ColorError> for ParseError` conversion
/// would have nowhere to obtain the location, so the conversion would silently
/// produce an error with a meaningless `(0, 0)` position. To prevent that,
/// `#[from]` is intentionally omitted on every inner-error variant here.
/// Constructing a [`ParseError`] from any of these domain errors must always go
/// through [`ParseError::new`], which forces the caller to supply the location
/// explicitly.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub(crate) enum ParseErrorKind {
    /// A color literal failed validation.
    #[error("{0}")]
    InvalidColor(ColorError),
    /// A signal name failed validation.
    #[error("{0}")]
    InvalidName(NameError),
    /// A user text value failed validation.
    #[error("{0}")]
    InvalidText(TextError),
    /// A length value failed validation.
    #[error("{0}")]
    InvalidLength(LengthError),
    /// An anchor name failed validation.
    #[error("{0}")]
    InvalidAnchorName(AnchorNameError),
    /// The argument to `@skip` was negative or unparsable. The payload
    /// carries the offending text so renderers can quote it.
    #[error("invalid @skip amount: \"{0}\"")]
    InvalidSkipAmount(String),
    /// A `?` was reached before any anchor level had been seen.
    #[error("`?` has no preceding level anchor")]
    DontCareWithoutAnchor,
    /// The same anchor identifier was defined more than once.
    #[error("anchor defined more than once")]
    DuplicateAnchor,
    /// An arrow referenced an anchor that was never defined. The payload
    /// carries the textual form of the offending reference (`@{name}` or
    /// `@N`) so renderers can quote it.
    #[error("anchor reference is undefined: \"{0}\"")]
    UndefinedAnchor(String),
    /// More than one attribute of the same category was specified on `@->`.
    /// The payload carries the offending (second) attribute token verbatim.
    #[error("@-> has a duplicate attribute: \"{0}\"")]
    DuplicateArrowAttribute(String),
    /// `@->` contained a token that did not match any known attribute.
    /// The payload carries the offending token verbatim.
    #[error("@-> has an unknown attribute: \"{0}\"")]
    UnknownArrowAttribute(String),
    /// A parameter name was not recognised. The payload carries the
    /// offending `@<name>` (without the leading `@`).
    #[error("unknown parameter: \"@{0}\"")]
    UnknownParameter(String),
    /// `@slant` ≥ `@step`: slant must be strictly less than step. The
    /// payload carries the two current values (px, formatted to one
    /// decimal) so the message can quote them.
    #[error("slant ({1} px) must be strictly less than step ({0} px)")]
    InvalidStepSlant(f32, f32),
    /// A TCML level character was not one of the recognised symbols.
    #[error("invalid level character: {0:?}")]
    InvalidLevelChar(char),
    /// The level string of a signal row starts with a text character, not a level symbol.
    #[error("level string must start with a level symbol (_~=-?)")]
    MissingInitialLevel,
    /// `[` had no matching `]`.
    #[error("`[` has no matching `]`")]
    UnclosedHighlight,
    /// `]` had no preceding `[`.
    #[error("`]` has no matching `[`")]
    UnopenedHighlightEnd,
    /// `"..."` quoted signal name was never closed.
    #[error("`\"...\"` is not closed")]
    UnclosedQuote,
    /// `@clock(...)` had a duplicate, unknown, or malformed attribute. The
    /// payload carries the offending attribute text (verbatim, trimmed) so
    /// front ends can show users exactly which token is at fault.
    #[error("@clock has an invalid attribute: \"{0}\"")]
    ClockInvalidAttribute(String),
    /// `@->` syntax could not be parsed (missing brackets, missing comma, etc.).
    #[error("invalid @-> syntax")]
    InvalidArrowSyntax,
    /// `@signal(...)` referenced an unknown, duplicate, or malformed
    /// attribute (or the `(...)` syntax itself was broken). The payload
    /// carries the offending text (the trimmed attribute token, or the
    /// argument substring for syntax errors) so front ends can pinpoint it.
    #[error("@signal has an invalid attribute: \"{0}\"")]
    UnknownSignalAttribute(String),
    /// `% x y text` had an invalid coordinate. The payload carries the
    /// offending token verbatim (empty when the coordinate was missing
    /// entirely).
    #[error("invalid overlay coordinate: \"{0}\"")]
    InvalidOverlayCoordinate(String),
    /// `@skip(...)` could not be parsed (missing parentheses, etc.).
    #[error("invalid @skip syntax")]
    InvalidSkipSyntax,
    /// `@overline` alias received an argument; the alias takes no argument.
    /// The payload carries the offending tail (verbatim, trimmed) so the
    /// rendered message can quote it.
    #[error("@overline takes no argument; got \"{0}\". Use `@signal(overline)` for the long form")]
    InvalidOverlineSyntax(String),
    /// A numeric value parameter (`@step`, `@scale`, ...) received a value
    /// that could not be parsed as a number. Payload: directive canonical
    /// name (without the leading `@`), offending value text.
    #[error("@{0} expects a number, got \"{1}\"")]
    NumericNotParseable(String, String),
    /// A numeric value parameter received a magnitude outside the allowed
    /// finite range (absolute value capped at the architectural limit).
    /// Payload: directive canonical name, parsed value, limit.
    #[error("@{0} value {1} exceeds the magnitude limit ({2})")]
    NumericOverflow(String, f64, f64),
    /// A numeric value parameter required strictly-positive input but the
    /// caller passed zero or a negative number. Payload: directive name,
    /// the offending value.
    #[error("@{0} must be strictly positive, got {1}")]
    NumericNotPositive(String, f64),
    /// A numeric value parameter required non-negative input but the
    /// caller passed a negative number. Payload: directive name, value.
    #[error("@{0} must be non-negative, got {1}")]
    NumericNotNonNegative(String, f64),
    /// `@titlealign` received a keyword that is not `center`/`left`/`right`.
    #[error("invalid @titlealign value: \"{0}\"; expected center, left, or right")]
    InvalidTitleAlign(String),
    /// `@ruler` received a keyword that is not `on`/`off`. The payload
    /// carries the offending text verbatim (empty string for no argument).
    #[error("invalid @ruler value: \"{0}\"; expected on or off")]
    InvalidRulerValue(String),
    /// `@title` was used without an argument.
    #[error("@title requires an argument; write `@title \"\"` for an empty title row")]
    TitleRequiresArgument,
    /// `@` was followed by something other than `{` or an ASCII digit, so
    /// the anchor scanner could not begin an anchor token.
    #[error("expected `{{name}}` or digit after `@`")]
    AnchorExpectedNameOrDigit,
    /// `@{` had no matching `}` before the run of characters available to
    /// the anchor scanner ran out (typically end-of-line).
    #[error("`@{{` is not closed")]
    AnchorBraceNotClosed,
    /// An anchor index (`@N`) digit run failed to parse into `u32`. The only
    /// reachable cause today is a digit run whose numeric value exceeds
    /// `u32::MAX` (the scanner already filters to ASCII digits, so format
    /// errors cannot occur). The variant name keeps a generic shape so future
    /// scanner extensions can route additional `@N` parse failures through it.
    /// Payload: the offending digit run, verbatim.
    #[error("invalid anchor index: \"{0}\"")]
    AnchorIndexNotParseable(String),
    /// `@highlight_style key=value ...` had an empty attribute name (e.g.
    /// `@highlight_style =value`).
    #[error("@highlight_style: attribute name is empty")]
    HighlightStyleEmptyAttrName,
    /// `@highlight_style` attribute was missing `=` after the key name.
    /// Payload: the offending key.
    #[error("@highlight_style: missing `=` after attribute \"{0}\"")]
    HighlightStyleMissingEquals(String),
    /// `@highlight_style` quoted attribute value did not have a closing `"`.
    #[error("@highlight_style: unterminated quoted attribute value")]
    HighlightStyleUnterminatedValue,
}

impl ParseError {
    /// Construct a [`ParseError`] with the default `length == 0` (insertion
    /// point). Existing call sites use this when the offending range has not
    /// been tracked explicitly.
    pub(crate) fn new(location: SourceLocation, kind: ParseErrorKind) -> Self {
        Self {
            location,
            length: 0,
            kind,
        }
    }

    /// Construct a [`ParseError`] with an explicit character `length`.
    /// `length == 0` is the same insertion-point semantics as
    /// [`ParseError::new`]; `length >= 1` covers a real character range.
    pub(crate) fn with_length(location: SourceLocation, length: u32, kind: ParseErrorKind) -> Self {
        Self {
            location,
            length,
            kind,
        }
    }

    /// Borrow the error kind. Used by parser unit tests to verify the
    /// produced error variant; production code only formats the error via
    /// `Display`.
    #[cfg(test)]
    pub(crate) fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    /// 1-based source line number where the error was detected.
    pub fn line(&self) -> u32 {
        self.location.line
    }

    /// 1-based source column number where the error was detected. When the
    /// `length` of this error is `0`, this is the column of the insertion
    /// point (the character that would close the unterminated token).
    pub fn column(&self) -> u32 {
        self.location.column
    }

    /// Character-unit width of the offending range. `0` means the error is an
    /// insertion-point (e.g. `UnclosedQuote` reaching end of input). Front
    /// ends rendering carets must still draw one `^` for `length == 0`, per
    /// `docs/spec/cli.md` §パースエラー出力形式.
    pub fn length(&self) -> u32 {
        self.length
    }

    /// English-fixed short description of what went wrong, suitable for the
    /// `error:` header line of the rustc-style 4-component CLI format. The
    /// returned string never ends in a period.
    ///
    /// Allocates a new `String` per call; prefer [`Self::message_display`]
    /// when writing into a buffer or formatter.
    ///
    /// The text is the same string emitted by the inner kind's `Display`.
    /// Front ends are expected to combine this with `line()` / `column()` /
    /// `length()` and the original source snippet to build the full message.
    pub fn message(&self) -> String {
        self.kind.to_string()
    }

    /// Borrow the error kind's `Display` text without allocating.
    ///
    /// The returned value formats identically to [`Self::message`] but writes
    /// straight into the caller's `Formatter` / buffer (via `write!`), which
    /// lets renderers avoid the per-call `String` allocation that
    /// [`Self::message`] performs. Front ends rendering the `error:` header
    /// line should prefer this method.
    pub fn message_display(&self) -> impl std::fmt::Display + '_ {
        &self.kind
    }
}

#[cfg(test)]
mod tests;
