use super::format_parse_failure;
use crate::error::ParseFailure;
use std::path::Path;
use tchart_core::errors::ParseError;
use tchart_core::parser::parse;

fn fail_with(source: &str, path_str: &str) -> ParseFailure {
    let error: ParseError = parse(source).expect_err("expected parse error");
    ParseFailure::from_file(Path::new(path_str), source.to_owned(), error)
}

#[test]
fn header_has_no_trailing_period() {
    let failure = fail_with("Sig ?==\n", "sample.tc");
    let rendered = format_parse_failure(&failure);
    let header = rendered
        .lines()
        .next()
        .expect("rendered output has at least one line");
    assert!(header.starts_with("error: "));
    assert!(!header.trim_end().ends_with('.'));
}

#[test]
fn four_components_present() {
    let failure = fail_with("Sig ?==\n", "sample.tc");
    let rendered = format_parse_failure(&failure);
    let lines: Vec<&str> = rendered.lines().collect();
    assert!(lines.len() >= 4);
    assert!(lines[0].starts_with("error: "));
    assert!(lines[1].starts_with(" --> sample.tc:"));
    assert!(lines[2].starts_with("1 | "));
    assert!(lines[3].contains('^'));
    assert!(lines[3].contains('|'));
}

#[test]
fn caret_count_for_length_zero_is_one() {
    // UnclosedQuote → length=0 (insertion point).
    let failure = fail_with("SigA _\"hello\n", "x.tc");
    let rendered = format_parse_failure(&failure);
    let caret_line = rendered
        .lines()
        .find(|line| line.contains('^'))
        .expect("caret line present");
    let caret_count = caret_line
        .chars()
        .filter(|character| *character == '^')
        .count();
    assert_eq!(caret_count, 1);
}

#[test]
fn tab_expanded_to_four_spaces() {
    let failure = fail_with("\tSig ?==\n", "tab.tc");
    let rendered = format_parse_failure(&failure);
    let snippet = rendered
        .lines()
        .find(|line| line.starts_with("1 | "))
        .expect("snippet line present");
    // Original `\t` should have been expanded into 4 spaces in the
    // snippet body following the `1 | ` gutter.
    let body = snippet.strip_prefix("1 | ").unwrap_or(snippet);
    assert!(
        body.starts_with("    "),
        "snippet body did not begin with 4 expanded spaces: {body:?}"
    );
}

/// The column reported in the ` --> FILE:LINE:COL` line must equal the
/// caret column on the line below, per `docs/spec/cli.md` §パースエラー出力形式
/// §構成要素 item 3 ("列番号もこの展開後の桁を指す"). Both must reflect
/// tab-expanded display columns; they cannot disagree.
#[test]
fn location_column_matches_caret_column() {
    let failure = fail_with("\tSig ?==\n", "tab.tc");
    let rendered = format_parse_failure(&failure);
    let lines: Vec<&str> = rendered.lines().collect();
    let location = lines
        .iter()
        .find(|line| line.starts_with(" --> "))
        .expect("location line");
    let caret_line = lines
        .iter()
        .find(|line| line.contains('^'))
        .expect("caret line");
    let reported_column: usize = location
        .rsplit(':')
        .next()
        .and_then(|text| text.trim().parse().ok())
        .expect("location column is numeric");
    let caret_column = caret_column_from(caret_line);
    assert_eq!(
        reported_column, caret_column,
        "location-line column must match caret column"
    );
}

/// Extract the 1-based column position of the first `^` in a caret line
/// (`<gutter spaces> | <padding>^...`), measured from the snippet body start.
fn caret_column_from(caret_line: &str) -> usize {
    let prefix_chars = caret_line
        .split_once('|')
        .map(|(before, _)| before.chars().count() + 2)
        .expect("caret line has `|` gutter");
    let caret_chars_before = caret_line
        .char_indices()
        .find(|(_, character)| *character == '^')
        .map(|(byte_index, _)| caret_line[..byte_index].chars().count())
        .expect("caret line contains `^`");
    caret_chars_before - prefix_chars + 1
}
