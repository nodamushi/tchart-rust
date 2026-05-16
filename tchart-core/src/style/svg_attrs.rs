//! Free-form SVG attribute lists used by highlight / don't-care styles.

use std::iter::Peekable;
use std::str::Chars;

use crate::errors::ParseErrorKind;
use crate::text::UserText;

/// An ordered list of `(key, value)` SVG attribute pairs.
///
/// See `docs/spec/types.md` §4. Keys are stored as raw `String` because
/// tchart-coffee allows arbitrary attribute names; validation happens in the
/// parser layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SvgAttrList(Vec<(String, UserText)>);

/// Allowed SVG presentation attribute keys for highlight / dontcare style lists.
///
/// Only these attributes are permitted through the security filter.
/// See `docs/spec/svg-rendering.md` "プリゼンテーション属性ホワイトリスト".
const SAFE_ATTRS: &[&str] = &[
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-opacity",
    "stroke-width",
    "stroke-dasharray",
    "opacity",
];

impl From<Vec<(String, UserText)>> for SvgAttrList {
    fn from(pairs: Vec<(String, UserText)>) -> Self {
        Self(pairs)
    }
}

impl SvgAttrList {
    /// Parse an attribute list of the form `key1="value1" key2=value2 ...`.
    /// Whitespace separates pairs; double-quoted values may contain spaces.
    ///
    /// Returns [`ParseErrorKind`] (without a [`SourceLocation`]) so the caller
    /// can wrap with the directive's own location.
    pub(crate) fn parse(value: &str) -> Result<Self, ParseErrorKind> {
        let mut scanner = AttrScanner::new(value);
        let mut entries: Vec<(String, UserText)> = Vec::new();
        loop {
            scanner.skip_whitespace();
            if scanner.is_empty() {
                return Ok(Self(entries));
            }
            entries.push(scanner.consume_attr_pair()?);
        }
    }

    /// Build a list from static key-value string pairs.
    pub(crate) fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let entries = pairs
            .iter()
            .map(|(key, value)| {
                let text = UserText::parse(value).expect("default attr value must parse");
                ((*key).to_owned(), text)
            })
            .collect();
        Self(entries)
    }

    /// Returns the attribute pairs as a slice. Used by parser unit tests to
    /// verify that the parsed attribute list contains the expected pairs.
    #[cfg(test)]
    pub(crate) fn as_slice(&self) -> &[(String, UserText)] {
        &self.0
    }

    /// Iterator over only the SVG-safe `(key, value)` attribute pairs.
    ///
    /// Filters to allowed presentation attribute keys.
    /// See `docs/spec/svg-rendering.md` "プリゼンテーション属性ホワイトリスト".
    pub(crate) fn safe_pairs(&self) -> impl Iterator<Item = &(String, UserText)> {
        self.0
            .iter()
            .filter(|(key, _)| Self::is_safe_attr_name(key))
    }

    /// Returns `true` when `name` is an allowed SVG presentation attribute.
    fn is_safe_attr_name(name: &str) -> bool {
        SAFE_ATTRS
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(name))
    }
}

/// Cursor over an attribute-list string. Owns the `Peekable<Chars>` so that
/// every helper is a method on the cursor instead of a free function taking
/// `&mut Peekable<Chars>`.
struct AttrScanner<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> AttrScanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
        }
    }

    fn is_empty(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn skip_whitespace(&mut self) {
        while let Some(&character) = self.chars.peek()
            && character.is_whitespace()
        {
            self.chars.next();
        }
    }

    fn consume_attr_pair(&mut self) -> Result<(String, UserText), ParseErrorKind> {
        let key = self.read_until(|character| character == '=' || character.is_whitespace());
        if key.is_empty() {
            return Err(ParseErrorKind::HighlightStyleEmptyAttrName);
        }
        self.skip_whitespace();
        if self.chars.next() != Some('=') {
            return Err(ParseErrorKind::HighlightStyleMissingEquals(key));
        }
        self.skip_whitespace();
        let raw = self.consume_attr_value()?;
        let text = UserText::parse(&raw).map_err(ParseErrorKind::InvalidText)?;
        Ok((key, text))
    }

    fn consume_attr_value(&mut self) -> Result<String, ParseErrorKind> {
        if self.chars.peek() != Some(&'"') {
            return Ok(self.read_until(char::is_whitespace));
        }
        self.chars.next();
        let value = self.read_until(|character| character == '"');
        if self.chars.next() != Some('"') {
            return Err(ParseErrorKind::HighlightStyleUnterminatedValue);
        }
        Ok(value)
    }

    fn read_until(&mut self, mut stop: impl FnMut(char) -> bool) -> String {
        let mut output = String::new();
        while let Some(&character) = self.chars.peek() {
            if stop(character) {
                break;
            }
            output.push(character);
            self.chars.next();
        }
        output
    }
}
