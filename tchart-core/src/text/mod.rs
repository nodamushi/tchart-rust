//! Text-shaped value types.
//!
//! See `docs/spec/types.md` §2.2 / §2.3 / §2.4.
//!
//! All text inputs from TCML pass through one of these NewTypes so that raw
//! `String` values never reach layout or rendering with unchecked content.
//!
//! Internal newline normalization: incoming `\r\n` is rewritten to `\n` at
//! parse time so the stored representation only ever contains `\n` line
//! separators (per `docs/spec/types.md` §2.2). Lone `\r` is rejected as a
//! forbidden control character to keep `lines()` semantics consistent.

use crate::defaults::{DEFAULT_FONT_FAMILY, DEFAULT_FONTSIZE_PX};
use crate::units::Px;

/// A signal name. May contain `\n` for multi-line labels but no other control characters.
///
/// See `docs/spec/types.md` §2.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalName(String);

/// User-supplied text such as `<embedded-label>` or `% overlay text`.
///
/// See `docs/spec/types.md` §2.3. Allows newline and `\t`; rejects every other
/// Unicode control character (`Cc`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserText(String);

/// A single, validated, but **unescaped** line of user-supplied text.
///
/// Returned by [`SignalName::lines`] / [`UserText::lines`] /
/// [`FontFamily::as_unsafe_line`]. The contained slice carries no `\n`
/// (validated by the producer) but its other characters are still
/// user-supplied — callers must hand the value to the SVG escape API
/// (`UserValue::write_escaped` in `crate::svg::buf`) and must not concatenate
/// it raw into output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnsafeLineText<'text>(&'text str);

/// A CSS / SVG font-family identifier list.
///
/// See `docs/spec/types.md` §2.4.
///
/// Stored as a verified string. Disallowed: empty, control characters,
/// double quotes (which would break SVG attribute serialization).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFamily(String);

/// A font specification (family + size).
///
/// See `docs/spec/types.md` §2.4.
#[derive(Debug, Clone, PartialEq)]
pub struct FontSpec {
    family: FontFamily,
    size: Px,
}

impl FontSpec {
    /// Return a new `FontSpec` with the given family and size.
    pub fn new(family: FontFamily, size: Px) -> Self {
        Self { family, size }
    }

    /// Return the font family.
    pub fn family(&self) -> &FontFamily {
        &self.family
    }

    /// Return the font size.
    pub fn size(&self) -> Px {
        self.size
    }

    /// Set the font family.
    pub fn set_family(&mut self, family: FontFamily) {
        self.family = family;
    }

    /// Set the font size.
    pub fn set_size(&mut self, size: Px) {
        self.size = size;
    }

    /// Produce a CSS `font` shorthand string (`"<size>px <family>"`) suitable
    /// for `Canvas.measureText`. The family list is verified at parse time
    /// (no control chars, no `"`), so the string is safe to feed to a CSS
    /// font setter.
    pub fn to_canvas_css(&self) -> String {
        format!("{}px {}", self.size.to_f32(), self.family.0)
    }
}

impl Default for FontSpec {
    fn default() -> Self {
        let family =
            FontFamily::parse(DEFAULT_FONT_FAMILY).expect("default font family must parse");
        let size = DEFAULT_FONTSIZE_PX;
        Self { family, size }
    }
}

/// Errors produced when parsing a [`SignalName`].
///
/// Variants that name a specific offending character carry the character's
/// 0-based offset *within the parsed input string* so the caller can compute
/// an exact source column. The offset is in Unicode scalar values, not bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum NameError {
    /// The supplied name was empty.
    #[error("signal name is empty")]
    Empty,
    /// The name contained a forbidden control character (anything other than `\n` / `\r\n`).
    #[error("signal name contains a forbidden control character")]
    ForbiddenControlChar { char_offset: u32 },
}

impl NameError {
    /// 0-based char offset within the parsed value where the caret should
    /// land, or `None` when the error is about the value as a whole.
    pub(crate) fn char_offset(&self) -> Option<u32> {
        match self {
            Self::Empty => None,
            Self::ForbiddenControlChar { char_offset } => Some(*char_offset),
        }
    }
}

