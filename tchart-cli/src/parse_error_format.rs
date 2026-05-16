//! Render a [`ParseFailure`] as the rustc-style 4-component error message
//! defined in `docs/spec/cli.md` §パースエラー出力形式.
//!
//! Output layout (no blank "alignment" line between header and snippet, per
//! the spec's "最小フォーマット" rule):
//!
//! ```text
//! error: cannot parse "xyz" as a number for @step
//!  --> sample.tc:3:7
//! 3 | @step xyz
//!   |       ^^^
//! ```
//!
//! The renderer never modifies the original [`ParseError`] message; it only
//! composes the four lines around it.

use std::fmt;
use std::path::Path;

use crate::error::ParseFailure;

/// Pseudo-path used in the location line when the input was read from
/// standard input (the CLI does not currently support stdin, but the spec
/// defines this value for forward compatibility).
const STDIN_DISPLAY: &str = "<stdin>";

/// Tab expansion width applied to the snippet line. Spec value, fixed at 4.
const TAB_WIDTH: usize = 4;

/// Render a [`ParseFailure`] as the rustc-style 4-component message
/// (`error:` header + ` --> ` location + snippet + caret).
///
/// The output string includes a trailing newline so callers can write it to
/// stderr without further punctuation.
pub(crate) fn format_parse_failure(failure: &ParseFailure) -> String {
    Rendered::from(failure).to_string()
}

/// All the pieces the four rustc-style lines need. Built from the
/// [`ParseFailure`] once, then rendered via [`fmt::Display`] without further
/// allocation per line.
///
/// Per the spec, the column reported in the location line (` --> FILE:LINE:COL`)
/// and the caret position in the snippet are both measured in tab-expanded
/// display columns. Storing one `display_column` field guarantees the two
/// lines cannot disagree.
struct Rendered<'failure> {
    /// Backing failure; borrowed only for the kind's `Display`.
    failure: &'failure ParseFailure,
    /// File label (path or `<stdin>`) for the ` --> FILE` line.
    file_label: FileLabel<'failure>,
    /// 1-based source line number.
    line_number: u32,
    /// 1-based tab-expanded display column shared by the location and caret lines.
    display_column: u32,
    /// Snippet line with tabs expanded to spaces.
    snippet_text: String,
    /// Width (chars) of the `line_number` decimal representation; used to
    /// align the caret-row gutter with the snippet-row gutter.
    gutter_width: usize,
    /// Caret count. `length >= 1` produces `length` carets; `length == 0`
    /// (insertion-point error) still produces exactly one caret per spec.
    caret_count: usize,
}

/// File label for the location line.
enum FileLabel<'failure> {
    File(&'failure Path),
    Stdin,
}

impl fmt::Display for FileLabel<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileLabel::File(path) => write!(formatter, "{}", path.display()),
            FileLabel::Stdin => formatter.write_str(STDIN_DISPLAY),
        }
    }
}

impl<'failure> From<&'failure ParseFailure> for Rendered<'failure> {
    fn from(failure: &'failure ParseFailure) -> Self {
        let error = failure.error();
        let line_number = error.line();
        let snippet_raw = pick_snippet_line(failure.source(), line_number);
        let (snippet_text, display_column) = expand_tabs(snippet_raw, error.column());
        let gutter_width = digit_count(line_number);
        let caret_count = if error.length() == 0 {
            1
        } else {
            error.length() as usize
        };
        let file_label = match failure.path() {
            Some(path) => FileLabel::File(path),
            None => FileLabel::Stdin,
        };
        Self {
            failure,
            file_label,
            line_number,
            display_column,
            snippet_text,
            gutter_width,
            caret_count,
        }
    }
}

impl fmt::Display for Rendered<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. `error: <message>` header.
        writeln!(
            formatter,
            "error: {}",
            self.failure.error().message_display()
        )?;
        // 2. ` --> FILE:LINE:COL`.
        writeln!(
            formatter,
            " --> {}:{}:{}",
            self.file_label, self.line_number, self.display_column
        )?;
        // 3. `<LINE> | <snippet>`.
        writeln!(formatter, "{} | {}", self.line_number, self.snippet_text)?;
        // 4. `<gutter spaces> | <padding>^^^...`.
        for _ in 0..self.gutter_width {
            formatter.write_str(" ")?;
        }
        formatter.write_str(" | ")?;
        let padding = self.display_column.saturating_sub(1) as usize;
        for _ in 0..padding {
            formatter.write_str(" ")?;
        }
        for _ in 0..self.caret_count {
            formatter.write_str("^")?;
        }
        formatter.write_str("\n")
    }
}

/// Decimal-digit count of `value`. A separate helper because computing it via
/// `value.to_string().len()` would allocate on every render.
fn digit_count(value: u32) -> usize {
    if value == 0 {
        return 1;
    }
    let mut count: usize = 0;
    let mut remaining = value;
    while remaining > 0 {
        count += 1;
        remaining /= 10;
    }
    count
}

/// Borrow the snippet line for `line_number` (1-based) from `source`.
/// Trailing `\r` is dropped so CRLF inputs render cleanly. Returns an empty
/// string when the line is out of range (defensive — should not happen when
/// the parser produced a valid `(line, column)`).
fn pick_snippet_line(source: &str, line_number: u32) -> &str {
    if line_number == 0 {
        return "";
    }
    let index = (line_number - 1) as usize;
    source
        .split('\n')
        .nth(index)
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .unwrap_or("")
}

/// Expand tabs in `line` to [`TAB_WIDTH`] spaces and re-map the 1-based
/// `core_column` (which counts each tab as 1) to the column in the
/// tab-expanded string.
///
/// Returns the expanded line and the re-mapped column.
fn expand_tabs(line: &str, core_column: u32) -> (String, u32) {
    if !line.contains('\t') {
        return (line.to_owned(), core_column);
    }
    let mut expanded = String::with_capacity(line.len());
    let mut display_column: u32 = 1;
    let mut mapped_column: u32 = core_column;
    let target_character = core_column.saturating_sub(1) as usize;
    let mut found_target = false;
    for (character_index, character) in line.chars().enumerate() {
        if character_index == target_character {
            mapped_column = display_column;
            found_target = true;
        }
        if character == '\t' {
            for _ in 0..TAB_WIDTH {
                expanded.push(' ');
            }
            display_column += TAB_WIDTH as u32;
        } else {
            expanded.push(character);
            display_column += 1;
        }
    }
    if !found_target {
        // Caret falls past the last character (typical for insertion-point
        // errors at end of line); place it at the display column following
        // the last rendered character.
        mapped_column = display_column;
    }
    (expanded, mapped_column)
}

#[cfg(test)]
mod tests;
