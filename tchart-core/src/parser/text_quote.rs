//! Quote-aware text scanning helpers shared across the parser.
//!
//! Two related concerns live here:
//!
//! - `"..."`-quoted token collection used by `@title` and `"..."`-prefixed
//!   signal-name lines: locate an unescaped closing `"`, accumulate
//!   continuation lines while the quote is open, and unescape the payload.
//! - Inline `//` comment stripping ([`strip_inline_comment`]). The cut must
//!   track quote state so a `//` inside an open `"..."` region is preserved
//!   as literal text rather than treated as a comment marker. Because that
//!   quote-state machine is the same one used by the quoted-token path, the
//!   helper lives in this module rather than in a separate comment-stripping
//!   module that would have to reimplement quote tracking.

use crate::errors::{ParseError, ParseErrorKind, SourceLocation};

/// Return `line` truncated at the first unquoted `//` (the TCML inline
/// comment marker, per `docs/spec/tcml-format.md` §「行の種類」). The cut
/// drops the `//` characters and everything after them on that line.
///
/// Quote handling: while a `"..."` region is open (the closing `"` has not
/// yet been seen on this line), `//` inside the quote is preserved as
/// literal text. A backslash escapes the next character (`\"` keeps the
/// quote region open).
///
/// When the line opens a quote that does not close on the same line, the
/// quote spans into the next source line; this function leaves the line
/// untouched in that case so a later pass (`QuotedToken::collect`) can keep
/// reading. The caller is responsible for re-trimming the tail emitted by
/// the multi-line quote pass.
pub(super) fn strip_inline_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut characters = line.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        match character {
            '\\' if in_quote => {
                // Consume the next character regardless of byte width.
                characters.next();
            }
            '"' => in_quote = !in_quote,
            '/' if !in_quote => {
                if let Some(&(_, next)) = characters.peek()
                    && next == '/'
                {
                    return &line[..index];
                }
            }
            _ => {}
        }
    }
    line
}

/// Result of [`QuotedToken::collect`]. A named struct so call sites can
/// refer to each component by name.
pub(super) struct QuotedToken<'a> {
    /// Unescaped payload between the outer `"` characters.
    pub(super) text: String,
    /// Portion of the final source line after the closing `"`.
    pub(super) tail: &'a str,
    /// Number of source lines consumed (always `>= 1`).
    pub(super) consumed_lines: usize,
}

impl<'a> QuotedToken<'a> {
    /// Walk `lines` starting at `start`, treating `first_line_after_open`
    /// (the substring of `lines[start]` that begins immediately after the
    /// opening `"`) as the initial quoted-payload slice. Returns when the
    /// closing unescaped `"` is found, or yields
    /// [`ParseErrorKind::UnclosedQuote`] when the source runs out before the
    /// closing `"`.
    ///
    /// Standard escape interpretation: `\"`, `\\`, `\n`. Unknown escapes are
    /// preserved verbatim.
    pub(super) fn collect(
        lines: &'a [&'a str],
        start: usize,
        first_line_after_open: &'a str,
        location: SourceLocation,
    ) -> Result<Self, ParseError> {
        let mut content = String::new();
        let mut current = first_line_after_open;
        let mut index = start;
        loop {
            if let Some(end) = find_unescaped_quote(current) {
                content.push_str(&current[..end]);
                return Ok(Self {
                    text: unescape_quoted(&content),
                    tail: &current[end + 1..],
                    consumed_lines: index - start + 1,
                });
            }
            content.push_str(current);
            content.push('\n');
            index += 1;
            let Some(&next_line) = lines.get(index) else {
                return Err(ParseError::new(location, ParseErrorKind::UnclosedQuote));
            };
            current = next_line;
        }
    }
}

/// Position of the first unescaped `"` in `input`, or `None` if there is
/// none. `\"` and `\\` consume the next character together.
///
/// Iterates via `char_indices` so non-ASCII bytes never get sliced (review
/// item H-02). The escape rules are byte-safe (`\\` is ASCII so the next
/// `char` is whatever it is, including multi-byte content).
fn find_unescaped_quote(input: &str) -> Option<usize> {
    let mut indices = input.char_indices();
    while let Some((index, character)) = indices.next() {
        if character == '\\' {
            // Skip the escaped character (any byte width).
            indices.next();
            continue;
        }
        if character == '"' {
            return Some(index);
        }
    }
    None
}

fn unescape_quoted(input: &str) -> String {
    unescape(input, quoted_escape_target)
}

/// Generic unescape pass shared by [`unescape_quoted`] and
/// [`super::label::unescape_label`].
///
/// `target` returns the replacement character for an escape sequence
/// `\<character>`, or `None` to preserve the backslash verbatim. `\` at end
/// of input is also preserved verbatim.
pub(super) fn unescape(input: &str, target: fn(char) -> Option<char>) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some(after) => match target(after) {
                Some(replacement) => output.push(replacement),
                None => {
                    output.push('\\');
                    output.push(after);
                }
            },
            None => output.push('\\'),
        }
    }
    output
}

fn quoted_escape_target(after: char) -> Option<char> {
    match after {
        '"' => Some('"'),
        'n' => Some('\n'),
        '\\' => Some('\\'),
        _ => None,
    }
}