/// Errors produced when parsing a [`UserText`] or [`FontFamily`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TextError {
    /// The supplied text was empty (only relevant for [`FontFamily`]).
    #[error("text is empty")]
    Empty,
    /// The text contained a forbidden control character.
    #[error("text contains a forbidden control character")]
    ForbiddenControlChar { char_offset: u32 },
    /// The text contained a forbidden character such as `"`.
    #[error("text contains a forbidden character")]
    ForbiddenCharacter { char_offset: u32 },
}

impl TextError {
    /// 0-based char offset within the parsed value where the caret should
    /// land, or `None` when the error is about the value as a whole.
    pub(crate) fn char_offset(&self) -> Option<u32> {
        match self {
            Self::Empty => None,
            Self::ForbiddenControlChar { char_offset }
            | Self::ForbiddenCharacter { char_offset } => Some(*char_offset),
        }
    }
}

impl SignalName {
    /// Parse a signal name. Allows `\n` (and normalizes `\r\n` → `\n`); rejects
    /// empty input and any other control characters including lone `\r`.
    /// On rejection, the error carries the 0-based char offset of the
    /// offending character within `input` so the caller can compute a
    /// precise source column.
    pub(crate) fn parse(input: &str) -> Result<Self, NameError> {
        if input.is_empty() {
            return Err(NameError::Empty);
        }
        validate_text_chars(input, /* allow_tab */ false)
            .map_err(|char_offset| NameError::ForbiddenControlChar { char_offset })?;
        let normalized = normalize_line_endings(input)
            .expect("validate_text_chars already accepted all CR / LF sequences");
        if normalized.is_empty() {
            return Err(NameError::Empty);
        }
        Ok(SignalName(normalized))
    }

    /// Iterate over each `\n`-separated line of the name as an
    /// [`UnsafeLineText`] (still unescaped, single-line, must go through the
    /// SVG escape API before reaching output).
    pub(crate) fn lines(&self) -> impl Iterator<Item = UnsafeLineText<'_>> + '_ {
        self.0.lines().map(UnsafeLineText)
    }

    /// Number of `\n`-separated lines.
    pub(crate) fn count_line(&self) -> usize {
        self.0.lines().count()
    }

