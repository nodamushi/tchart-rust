use super::{ColorError, NameError, ParseError, ParseErrorKind, SourceLocation};
use crate::parser::parse;

#[test]
fn source_location_display() {
    let location = SourceLocation::new(3, 7);
    assert_eq!(location.to_string(), "line 3, column 7");
}

#[test]
fn parse_error_display_wraps_inner() {
    let error = ParseError::new(
        SourceLocation::new(1, 2),
        ParseErrorKind::InvalidColor(ColorError::UnknownName),
    );
    let rendered = error.to_string();
    assert!(rendered.starts_with("line 1, column 2: "));
    assert!(rendered.contains("unknown CSS color name"));
}

#[test]
fn parse_error_kind_round_trips_through_display() {
    // Any kind that carries a payload should appear in the rendered Display
    // text. Use `TitleRequiresArgument` (no payload) as a stable example —
    // its message is fixed.
    let error = ParseError::new(
        SourceLocation::new(5, 10),
        ParseErrorKind::TitleRequiresArgument,
    );
    let rendered = error.to_string();
    assert!(rendered.starts_with("line 5, column 10: "));
    assert!(rendered.contains("@title requires an argument"));
}

#[test]
fn parse_error_name_kind() {
    let error = ParseError::new(
        SourceLocation::new(2, 4),
        ParseErrorKind::InvalidName(NameError::Empty),
    );
    assert!(error.to_string().contains("signal name is empty"));
}

// =============================================================================
// (line, col, length) requirement: docs/spec/tcml-format.md §位置情報の必須化
// =============================================================================

/// `ParseError::line()` / `col()` / `length()` / `message()` must be available
/// on the public API so the CLI / editor / wasm front ends can render the
/// rustc-style 4-component error format.
#[test]
fn parse_error_exposes_public_line_col_length_and_message() {
    let error = ParseError::with_length(
        SourceLocation::new(3, 7),
        3,
        ParseErrorKind::TitleRequiresArgument,
    );
    assert_eq!(error.line(), 3);
    assert_eq!(error.column(), 7);
    assert_eq!(error.length(), 3);
    assert!(!error.message().is_empty());
}

/// `ParseError::new(location, kind)` (without an explicit length) keeps the
/// existing one-character behaviour: length defaults to 0 (insertion point).
#[test]
fn parse_error_new_defaults_length_to_zero() {
    let error = ParseError::new(
        SourceLocation::new(1, 1),
        ParseErrorKind::TitleRequiresArgument,
    );
    assert_eq!(error.length(), 0);
}

/// `message()` for any variant must be an English sentence with no trailing
/// period — the spec requires English-fixed text matched by CLI tooling.
#[test]
fn parse_error_message_has_no_trailing_period_for_all_variants() {
    let cases: Vec<ParseErrorKind> = vec![
        ParseErrorKind::DontCareWithoutAnchor,
        ParseErrorKind::DuplicateAnchor,
        ParseErrorKind::UndefinedAnchor("a".to_owned()),
        ParseErrorKind::UnclosedQuote,
        ParseErrorKind::MissingInitialLevel,
        ParseErrorKind::UnknownParameter("unknown".to_owned()),
        ParseErrorKind::InvalidStepSlant(20.0, 5.0),
        ParseErrorKind::InvalidSkipAmount("abc".to_owned()),
    ];
    for kind in cases {
        let error = ParseError::new(SourceLocation::new(1, 1), kind.clone());
        let message = error.message();
        assert!(!message.is_empty(), "message empty for kind {kind:?}");
        assert!(
            !message.trim_end().ends_with('.'),
            "message must not end with period for kind {kind:?}: {message:?}"
        );
    }
}

/// `parse("Sig ?==\n")` must report `DontCareWithoutAnchor` with `length == 1`
/// pointing at the `?` character (col >= 1).
#[test]
fn dontcare_without_anchor_carries_length_one() {
    let result = parse("Sig ?==\n");
    let error = result.expect_err("expected DontCareWithoutAnchor");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
    assert_eq!(error.line(), 1);
    assert_eq!(error.length(), 1, "`?` is a single character");
}

