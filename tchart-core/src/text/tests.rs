use super::{FontFamily, FontSpec, NameError, SignalName, TextError, UnsafeLineText, UserText};
use crate::units::Px;

fn collect_lines<'text, I>(lines: I) -> Vec<&'text str>
where
    I: Iterator<Item = UnsafeLineText<'text>>,
{
    lines.map(UnsafeLineText::unsafe_text).collect()
}

#[test]
fn signal_name_accepts_plain_ascii() {
    let name = SignalName::parse("clk").expect("valid name");
    assert_eq!(name.as_str(), "clk");
}

#[test]
fn signal_name_accepts_newline_for_multi_line() {
    let name = SignalName::parse("a\nb").expect("multi-line name");
    assert_eq!(collect_lines(name.lines()), vec!["a", "b"]);
}

#[test]
fn signal_name_accepts_crlf() {
    // Windows line endings normalize to `\n` so `lines()` yields the same
    // partition as the LF case.
    let name = SignalName::parse("a\r\nb").expect("CRLF normalized");
    assert_eq!(name.as_str(), "a\nb");
    assert_eq!(collect_lines(name.lines()), vec!["a", "b"]);
}

#[test]
fn signal_name_rejects_lone_carriage_return() {
    // CR not followed by LF is a forbidden control character (cannot be safely
    // rendered as a line break by `str::lines()`).
    assert_eq!(
        SignalName::parse("a\rb"),
        Err(NameError::ForbiddenControlChar { char_offset: 1 })
    );
}

#[test]
fn signal_name_rejects_trailing_lone_carriage_return() {
    assert_eq!(
        SignalName::parse("ab\r"),
        Err(NameError::ForbiddenControlChar { char_offset: 2 })
    );
}

#[test]
fn signal_name_rejects_tab() {
    assert_eq!(
        SignalName::parse("a\tb"),
        Err(NameError::ForbiddenControlChar { char_offset: 1 })
    );
}

#[test]
fn signal_name_rejects_empty() {
    assert_eq!(SignalName::parse(""), Err(NameError::Empty));
}

#[test]
fn signal_name_lines_handles_single_line() {
    let name = SignalName::parse("only").expect("valid");
    assert_eq!(collect_lines(name.lines()), vec!["only"]);
}

#[test]
fn user_text_accepts_newline_and_tab() {
    let text = UserText::parse("hello\tworld\nbye").expect("valid user text");
    assert_eq!(text.as_str(), "hello\tworld\nbye");
}

#[test]
fn user_text_accepts_crlf() {
    let text = UserText::parse("hello\r\nworld").expect("CRLF normalized");
    assert_eq!(text.as_str(), "hello\nworld");
    assert_eq!(collect_lines(text.lines()), vec!["hello", "world"]);
}

#[test]
fn user_text_rejects_lone_carriage_return() {
    assert_eq!(
        UserText::parse("a\rb"),
        Err(TextError::ForbiddenControlChar { char_offset: 1 })
    );
}

#[test]
fn user_text_accepts_empty_string() {
    // UserText is allowed to be empty; only control chars besides \n / \t are forbidden.
    assert!(UserText::parse("").is_ok());
}

#[test]
fn font_family_rejects_empty() {
    assert_eq!(FontFamily::parse(""), Err(TextError::Empty));
    assert_eq!(FontFamily::parse("   "), Err(TextError::Empty));
}

#[test]
fn font_family_rejects_quote() {
    // "foo\"bar" — the only entry never closes its quote. The error points
    // at the opening `"` at char offset 3.
    assert_eq!(
        FontFamily::parse("foo\"bar"),
        Err(TextError::ForbiddenCharacter { char_offset: 3 })
    );
}

#[test]
fn font_family_rejects_newline() {
    // FontFamily is single-line; even normalized `\n` is not allowed.
    assert_eq!(
        FontFamily::parse("foo\nbar"),
        Err(TextError::ForbiddenControlChar { char_offset: 3 })
    );
}

#[test]
fn font_family_accepts_typical_value() {
    let family = FontFamily::parse("sans-serif").expect("valid family");
    assert_eq!(family.as_str(), "sans-serif");
    assert_eq!(family.as_unsafe_line().unsafe_text(), "sans-serif");
}

#[test]
fn font_spec_construction() {
    let family = FontFamily::parse("monospace").expect("valid family");
    let spec = FontSpec::new(family, Px(14.0));
    assert_eq!(spec.family().as_str(), "monospace");
    assert_eq!(spec.size(), Px(14.0));
}

#[test]
fn font_spec_to_canvas_css_emits_size_and_family() {
    let family = FontFamily::parse("sans-serif").expect("valid family");
    let spec = FontSpec::new(family, Px(14.0));
    assert_eq!(spec.to_canvas_css(), "14px sans-serif");
}

#[test]
fn font_spec_to_canvas_css_keeps_family_list_verbatim() {
    let family = FontFamily::parse("Helvetica, Arial, sans-serif").expect("valid family");
    let spec = FontSpec::new(family, Px(12.5));
    assert_eq!(spec.to_canvas_css(), "12.5px Helvetica, Arial, sans-serif");
}

#[test]
fn font_family_accepts_quoted_csv_with_generic_fallback() {
    // Per docs/spec/tcml-format.md §「ローカルパラメータ」`font`, the value
    // may be a CSS-style fallback list: families with whitespace go in
    // `"..."`; generic families (sans-serif, monospace, ...) stay bare.
    let family =
        FontFamily::parse("\"Noto Sans CJK JP\", Roboto, sans-serif").expect("valid family");
    assert_eq!(family.as_str(), "\"Noto Sans CJK JP\", Roboto, sans-serif");
}

#[test]
fn font_family_accepts_generic_then_quoted_real_family() {
    let family = FontFamily::parse("monospace, \"Courier New\"").expect("valid family");
    assert_eq!(family.as_str(), "monospace, \"Courier New\"");
}

#[test]
fn font_family_treats_comma_inside_quotes_as_part_of_entry() {
    let family = FontFamily::parse("\"Sans, Bold\"").expect("valid family");
    assert_eq!(family.as_str(), "\"Sans, Bold\"");
}

#[test]
fn font_family_tolerates_extra_whitespace_around_csv_entries() {
    let family = FontFamily::parse("   Roboto  ,  Inter   ").expect("valid family");
    assert_eq!(family.as_str(), "Roboto, Inter");
}

#[test]
fn font_family_rejects_unbalanced_quote() {
    // Opening `"` at offset 0 never closes.
    assert_eq!(
        FontFamily::parse("\"Source Han Sans"),
        Err(TextError::ForbiddenCharacter { char_offset: 0 })
    );
}

#[test]
fn font_family_rejects_quote_after_closing_quote_in_entry() {
    // `"foo"bar"` — the third `"` (offset 8) reopens a quote that never
    // closes; `finish` reports the opening offset of that reopened quote.
    assert_eq!(
        FontFamily::parse("\"foo\"bar\""),
        Err(TextError::ForbiddenCharacter { char_offset: 8 })
    );
}