    /// Flatten a multi-line name into a single string, joining lines with a
    /// single space. Used by the WaveDrom converter, which has no concept of
    /// multi-line signal labels and must collapse them to one line.
    pub(crate) fn flatten_to_string(&self) -> String {
        self.lines()
            .map(UnsafeLineText::unsafe_text)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Test-only escape hatch returning the raw stored string. **Production
    /// code must go through [`SignalName::lines`] + the SVG escape API**;
    /// this method is gated on `cfg(test)` so that no production caller can
    /// extract the unescaped string.
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl UserText {
    /// Parse user text. Allows `\n` and `\t` (and normalizes `\r\n` → `\n`);
    /// rejects every other control character including lone `\r`.
    /// On rejection, the error carries the 0-based char offset of the
    /// offending character within `input`.
    pub(crate) fn parse(input: &str) -> Result<Self, TextError> {
        validate_text_chars(input, /* allow_tab */ true)
            .map_err(|char_offset| TextError::ForbiddenControlChar { char_offset })?;
        let normalized = normalize_line_endings(input)
            .expect("validate_text_chars already accepted all CR / LF sequences");
        Ok(UserText(normalized))
    }

    /// Number of `\n`-separated lines (matches `str::lines()` semantics:
    /// the trailing-newline-only case yields one line).
    pub(crate) fn count_line(&self) -> usize {
        self.0.lines().count()
    }

    /// Returns `true` if the user text contains no characters at all.
    ///
    /// A single space (`" "`) is **not** empty; this method discriminates
    /// the truly empty `""` case so callers can skip emitting visually-empty
    /// elements (e.g. an empty `<text>` for a bus label).
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over each `\n`-separated line as an [`UnsafeLineText`].
    pub(crate) fn lines(&self) -> impl Iterator<Item = UnsafeLineText<'_>> + '_ {
        self.0.lines().map(UnsafeLineText)
    }

    /// Test-only escape hatch; see [`SignalName::as_str`].
    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'text> UnsafeLineText<'text> {
    /// Borrow the contained (still unescaped) line. Visible to the `svg`
    /// module so the `UserValue` impl in `crate::svg::buf` can hand the
    /// characters to the XML escape pass; **no other production module should
    /// call this method**, callers obtain an `UnsafeLineText` and pass it
    /// straight to the SVG escape API.
    pub(crate) fn unsafe_text(self) -> &'text str {
        self.0
    }
}

impl FontFamily {
    /// Parse a font family identifier list. Accepts CSS-style comma-separated
    /// fallback lists with optional `"..."` quotes around entries that contain
    /// whitespace (per `docs/spec/tcml-format.md` §「ローカルパラメータ」`font`).
    ///
    /// Each entry is preserved verbatim with respect to its own quoting: if
    /// the entry was wrapped in `"..."` in the input, the surrounding quotes
    /// are kept in the stored form so the value can be emitted directly as an
    /// SVG / CSS `font-family` attribute; entries the caller did not quote
    /// (including bare family names such as `Comic Neue` and generic families
    /// such as `sans-serif` / `monospace`) are kept bare. Commas inside a
    /// quoted entry are treated as part of the entry, not as a separator.
    /// Inter-entry whitespace is normalized to `, `.
    ///
    /// Rejects empty input, control characters (including all newlines —
    /// font-family is single-line by SVG attribute rules), and any stray `"`
    /// inside an entry's payload.
    pub(crate) fn parse(input: &str) -> Result<Self, TextError> {
        let entries = input
            .chars()
            .enumerate()
            .try_fold(FamilyListBuilder::default(), FamilyListBuilder::feed)?
            .finish()?;
        if entries.is_empty() {
            return Err(TextError::Empty);
        }
        Ok(FontFamily(entries.join(", ")))
    }

    /// Borrow the family list as a single [`UnsafeLineText`] for the SVG
    /// escape API. Single line by construction (parse rejects all control
    /// chars).
    pub(crate) fn as_unsafe_line(&self) -> UnsafeLineText<'_> {
        UnsafeLineText(&self.0)
    }

    /// Test-only escape hatch; see [`SignalName::as_str`].
    #[cfg(test)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Incremental parser for a CSS-style comma-separated font-family list.
///
/// Designed for `Iterator::try_fold` consumption: each `feed` returns the
/// updated builder or an error, and `finish` produces the final entry list.
/// State is kept private so callers cannot mutate intermediate fields and
/// break the quote / comma invariants.
///
/// Per-entry quote structure: an entry is either fully unquoted (no `"` at
/// all), or fully wrapped in a single `"..."` pair around the trimmed
/// payload. Any other arrangement (`"foo"bar"`, `foo"bar"`, `foo"bar`,
/// `"foo"` followed by stray content, etc.) is a structural error caught at
/// entry commit time.
#[derive(Default)]
struct FamilyListBuilder {
    entries: Vec<String>,
    current: String,
    in_quotes: bool,
    /// 0-based char offset of the current entry's start within the full
    /// input; used so a per-entry structural error (e.g. unclosed `"`,
    /// stray `"`) carries the offset of the offending position rather than
    /// of the whole list.
    current_start: u32,
    /// Offset of the opening `"` for the currently-open entry. `0` is fine
    /// as a default sentinel because `in_quotes` distinguishes "no open
    /// quote" from "open quote at offset 0".
    quote_open_offset: u32,
}

impl FamilyListBuilder {
    /// Consume one input character, returning the updated builder.
    /// Rejects control characters in the same pass; `"` and `,` semantics
    /// are recognised and the structural quote check is deferred to
    /// `commit_entry` so the entire entry is inspected as a whole.
    fn feed(mut self, (index, character): (usize, char)) -> Result<Self, TextError> {
        let offset = u32::try_from(index).unwrap_or(u32::MAX);
        if character.is_control() {
            return Err(TextError::ForbiddenControlChar {
                char_offset: offset,
            });
        }
        if character == ',' && !self.in_quotes {
            self.commit_entry(offset)?;
            self.current_start = offset + 1;
            return Ok(self);
        }
        if character == '"' {
            if !self.in_quotes {
                self.quote_open_offset = offset;
            }
            self.in_quotes = !self.in_quotes;
        }
        self.current.push(character);
        Ok(self)
    }

    /// Finalize the builder: commit any trailing entry and return the list.
    fn finish(mut self) -> Result<Vec<String>, TextError> {
        if self.in_quotes {
            // Unclosed quote: point at the opening `"`.
            return Err(TextError::ForbiddenCharacter {
                char_offset: self.quote_open_offset,
            });
        }
        // The final commit has no following char, so report any structural
        // error against the entry's starting offset.
        let final_offset = self.current_start;
        self.commit_entry(final_offset)?;
        Ok(self.entries)
    }

    /// Validate the buffered entry's quote structure, push it onto `entries`,
    /// and reset the per-entry state.
    ///
    /// Valid forms (after trimming the buffer's outer whitespace):
    /// - bare payload with no `"` characters
    /// - exactly two `"` characters, one at the start and one at the end of
    ///   the trimmed buffer (i.e. a clean `"..."` wrap).
    ///
    /// An empty buffer (e.g. between two commas or before any input) is
    /// silently skipped; `parse` reports the all-empty case via the
    /// entries-vector emptiness check. An empty quoted form `""` is rejected
    /// as `Empty`.
    fn commit_entry(&mut self, end_offset: u32) -> Result<(), TextError> {
        let trimmed = self.current.trim();
        let quote_count = trimmed.bytes().filter(|byte| *byte == b'"').count();
        if quote_count == 0 {
            if !trimmed.is_empty() {
                self.entries.push(trimmed.to_owned());
            }
        } else if quote_count == 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
            // `quote_count == 2` plus start/end being `"` guarantees the buffer
            // is a clean wrap; the only structurally-valid but semantically
            // empty form left is `""` itself.
            if trimmed.len() == 2 {
                return Err(TextError::Empty);
            }
            self.entries.push(trimmed.to_owned());
        } else {
            // Structurally-malformed entry: point at the entry's last
            // observed offset (the `,` or end-of-input terminator) — the
            // best single column we can name without re-scanning the
            // buffer for the specific bad quote.
            return Err(TextError::ForbiddenCharacter {
                char_offset: end_offset,
            });
        }
        self.current.clear();
        Ok(())
    }
}

/// Normalize Windows-style `\r\n` line endings to `\n`. Returns `None` when
/// Walk `input` once, returning the 0-based char offset of the first
/// forbidden control character. Allows `\n` and `\r\n` (the latter as a
/// paired sequence advancing two chars at once); rejects lone `\r` and any
/// other control character. When `allow_tab` is true, `\t` is also allowed
/// (`UserText` permits tabs but `SignalName` does not).
fn validate_text_chars(input: &str, allow_tab: bool) -> Result<(), u32> {
    let mut chars = input.chars().enumerate().peekable();
    while let Some((index, character)) = chars.next() {
        let offset = u32::try_from(index).unwrap_or(u32::MAX);
        if character == '\r' {
            // Accept `\r\n`; reject lone `\r`.
            if chars.next_if(|(_, next)| *next == '\n').is_some() {
                continue;
            }
            return Err(offset);
        }
        if character == '\n' || (allow_tab && character == '\t') {
            continue;
        }
        if character.is_control() {
            return Err(offset);
        }
    }
    Ok(())
}

/// `input` contains a lone `\r` (CR not immediately followed by LF), which is
/// rejected as a forbidden control character by the caller.
///
/// The bare-string fast path avoids allocation when no `\r` is present.
fn normalize_line_endings(input: &str) -> Option<String> {
    if !input.contains('\r') {
        return Some(input.to_owned());
    }
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\r' {
            chars.next_if(|next| *next == '\n')?;
            output.push('\n');
        } else {
            output.push(character);
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests;