/// An unclosed `"..."` literal inside a signal level row must report
/// `UnclosedQuote` with `length == 0` (insertion-point error at end of line
/// or end of file).
#[test]
fn unclosed_quote_carries_length_zero() {
    // `SigA _"hello\n` — the level char `_` satisfies `MissingInitialLevel`,
    // then `"hello` opens a quoted text literal that is never closed.
    let result = parse("SigA _\"hello\n");
    let error = result.expect_err("expected UnclosedQuote");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedQuote));
    assert_eq!(error.length(), 0, "insertion-point error has zero width");
}

/// `parse("@fontsize -1\n")` must report `InvalidLength` with line=1 and a
/// `length >= 1` covering the rejected argument.
#[test]
fn invalid_length_carries_argument_range() {
    let result = parse("@fontsize -1\n");
    let error = result.expect_err("expected InvalidLength");
    // The parser may pick any of several variants for "@fontsize must be
    // strictly positive" — the data-side requirement is that line and
    // length are populated regardless of which variant fires.
    assert_eq!(error.line(), 1);
    assert!(
        error.length() >= 1,
        "argument has at least one character, got length={}",
        error.length()
    );
}

/// Each `ParseErrorKind` variant must yield a non-empty English message
/// through `ParseError::message()`. Used by the CLI to print the `error:`
/// header line.
#[test]
fn parse_error_message_is_non_empty_for_dont_care_without_anchor() {
    let error = ParseError::new(
        SourceLocation::new(1, 1),
        ParseErrorKind::DontCareWithoutAnchor,
    );
    assert!(!error.message().is_empty());
}

// =============================================================================
// Precise column tracking (docs/tests/tcml-parser.feature.md
// §ParseError 位置情報の必須化 / `@step xyz` col=7, `Sig ?==` col=5 等)
// =============================================================================

/// `@step xyz` — the rejected argument `xyz` starts at character column 7
/// (`@`=1, `s`=2, `t`=3, `e`=4, `p`=5, ` `=6, `x`=7). `length` must be 3
/// (the full `xyz` token, character count).
#[test]
fn invalid_length_argument_column_for_at_step_xyz() {
    let result = parse("@step xyz\n");
    let error = result.expect_err("expected parse error for `@step xyz`");
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 7, "`xyz` starts at col 7");
    assert_eq!(error.length(), 3, "`xyz` is 3 characters");
}

/// `Sig ?==` — the `?` token sits at column 5
/// (`S`=1, `i`=2, `g`=3, ` `=4, `?`=5). `length` must be 1 (a single `?`).
#[test]
fn dontcare_without_anchor_column_for_sig_question_eq_eq() {
    let result = parse("Sig ?==\n");
    let error = result.expect_err("expected DontCareWithoutAnchor");
    assert!(matches!(
        error.kind(),
        ParseErrorKind::DontCareWithoutAnchor
    ));
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 5, "`?` is at col 5");
    assert_eq!(error.length(), 1);
}

/// `SigA _"hello\n` — the unclosed `"` opens at character column 7
/// (`S`=1, `i`=2, `g`=3, `A`=4, ` `=5, `_`=6, `"`=7). The error is an
/// insertion-point error (`length == 0`) but `col` still points at the
/// opening `"` so the caret renders under it.
#[test]
fn unclosed_quote_column_points_at_opening_quote() {
    let result = parse("SigA _\"hello\n");
    let error = result.expect_err("expected UnclosedQuote");
    assert!(matches!(error.kind(), ParseErrorKind::UnclosedQuote));
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 7, "opening `\"` sits at col 7");
    assert_eq!(error.length(), 0, "insertion-point error has zero width");
}

/// Multi-byte UTF-8 signal name: column is measured in Unicode characters,
/// not bytes. `日本語 abc\n` — the trailing `abc` is a bare-text run before
/// any level character, so the tokenizer emits `MissingInitialLevel`. The
/// reported `col` must be the character index of the first text character,
/// counting each Japanese ideograph as one character (not 3 bytes).
#[test]
fn multibyte_signal_name_uses_character_column() {
    // `日本語 abc` — name `日本語` (3 chars) + space + level string `abc`.
    // `日`=1, `本`=2, `語`=3, ` `=4, `a`=5, ... — first bad char `a` at col 5.
    let result = parse("\u{65E5}\u{672C}\u{8A9E} abc\n");
    let error = result.expect_err("expected MissingInitialLevel");
    assert_eq!(error.line(), 1);
    assert_eq!(
        error.column(),
        5,
        "char-column must be 5 (`日本語 ` = 4 chars, then `a`); got col={}",
        error.column()
    );
}
