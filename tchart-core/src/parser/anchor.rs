//! `@{name}` / `@N` anchor scanner.

use std::iter::Peekable;
use std::num::NonZeroU32;
use std::str::CharIndices;

use crate::anchor::{AnchorId, AnchorName};
use crate::errors::{ParseError, ParseErrorKind, SourceLocation};

/// Adapter wrapping the source-string char iterator with an explicit
/// location. Methods consume one anchor token at a time.
pub(super) struct AnchorScanner<'source, 'iter> {
    chars: &'iter mut Peekable<CharIndices<'source>>,
    source: &'source str,
    location: SourceLocation,
}

impl<'source, 'iter> AnchorScanner<'source, 'iter> {
    /// Borrow the existing iterator. The caller has already positioned it
    /// just after the leading `@`.
    pub(super) fn new(
        chars: &'iter mut Peekable<CharIndices<'source>>,
        source: &'source str,
        location: SourceLocation,
    ) -> Self {
        Self {
            chars,
            source,
            location,
        }
    }

    /// Read one [`AnchorId`] token. The next character must be `{` (named
    /// anchor) or an ASCII digit (indexed anchor).
    pub(super) fn consume_id(&mut self) -> Result<AnchorId, ParseError> {
        match self.chars.peek().copied() {
            Some((_, '{')) => self.consume_named(),
            Some((index, character)) if character.is_ascii_digit() => self.consume_indexed(index),
            _ => Err(self.make_error_with_kind(ParseErrorKind::AnchorExpectedNameOrDigit)),
        }
    }

    /// Discards the opening `{` and returns the named anchor id.
    fn consume_named(&mut self) -> Result<AnchorId, ParseError> {
        // Consume the opening `{`.
        self.chars.next();
        let start = self
            .chars
            .peek()
            .map(|(index, _)| *index)
            .ok_or_else(|| self.make_error_with_kind(ParseErrorKind::AnchorBraceNotClosed))?;
        let end = self
            .chars
            .by_ref()
            .find(|(_, character)| *character == '}')
            .map(|(index, _)| index)
            .ok_or_else(|| self.make_error_with_kind(ParseErrorKind::AnchorBraceNotClosed))?;
        let raw = &self.source[start..end];
        let name = AnchorName::parse(raw).map_err(|error| {
            // Translate the inner offset into the source column. The name
            // text begins at `@` + 2 (`@` and `{`), so:
            //   column = anchor `@` column + 2 + inner_offset
            // Length is 1 for an offset-bearing error, the whole name's
            // width otherwise (Empty case).
            let (col_off, len) = match error.char_offset() {
                Some(offset) => (2u32.saturating_add(offset), 1),
                None => (2, u32::try_from(raw.chars().count()).unwrap_or(u32::MAX)),
            };
            ParseError::with_length(
                SourceLocation::new(self.location.line(), self.location.column() + col_off),
                len,
                ParseErrorKind::InvalidAnchorName(error),
            )
        })?;
        Ok(AnchorId::Named(name))
    }

    /// Reads a digit run as an indexed anchor.
    ///
    /// `digit_start` is the byte index of the first digit, captured by
    /// [`Self::consume_id`] from the peeked entry — passing it through avoids
    /// an `expect("caller guarantees a digit")` at the method boundary.
    fn consume_indexed(&mut self, digit_start: usize) -> Result<AnchorId, ParseError> {
        let mut end = digit_start;
        while let Some(&(index, character)) = self.chars.peek() {
            if !character.is_ascii_digit() {
                break;
            }
            self.chars.next();
            end = index + character.len_utf8();
        }
        let raw = &self.source[digit_start..end];
        let value: u32 = raw.parse().map_err(|_| {
            self.make_error_with_kind(ParseErrorKind::AnchorIndexNotParseable(raw.to_owned()))
        })?;
        let value = NonZeroU32::new(value)
            .ok_or_else(|| self.make_error_with_kind(ParseErrorKind::AnchorIndexZero))?;
        Ok(AnchorId::Indexed(value))
    }

    fn make_error_with_kind(&self, kind: ParseErrorKind) -> ParseError {
        // Anchor scanner errors all fire on the leading `@`; the simplest
        // visible-caret length is 1 (the `@` itself). Callers wanting wider
        // ranges (e.g. an unclosed `@{abc`) would need additional state and
        // are not worth the complexity until requested.
        ParseError::with_length(self.location, 1, kind)
    }
}
