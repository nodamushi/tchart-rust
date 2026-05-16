use super::{Color, ColorError};

#[test]
fn parse_none() {
    assert_eq!(Color::parse("none"), Ok(Color::NONE));
    assert_eq!(Color::parse("NONE"), Ok(Color::NONE));
}

#[test]
fn none_constant_is_none() {
    assert!(Color::NONE.is_none());
}

#[test]
fn parse_short_hex_expands() {
    let parsed = Color::parse("#f08").expect("valid #rgb");
    assert_eq!(parsed.to_css_string(), "#ff0088");
}

#[test]
fn parse_long_hex() {
    let parsed = Color::parse("#abcdef").expect("valid #rrggbb");
    assert_eq!(parsed.to_css_string(), "#abcdef");
}

#[test]
fn parse_hex_with_alpha_keeps_alpha() {
    let parsed = Color::parse("#11223344").expect("valid #rrggbbaa");
    assert_eq!(parsed.to_css_string(), "#11223344");
}

#[test]
fn parse_hex_with_full_alpha_drops_alpha_in_css() {
    let parsed = Color::parse("#112233ff").expect("valid #rrggbbff");
    assert_eq!(parsed.to_css_string(), "#112233");
}

#[test]
fn parse_named_color() {
    let parsed = Color::parse("red").expect("named color");
    assert_eq!(parsed.to_css_string(), "red");
}

#[test]
fn parse_named_color_is_case_insensitive() {
    assert_eq!(Color::parse("Red"), Color::parse("red"));
}

#[test]
fn named_color_normalises_to_lowercase_in_output() {
    let parsed = Color::parse("RED").expect("named color in upper case");
    assert_eq!(parsed.to_css_string(), "red");
}

#[test]
fn parse_empty_is_error() {
    assert_eq!(Color::parse(""), Err(ColorError::Empty));
    assert_eq!(Color::parse("   "), Err(ColorError::Empty));
}

#[test]
fn parse_unknown_name_is_error() {
    assert_eq!(
        Color::parse("definitely-not-a-color"),
        Err(ColorError::UnknownName)
    );
}

#[test]
fn parse_bad_hex_length_is_error() {
    assert_eq!(Color::parse("#12"), Err(ColorError::InvalidHexLength));
    assert_eq!(Color::parse("#12345"), Err(ColorError::InvalidHexLength));
}

#[test]
fn parse_bad_hex_digit_is_error() {
    // First `z` is at index 0 within the hex slice (after the `#`).
    assert_eq!(
        Color::parse("#zzzzzz"),
        Err(ColorError::InvalidHexDigit { char_offset: 0 })
    );
}

#[test]
fn round_trip_through_css() {
    for input in ["none", "#abcdef", "#11223344", "red"] {
        let color = Color::parse(input).expect("valid input");
        let rendered = color.to_css_string();
        let again = Color::parse(&rendered).expect("re-parse");
        assert_eq!(color, again, "round-trip failed for {input}");
    }
}
